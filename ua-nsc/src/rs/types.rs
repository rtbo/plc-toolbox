use std::collections::{HashMap, HashSet};

use tokio::io;

use crate::codegen::CodeGen;
use crate::parse::{NodeId, types};

const UA_NS: &str = "http://opcfoundation.org/UA/";
const BINARYSCHEMA_NS: &str = "http://opcfoundation.org/BinarySchema/";

pub struct Options {
    pub ua_core_module: String,
}

pub async fn generate<W>(
    types: &types::TypeDictionary,
    ua_type_ids: &HashMap<String, NodeId>,
    options: &Options,
    out: &mut W,
) -> io::Result<()>
where
    W: io::AsyncWrite + Unpin,
{
    let mut ctx = Ctx {
        types,
        ua_type_ids,
        options,
        cg: super::RsCodeGen::new(out),
        extension_objects: Vec::new(),
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

    ctx.generate_decode_fn().await?;

    ctx.cg.flush().await
}

fn excluded_types(target_ns: &str) -> HashSet<&'static str> {
    if target_ns == UA_NS {
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
    types: &'a types::TypeDictionary,
    ua_type_ids: &'a HashMap<String, NodeId>,
    options: &'a Options,
    cg: super::RsCodeGen<W>,
    /// (type name, type identifier) of the generated types deriving from ExtensionObject.
    extension_objects: Vec<(String, u32)>,
}

impl<'a, W> Ctx<'a, W>
where
    W: io::AsyncWrite + Unpin,
{
    fn translate_namespace(&self, ns: &str) -> Option<&str> {
        // TODO: handle user supplied NS

        if ns == UA_NS { Some("ua") } else { None }
    }

    fn translate_ns_name(&self, qname: &types::QualName) -> String {
        let name = qname.name.as_str();
        let Some(ns) = qname.namespace.as_deref() else {
            return name.to_string();
        };
        if ns == BINARYSCHEMA_NS {
            return self.translate_opc_name(name).to_string();
        }
        match self.translate_namespace(ns) {
            Some(translated_ns) => format!("{}::{}", translated_ns, name),
            None => name.to_string(),
        }
    }

    fn translate_opc_name<'b>(&self, name: &'b str) -> &'b str {
        match name {
            "Boolean" => "bool",
            "SByte" => "i8",
            "Byte" => "u8",
            "Int16" => "i16",
            "UInt16" => "u16",
            "Int32" => "i32",
            "UInt32" => "u32",
            "Int64" => "i64",
            "UInt64" => "u64",
            "Float" => "f32",
            "Double" => "f64",
            "String" => "ua::String",
            "DateTime" => "ua::DateTime",
            "Guid" => "ua::Guid",
            "ByteString" => "ua::ByteString",
            "XmlElement" => "ua::XmlElement",
            _ => name, // Default case, return the name as is
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
            .write_line(&format!(
                "use {}::encoding::binary;",
                self.options.ua_core_module
            ))
            .await?;
        self.cg
            .write_line(&format!("use {}::ua;", self.options.ua_core_module))
            .await?;
        self.cg
            .write_line("use ua_macros::{DecodeBinary, EncodeBinary};")
            .await?;
        Ok(())
    }

    async fn generate_type_def(&mut self, type_def: &types::Type) -> io::Result<()> {
        match type_def {
            types::Type::Opaque {
                desc,
                length_in_bits,
                byte_order_significant,
            } => {
                self.generate_opaque_type(desc, *length_in_bits, *byte_order_significant)
                    .await?;
            }
            types::Type::Enumerated {
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
            types::Type::Structured {
                desc,
                fields,
                base_type,
            } => {
                self.generate_structured_type(desc, fields, base_type.as_ref())
                    .await?;
            }
        }
        if let types::Type::Structured {
            desc, base_type, ..
        } = type_def
        {
            if self.derives_extension_object(base_type.as_ref()) {
                self.generate_extension_object_impl(&desc.name).await?;
            }
        }
        Ok(())
    }

    fn is_local_namespace(&self, qname: &types::QualName) -> bool {
        match qname.namespace.as_deref() {
            None => true,
            Some(ns) => ns == self.types.target_namespace,
        }
    }

    /// Whether the type has ExtensionObject as base type, directly or through other types of the dictionary.
    fn derives_extension_object(&self, base_type: Option<&types::QualName>) -> bool {
        let mut base = base_type;
        // The depth limit protects against cyclic definitions.
        for _ in 0..32 {
            let Some(b) = base else {
                return false;
            };
            if is_extension_object_type(b) {
                return true;
            }
            if !self.is_local_namespace(b) {
                return false;
            }
            base = self
                .types
                .types
                .iter()
                .find_map(|t| match t {
                    types::Type::Structured {
                        desc, base_type, ..
                    } if desc.name == b.name => Some(base_type.as_ref()),
                    _ => None,
                })
                .flatten();
        }
        false
    }

    async fn generate_extension_object_impl(&mut self, name: &str) -> io::Result<()> {
        let Some(type_id) = self.ua_type_ids.get(name) else {
            return Ok(());
        };
        let core = self.options.ua_core_module.clone();
        self.extension_objects
            .push((name.to_string(), type_id.numeric_id));

        self.cg
            .start_block(&format!(
                "impl {}::ua::extension_object::Body for {} ",
                core, name
            ))
            .await?;
        self.cg
            .start_block("fn binary_body_encoded_len(&self) -> u32 ")
            .await?;
        self.cg
            .write_line(&format!(
                "{}::encoding::binary::Encode::encoded_len(self)",
                core
            ))
            .await?;
        self.cg.end_block().await?; // fn binary_body_encoded_len
        self.cg
            .start_block(&format!(
                "fn binary_body_encode(&self, mut buf: &mut dyn {}::encoding::binary::BufMut) ",
                core
            ))
            .await?;
        self.cg
            .write_line(&format!(
                "{}::encoding::binary::Encode::encode(self, &mut buf)",
                core
            ))
            .await?;
        self.cg.end_block().await?; // fn binary_body_encode
        self.cg.end_block().await?; // impl Body
        self.cg.write_line("").await?;
        Ok(())
    }

    async fn generate_decode_fn(&mut self) -> io::Result<()> {
        if self.extension_objects.is_empty() {
            return Ok(());
        }
        let core = self.options.ua_core_module.clone();

        let mut map = phf_codegen::Map::new();
        for (name, id) in &self.extension_objects {
            map.entry(
                *id,
                format!("{}::ua::extension_object::decode_body::<{}>", core, name),
            );
        }

        self.cg.write_line(&format!("static DECODING_FNS: phf::Map<u32, {}::ua::extension_object::BinaryDecodingFn> = {};", core, map.build())).await?;

        self.cg
            .write_doc_comment(
                "Decode an extension object from the byte stream, using the provided header to identify the type to decode."
            )
            .await?;
        self.cg
            .start_block(&format!(
                "pub fn decode_extension_object(buf: &mut dyn binary::Buf, header: &ua::extension_object::Header, ctx: &binary::DecodingCtx) -> Result<Option<Box<dyn ua::extension_object::Body>>, binary::DecodingError> ",
            ))
            .await?;
        self.cg
            .write_line("header.type_id.numeric_identifier()")
            .await?;
        self.cg
            .write_line("    .and_then(|id| DECODING_FNS.get(&id))")
            .await?;
        self.cg
            .write_line("    .and_then(|dfn| dfn(buf, header, ctx).transpose())")
            .await?;
        self.cg.write_line("    .transpose()").await?;

        self.cg.end_block().await?; // fn decode_extension_object
        Ok(())
    }

    async fn generate_opaque_type(
        &mut self,
        desc: &types::TypeDescription,
        _length_in_bits: Option<u32>,
        _byte_order_significant: bool,
    ) -> io::Result<()> {
        if let Some(doc) = desc.doc.as_deref() {
            self.cg.write_doc_comment(doc).await?;
        }
        self.cg
            .write_line("#[derive(Debug, Clone, PartialEq, Default)]")
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
        desc: &types::TypeDescription,
        length_in_bits: Option<u32>,
        _byte_order_significant: bool,
        is_option_set: bool,
        values: &[types::EnumValue],
    ) -> io::Result<()> {
        if values.is_empty() {
            return Ok(());
        }

        if let Some(doc) = desc.doc.as_deref() {
            self.cg.write_doc_comment(doc).await?;
        }
        self.cg
            .write_line("#[derive(Debug, Clone, Copy, PartialEq, Default)]")
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
        let mut default_issued = false;
        for val in values {
            if let (Some(name), Some(value)) = (&val.name, val.value) {
                if let Some(doc) = val.doc.as_deref() {
                    self.cg.write_doc_comment(doc).await?;
                }
                if !default_issued {
                    self.cg.write_line("#[default]").await?;
                    default_issued = true;
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
        desc: &types::TypeDescription,
        fields: &[types::Field],
        _base_type: Option<&types::QualName>,
    ) -> io::Result<()> {
        if let Some(doc) = desc.doc.as_deref() {
            self.cg.write_doc_comment(doc).await?;
        }
        self.cg.write_line("#[derive(Debug, Default)]").await?;
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
        if is_msg_struct(fields) {
            self.cg
                .start_block(&format!(
                    "impl {}::uasc::TypedMsg for {} ",
                    self.options.ua_core_module, &desc.name
                ))
                .await?;
            let msg_type = if self.types.target_namespace.as_str() == UA_NS
                && desc.name == "OpenSecureChannelRequest"
            {
                "Open"
            } else if self.types.target_namespace.as_str() == UA_NS
                && desc.name == "CloseSecureChannelRequest"
            {
                "Close"
            } else {
                "Msg"
            };
            self.cg
                .write_line(&format!(
                    "const MSG_TYPE: {}::uasc::MsgType = {}::uasc::MsgType::{};",
                    self.options.ua_core_module, self.options.ua_core_module, msg_type
                ))
                .await?;
            self.cg.end_block().await?;
            self.cg.write_line("").await?;
        }
        Ok(())
    }
}

fn is_length_field(fields: &[types::Field], idx: usize) -> bool {
    let name = &fields[idx].name;
    for j in (idx + 1..fields.len()).chain(0..idx) {
        let f = &fields[j];
        if f.length_field.as_deref() == Some(name) {
            return true;
        }
    }
    false
}

fn is_extension_object_type(type_name: &types::QualName) -> bool {
    type_name.namespace.as_deref() == Some(UA_NS) && type_name.name == "ExtensionObject"
}

fn is_request_type(field: &types::Field) -> bool {
    let Some(type_name) = &field.type_name else {
        return false;
    };
    type_name.namespace.as_deref() == Some(UA_NS) && type_name.name == "RequestHeader"
}

fn is_response_type(field: &types::Field) -> bool {
    let Some(type_name) = &field.type_name else {
        return false;
    };
    type_name.namespace.as_deref() == Some(UA_NS) && type_name.name == "ResponseHeader"
}

fn is_msg_struct(fields: &[types::Field]) -> bool {
    fields.len() > 1 && (is_request_type(&fields[0]) || is_response_type(&fields[0]))
}
