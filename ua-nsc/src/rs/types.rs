use std::collections::{HashMap, HashSet};

use std::io;

use crate::codegen::CodeGen;
use crate::parse::{NodeId, types};

const UA_NS: &str = "http://opcfoundation.org/UA/";
const BINARYSCHEMA_NS: &str = "http://opcfoundation.org/BinarySchema/";

pub struct Options {
    pub ua_core_module: String,
}

pub fn generate<W>(
    types: &types::TypeDictionary,
    ua_type_ids: &HashMap<String, NodeId>,
    options: &Options,
    out: &mut W,
) -> io::Result<()>
where
    W: io::Write + Unpin,
{
    let mut ctx = Ctx {
        types,
        ua_type_ids,
        options,
        cg: super::RsCodeGen::new(out),
    };

    if let Some(doc) = &types.doc {
        ctx.cg.write_module_doc_comment(doc)?;
        ctx.cg.write_line("")?;
    }

    ctx.write_module_imports()?;
    ctx.cg.write_line("")?;
    ctx.write_namespace_def()?;
    ctx.cg.write_line("")?;

    let excluded_types = excluded_types(&types.target_namespace);

    for type_def in types
        .types
        .iter()
        .filter(|t| !excluded_types.contains(t.name()))
    {
        // Generate Rust definitions for each type here.
        ctx.generate_type_def(type_def)?;
    }

    ctx.generate_decode_fn(&excluded_types)?;

    ctx.cg.flush()
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
    W: io::Write + Unpin,
{
    types: &'a types::TypeDictionary,
    ua_type_ids: &'a HashMap<String, NodeId>,
    options: &'a Options,
    cg: super::RsCodeGen<W>,
}

impl<'a, W> Ctx<'a, W>
where
    W: io::Write + Unpin,
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

    fn write_module_imports(&mut self) -> io::Result<()> {
        self.cg.write_line(&format!(
            "use {}::encoding::binary;",
            self.options.ua_core_module
        ))?;
        self.cg
            .write_line(&format!("use {}::ua;", self.options.ua_core_module))?;
        self.cg
            .write_line("use ua_macros::{DecodeBinary, EncodeBinary};")?;
        Ok(())
    }

    fn write_namespace_def(&mut self) -> io::Result<()> {
        self.cg.write_line(&format!(
            "pub const NS_URI: &str = \"{}\";",
            self.types.target_namespace
        ))?;
        Ok(())
    }

    fn generate_type_def(&mut self, type_def: &types::Type) -> io::Result<()> {
        match type_def {
            types::Type::Opaque {
                desc,
                length_in_bits,
                byte_order_significant,
            } => {
                self.generate_opaque_type(desc, *length_in_bits, *byte_order_significant)?;
            }
            types::Type::Enumerated {
                desc,
                length_in_bits,
                byte_order_significant,
                is_option_set,
                values,
            } => {
                if values.is_empty() {
                    return Ok(());
                }

                self.generate_enumerated_type(
                    desc,
                    *length_in_bits,
                    *byte_order_significant,
                    *is_option_set,
                    &values,
                )?;
            }
            types::Type::Structured {
                desc,
                fields,
                base_type,
            } => {
                self.generate_structured_type(desc, fields, base_type.as_ref())?;
            }
        }
        // if let types::Type::Structured {
        //     desc, base_type, ..
        // } = type_def
        // {
        //     if self.derives_extension_object(base_type.as_ref()) {
        //         self.generate_extension_object_impl(&desc.name)?;
        //     }
        // }
        self.write_data_type_id_impl(type_def)?;
        self.generate_encoding_impl(type_def)?;

        self.cg.write_line("")?;

        Ok(())
    }

    fn generate_encoding_impl(&mut self, type_def: &types::Type) -> io::Result<()> {
        let Some(type_id) = self.ua_type_ids.get(type_def.name()) else {
            return Ok(());
        };
        let Some(bin_type_id) = self
            .ua_type_ids
            .get(&format!("{}_Encoding_DefaultBinary", type_def.name()))
        else {
            return Ok(());
        };
        let core = self.options.ua_core_module.clone();
        self.cg.start_block(&format!(
            "impl {}::encoding::Value for {} ",
            core,
            type_def.name()
        ))?;

        self.cg
            .start_block("fn datatype_uri(&self) -> &'static str ")?;
        self.cg.write_line("NS_URI")?;
        self.cg.end_block()?; // fn datatype_uri

        self.cg.start_block("fn datatype_id(&self) -> u32 ")?;
        self.cg.write_line(&format!("{}", type_id.numeric_id))?;
        self.cg.end_block()?; // fn datatype_id

        self.cg
            .start_block("fn binary_encoding_id(&self) -> u32 ")?;
        self.cg.write_line(&format!("{}", bin_type_id.numeric_id))?;
        self.cg.end_block()?; // fn binary_encoding_id

        self.cg.end_block()?; // impl DataTypeId
        self.cg.write_line("")?;

        //         self.cg.write_line(&format!(
        //             "impl {}::encoding::binary::Encode for {} ",
        //             core,
        //             type_def.name()
        //         ))
        //
        //         self.cg
        //             .start_block("fn encoded_len(&self) -> u32 ")
        //             ?;
        //         self.cg
        //             .write_line(&format!(
        //                 "{}::encoding::binary::Encode::encoded_len(self)",
        //                 core
        //             ))
        //             ?;
        //         self.cg.end_block()?; // fn binary_encoded_len
        //
        //         self.cg
        //             .start_block(&format!(
        //                 "fn binary_encode(&self, buf: &mut {}::encoding::binary::EncodeBuf<'_>)",
        //                 core
        //             ))
        //             ?;
        //         self.cg
        //             .write_line(&format!(
        //                 "{}::encoding::binary::Encode::encode(self, buf)",
        //                 core
        //             ))
        //             ?;
        //         self.cg.end_block()?; // fn binary_encode
        //
        //         self.cg.end_block()?; // impl DataTypeId
        //         self.cg.write_line("")?;

        Ok(())
    }
    fn write_data_type_id_impl(&mut self, type_def: &types::Type) -> io::Result<()> {
        let Some(type_id) = self.ua_type_ids.get(type_def.name()) else {
            return Ok(());
        };
        let core = self.options.ua_core_module.clone();
        self.cg.start_block(&format!(
            "impl {}::msg::DataTypeId for {} ",
            core,
            type_def.name()
        ))?;
        self.cg.write_line("const NS_URI: &'static str = NS_URI;")?;
        self.cg
            .write_line(&format!("const DATATYPE_ID: u32 = {};", type_id.numeric_id))?;
        self.cg.end_block()?; // impl DataTypeId
        self.cg.write_line("")?;

        if let Some(xml_type_id) = self
            .ua_type_ids
            .get(&format!("{}_Encoding_DefaultXml", type_def.name()))
        {
            self.cg.start_block(&format!(
                "impl {}::msg::XmlEncodingId for {} ",
                core,
                type_def.name()
            ))?;
            self.cg.write_line(&format!(
                "const XML_ENCODING_ID: u32 = {};",
                xml_type_id.numeric_id
            ))?;
            self.cg.end_block()?; // impl XmlEncodingId
            self.cg.write_line("")?;
        }

        if let Some(bin_type_id) = self
            .ua_type_ids
            .get(&format!("{}_Encoding_DefaultBinary", type_def.name()))
        {
            self.cg.start_block(&format!(
                "impl {}::msg::BinaryEncodingId for {} ",
                core,
                type_def.name()
            ))?;
            self.cg.write_line(&format!(
                "const BINARY_ENCODING_ID: u32 = {};",
                bin_type_id.numeric_id
            ))?;
            self.cg.end_block()?; // impl BinaryEncodingId
            self.cg.write_line("")?;
        }
        Ok(())
    }

    fn generate_decode_fn(&mut self, excluded_types: &HashSet<&'static str>) -> io::Result<()> {
        // if self.extension_objects.is_empty() {
        //     return Ok(());
        // }
        let core = self.options.ua_core_module.clone();

        let mut map = phf_codegen::Map::new();
        for typ in self.types.types.iter() {
            let name = typ.name();
            if excluded_types.contains(name) {
                continue;
            }
            if let Some(bin_type_id) = self
                .ua_type_ids
                .get(&format!("{}_Encoding_DefaultBinary", name))
            {
                map.entry(
                    bin_type_id.numeric_id,
                    format!("{}::encoding::binary::decode_value::<{}>", core, name),
                );
            }
        }

        //
        // for (name, id) in &self.extension_objects {
        //     map.entry(
        //         *id,
        //         format!("{}::encoding::binary::decode_value::<{}>", core, name),
        //     );
        // }

        self.cg.write_line(&format!(
            "static BIN_DECODING_FNS: phf::Map<u32, {}::encoding::binary::DecodingFn> = {};",
            core,
            map.build()
        ))?;

        self.cg
            .write_doc_comment(
                "Decode an extension object from the byte stream, using the provided header to identify the type to decode."
            )
            ?;
        self.cg
            .start_block(&format!(
                "pub fn decode_binary_value(buf: &mut binary::DecodeBuf, binary_type_id: u32, ctx: &binary::DecodingCtx) -> Result<Option<Box<dyn {}::encoding::Value>>, ua::StatusCode> ",
                core
            ))
            ?;
        self.cg
            .write_line("BIN_DECODING_FNS.get(&binary_type_id)")?;
        self.cg
            .write_line("    .and_then(|dfn| dfn(buf, binary_type_id, ctx).transpose())")?;
        self.cg.write_line("    .transpose()")?;

        self.cg.end_block()?; // fn decode_extension_object
        Ok(())
    }

    fn generate_opaque_type(
        &mut self,
        desc: &types::TypeDescription,
        _length_in_bits: Option<u32>,
        _byte_order_significant: bool,
    ) -> io::Result<()> {
        if let Some(doc) = desc.doc.as_deref() {
            self.cg.write_doc_comment(doc)?;
        }
        self.cg
            .write_line("#[derive(Debug, Clone, PartialEq, Default)]")?;
        self.cg
            .write_line("#[derive(DecodeBinary, EncodeBinary)]")?;
        self.cg
            .write_line(&format!("pub struct {}(pub ua::ByteString);", &desc.name))?;
        self.cg.write_line("")?;
        Ok(())
    }

    fn generate_enumerated_type(
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
            self.cg.write_doc_comment(doc)?;
        }
        self.cg
            .write_line("#[derive(Debug, Clone, Copy, PartialEq, Default)]")?;
        self.cg
            .write_line("#[derive(DecodeBinary, EncodeBinary)]")?;
        if let Some(len) = length_in_bits {
            if len % 8 == 0 {
                self.cg.write_line(&format!("#[repr(u{})]", len))?;
            }
        }
        self.cg.start_block(&format!("pub enum {} ", &desc.name))?;
        let mut default_issued = false;
        for val in values {
            if let (Some(name), Some(value)) = (&val.name, val.value) {
                if let Some(doc) = val.doc.as_deref() {
                    self.cg.write_doc_comment(doc)?;
                }
                if !default_issued {
                    self.cg.write_line("#[default]")?;
                    default_issued = true;
                }
                if is_option_set {
                    self.cg.write_line(&format!("{} = 0x{:x},", name, value))?;
                } else {
                    self.cg.write_line(&format!("{} = {},", name, value))?;
                }
            }
        }
        self.cg.end_block()?;
        self.cg.write_line("")?;
        Ok(())
    }

    fn generate_structured_type(
        &mut self,
        desc: &types::TypeDescription,
        fields: &[types::Field],
        _base_type: Option<&types::QualName>,
    ) -> io::Result<()> {
        if let Some(doc) = desc.doc.as_deref() {
            self.cg.write_doc_comment(doc)?;
        }
        self.cg.write_line("#[derive(Debug, Default)]")?;
        self.cg
            .write_line("#[derive(DecodeBinary, EncodeBinary)]")?;
        self.cg
            .start_block(&format!("pub struct {} ", &desc.name))?;

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
                self.cg.write_doc_comment(doc)?;
            }
            self.cg
                .write_line(&format!("pub {}: {},", field_name, type_name))?;
        }
        self.cg.end_block()?;
        self.cg.write_line("")?;
        // if is_msg_struct(fields) {
        //     self.cg
        //         .start_block(&format!(
        //             "impl {}::msg::uasc::StaticMsg for {} ",
        //             self.options.ua_core_module, &desc.name
        //         ))
        //         ?;
        //     let msg_type = if self.types.target_namespace.as_str() == UA_NS
        //         && desc.name == "OpenSecureChannelRequest"
        //     {
        //         "Open"
        //     } else if self.types.target_namespace.as_str() == UA_NS
        //         && desc.name == "CloseSecureChannelRequest"
        //     {
        //         "Close"
        //     } else {
        //         "Msg"
        //     };
        //     self.cg
        //         .write_line(&format!(
        //             "const MSG_TYPE: {}::msg::uasc::MsgType = {}::msg::uasc::MsgType::{};",
        //             self.options.ua_core_module, self.options.ua_core_module, msg_type
        //         ))
        //         ?;
        //     self.cg.end_block()?;
        //     self.cg.write_line("")?;
        // }
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
