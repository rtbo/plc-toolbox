use std::collections::{HashMap, HashSet};

use tokio::io;

use crate::codegen::CodeGen;
use crate::parse::{NodeId, opc_bsd};

pub struct Options {
    pub ua_core_module: String,
}

pub async fn generate<W>(
    types: &opc_bsd::TypeDictionary,
    ua_type_ids: &HashMap<String, NodeId>,
    options: &Options,
    out: &mut W,
) -> io::Result<()>
where
    W: io::AsyncWrite + Unpin,
{
    let mut ctx = Ctx {
        _types: types,
        ua_type_ids,
        options,
        cg: super::RsCodeGen::new(out),
    };

    if let Some(doc) = &types.doc {
        ctx.cg.write_module_doc_comment(doc).await?;
        ctx.cg.write_line("").await?;
    }

    ctx.write_module_imports().await?;
    ctx.cg.write_line("").await?;

    let excluded_types = excluded_types(&types.target_namespace);

    for type_def in types
        .types
        .iter()
        .filter(|t| !excluded_types.contains(t.name()))
    {
        // Generate Rust definitions for each type here.
        ctx.generate_type_def(type_def).await?;
    }

    ctx.cg.flush().await
}

fn excluded_types(target_ns: &str) -> HashSet<&'static str> {
    if target_ns == "http://opcfoundation.org/UA/" {
        let types = &[
            "Boolean",
            "SByte",
            "Byte",
            "Int16",
            "UInt16",
            "Int32",
            "UInt32",
            "Int64",
            "UInt64",
            "Float",
            "Double",
            "String",
            "DateTime",
            "Guid",
            "ByteString",
            "XmlElement",
            "NodeId",
            "NodeIdType",
            "TwoByteNodeId",
            "FourByteNodeId",
            "NumericNodeId",
            "StringNodeId",
            "GuidNodeId",
            "ByteStringNodeId",
            "ExpandedNodeId",
            "StatusCode",
            "QualifiedName",
            "LocaleId",
            "LocalizedText",
            "DataValue",
            "Variant",
            "DiagnosticInfo",
            "ExtensionObject",
            "Union",
        ];
        types.iter().copied().collect()
    } else {
        HashSet::new()
    }
}

struct Ctx<'a, W>
where
    W: io::AsyncWrite + Unpin,
{
    _types: &'a opc_bsd::TypeDictionary,
    ua_type_ids: &'a HashMap<String, NodeId>,
    options: &'a Options,
    cg: super::RsCodeGen<W>,
}

impl<'a, W> Ctx<'a, W>
where
    W: io::AsyncWrite + Unpin,
{
    fn translate_namespace(&self, ns: &str) -> Option<String> {
        if ns == "ua" {
            return Some("ua".to_string());
        }

        if ns == "opc" {
            return None;
        }

        None
    }

    fn translate_ns_name(&self, qname: &str) -> String {
        let colon = qname.find(':');
        if let Some(idx) = colon {
            let (ns, name) = qname.split_at(idx);
            let name = &name[1..]; // Skip the colon
            if ns == "opc" {
                return self.translate_opc_name(name);
            }
            if let Some(translated_ns) = self.translate_namespace(ns) {
                format!("{}::{}", translated_ns, name)
            } else {
                name.to_string()
            }
        } else {
            qname.to_string()
        }
    }

    fn translate_opc_name(&self, name: &str) -> String {
        match name {
            "Boolean" => "bool".to_string(),
            "SByte" => "i8".to_string(),
            "Byte" => "u8".to_string(),
            "Int16" => "i16".to_string(),
            "UInt16" => "u16".to_string(),
            "Int32" => "i32".to_string(),
            "UInt32" => "u32".to_string(),
            "Int64" => "i64".to_string(),
            "UInt64" => "u64".to_string(),
            "Float" => "f32".to_string(),
            "Double" => "f64".to_string(),
            "String" => "ua::String".to_string(),
            "DateTime" => "ua::DateTime".to_string(),
            "Guid" => "ua::Guid".to_string(),
            "ByteString" => "ua::ByteString".to_string(),
            "XmlElement" => "ua::XmlElement".to_string(),
            _ => name.to_string(), // Default case, return the name as is
        }
    }

    fn to_snake_case(&self, name: &str) -> String {
        let mut snake_case = String::new();
        let mut prev_underscore = false;
        let mut prev_cap = false;
        for (i, c) in name.chars().enumerate() {
            if c.is_uppercase() && !prev_underscore && !prev_cap {
                if i != 0 {
                    snake_case.push('_');
                }
                for lower_c in c.to_lowercase() {
                    snake_case.push(lower_c);
                }
            } else {
                for lower_c in c.to_lowercase() {
                    snake_case.push(lower_c);
                }
            }
            prev_underscore = c == '_';
            prev_cap = c.is_uppercase();
        }
        snake_case
    }

    async fn write_module_imports(&mut self) -> io::Result<()> {
        self.cg
            .write_line(&format!("use {}::ua;", self.options.ua_core_module))
            .await?;
        self.cg
            .write_line("use ua_macros::{DecodeBinary, EncodeBinary};")
            .await?;
        Ok(())
    }

    async fn generate_type_def(&mut self, type_def: &opc_bsd::Type) -> io::Result<()> {
        match type_def {
            opc_bsd::Type::Opaque {
                desc,
                length_in_bits,
                byte_order_significant,
            } => {
                self.generate_opaque_type(desc, *length_in_bits, *byte_order_significant)
                    .await?;
            }
            opc_bsd::Type::Enumerated {
                desc,
                length_in_bits,
                byte_order_significant,
                is_option_set,
                values,
            } => {
                self.generate_enumerated_type(
                    desc,
                    *length_in_bits,
                    *byte_order_significant,
                    *is_option_set,
                    &values,
                )
                .await?;
            }
            opc_bsd::Type::Structured {
                desc,
                fields,
                base_type,
            } => {
                self.generate_structured_type(desc, fields, base_type.as_deref())
                    .await?;
            }
        }
        self.generate_datatype_impl(type_def).await?;
        Ok(())
    }

    async fn generate_opaque_type(
        &mut self,
        desc: &opc_bsd::TypeDescription,
        _length_in_bits: Option<u32>,
        _byte_order_significant: bool,
    ) -> io::Result<()> {
        if let Some(doc) = desc.doc.as_deref() {
            self.cg.write_doc_comment(doc).await?;
        }
        self.cg
            .write_line("#[derive(Debug, Clone, PartialEq)]")
            .await?;
        self.cg
            .write_line("#[derive(DecodeBinary, EncodeBinary)]")
            .await?;
        self.cg
            .write_line(&format!("pub struct {}(pub ua::ByteString);", &desc.name))
            .await?;
        self.cg.write_line("").await?;
        Ok(())
    }

    async fn generate_enumerated_type(
        &mut self,
        desc: &opc_bsd::TypeDescription,
        length_in_bits: Option<u32>,
        _byte_order_significant: bool,
        is_option_set: bool,
        values: &[opc_bsd::EnumValue],
    ) -> io::Result<()> {
        if values.is_empty() {
            return Ok(());
        }

        if let Some(doc) = desc.doc.as_deref() {
            self.cg.write_doc_comment(doc).await?;
        }
        self.cg
            .write_line("#[derive(Debug, Clone, Copy, PartialEq)]")
            .await?;
        self.cg
            .write_line("#[derive(DecodeBinary, EncodeBinary)]")
            .await?;
        if let Some(len) = length_in_bits {
            if len % 8 == 0 {
                self.cg.write_line(&format!("#[repr(u{})]", len)).await?;
            }
        }
        self.cg
            .start_block(&format!("pub enum {} ", &desc.name))
            .await?;
        for val in values {
            if let (Some(name), Some(value)) = (&val.name, val.value) {
                if let Some(doc) = val.doc.as_deref() {
                    self.cg.write_doc_comment(doc).await?;
                }
                if is_option_set {
                    self.cg
                        .write_line(&format!("{} = 0x{:x},", name, value))
                        .await?;
                } else {
                    self.cg
                        .write_line(&format!("{} = {},", name, value))
                        .await?;
                }
            }
        }
        self.cg.end_block().await?;
        self.cg.write_line("").await?;
        Ok(())
    }

    async fn generate_structured_type(
        &mut self,
        desc: &opc_bsd::TypeDescription,
        fields: &[opc_bsd::Field],
        _base_type: Option<&str>,
    ) -> io::Result<()> {
        if let Some(doc) = desc.doc.as_deref() {
            self.cg.write_doc_comment(doc).await?;
        }
        self.cg
            .write_line("#[derive(Debug, Clone, PartialEq)]")
            .await?;
        self.cg
            .write_line("#[derive(DecodeBinary, EncodeBinary)]")
            .await?;
        self.cg
            .start_block(&format!("pub struct {} ", &desc.name))
            .await?;

        for i in 0..fields.len() {
            let field = &fields[i];
            let Some(type_name) = &field.type_name else {
                continue;
            };

            if is_length_field(fields, i) {
                continue;
            }

            let field_name = self.to_snake_case(&field.name);
            let mut type_name = self.translate_ns_name(type_name);

            let is_array = field.length_field.is_some();
            if is_array {
                type_name = format!("Vec<{}>", type_name);
            }

            if let Some(doc) = field.doc.as_deref() {
                self.cg.write_doc_comment(doc).await?;
            }
            self.cg
                .write_line(&format!("pub {}: {},", field_name, type_name))
                .await?;
        }
        self.cg.end_block().await?;
        self.cg.write_line("").await?;
        Ok(())
    }

    async fn generate_datatype_impl(&mut self, type_def: &opc_bsd::Type) -> io::Result<()> {
        if let opc_bsd::Type::Enumerated { values, .. } = type_def {
            if values.is_empty() {
                return Ok(());
            }
        }

        if let Some(type_id) = self.ua_type_ids.get(type_def.name()) {
            self.cg
                .start_block(&format!(
                    "impl {}::DataType for {} ",
                    self.options.ua_core_module,
                    type_def.name(),
                ))
                .await?;

            self.cg.start_block("fn type_id() -> ua::NodeId ").await?;
            // FIXME: handle other namespaces
            self.cg
                .write_line(&format!("ua::NodeId::Numeric(0, {})", type_id.numeric_id))
                .await?;
            self.cg.end_block().await?;

            self.cg.start_block("fn nullable() -> bool ").await?;
            self.cg.write_line("false").await?;
            self.cg.end_block().await?;

            self.cg.start_block("fn default() -> Self ").await?;
            match type_def {
                opc_bsd::Type::Opaque { .. } => {
                    self.cg
                        .write_line(&format!(
                            "{}(<ua::ByteString as {}::DataType>::default())",
                            type_def.name(),
                            self.options.ua_core_module
                        ))
                        .await?;
                }
                opc_bsd::Type::Enumerated { values, .. } => {
                    if let Some(first_value) = values.first() {
                        self.cg
                            .write_line(&format!(
                                "{}::{}",
                                type_def.name(),
                                first_value.name.as_deref().unwrap_or("default()")
                            ))
                            .await?;
                    }
                }
                opc_bsd::Type::Structured { fields, .. } => {
                    self.cg
                        .start_block(&format!("{} ", type_def.name()))
                        .await?;
                    for i in 0..fields.len() {
                        let field = &fields[i];
                        if is_length_field(fields, i) {
                            continue;
                        }
                        let field_name = self.to_snake_case(&field.name);
                        let is_array = field.length_field.is_some();
                        if is_array {
                            self.cg
                                .write_line(&format!("{}: Vec::new(),", field_name))
                                .await?;
                        } else {
                            self.cg
                                .write_line(&format!(
                                    "{}: {}::DataType::default(),",
                                    field_name, self.options.ua_core_module
                                ))
                                .await?;
                        }
                    }
                    self.cg.end_block().await?;
                }
            }
            self.cg.end_block().await?;

            self.cg.end_block().await?;
            self.cg.write_line("").await?;
        }
        Ok(())
    }
}

fn is_length_field(fields: &[opc_bsd::Field], idx: usize) -> bool {
    let name = &fields[idx].name;
    for j in (idx + 1..fields.len()).chain(0..idx) {
        let f = &fields[j];
        if f.length_field.as_deref() == Some(name) {
            return true;
        }
    }
    false
}
