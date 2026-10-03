//! Typescript Types code generation for OPC/UA.
use std::collections::HashMap;
use tokio::io::{self, AsyncWriteExt};

use crate::codegen::CodeGen;
use crate::parse::NodeId;
use crate::parse::opc_bsd::{EnumValue, Field, Type, TypeDescription, TypeDictionary};

pub async fn generate<W>(
    types: &TypeDictionary,
    ua_type_ids: &HashMap<String, NodeId>,
    out: &mut W,
) -> io::Result<()>
where
    W: io::AsyncWrite + Unpin,
{
    let mut ctx = Ctx {
        ua_type_ids,
        cg: super::TsCodeGen::new(out),
    };

    if let Some(doc) = &types.doc {
        ctx.write_module_doc_comment(doc).await?;
    }

    // Include built-in TypeScript types.
    ctx.write_builtin_types().await?;

    for type_def in &types.types {
        // Generate TypeScript definitions for each type here.
        ctx.generate_type_def(type_def).await?;
    }

    ctx.cg.output_mut().flush().await
}

struct Ctx<'a, W>
where
    W: io::AsyncWrite + Unpin,
{
    ua_type_ids: &'a HashMap<String, NodeId>,
    cg: super::TsCodeGen<W>,
}

impl<'a, W> Ctx<'a, W>
where
    W: io::AsyncWrite + Unpin,
{
    async fn write_line(&mut self, txt: &str) -> io::Result<()> {
        self.cg.write_line(txt).await
    }

    async fn start_block(&mut self, prefix: &str) -> io::Result<()> {
        self.cg.start_block(prefix).await
    }

    async fn end_block(&mut self) -> io::Result<()> {
        self.cg.end_block().await
    }

    async fn write_doc_comment(&mut self, doc: &str) -> io::Result<()> {
        let doc = doc.trim_end(); // remove possible trailing '\n'

        self.write_line("/**").await?;
        for line in doc.lines() {
            self.write_line(&format!(" * {}", line)).await?;
        }
        self.write_line(" */").await?;
        Ok(())
    }

    async fn write_module_doc_comment(&mut self, doc: &str) -> io::Result<()> {
        self.write_line("/**").await?;
        for line in doc.lines() {
            self.write_line(&format!(" * {}", line)).await?;
        }
        self.write_line(" * @module").await?;
        self.write_line(" */").await?;
        Ok(())
    }

    async fn write_type_doc_comment(
        &mut self,
        desc: &TypeDescription,
        class: &str,
    ) -> io::Result<()> {
        let mut doc_comment = String::new();
        if let Some(doc) = &desc.doc {
            doc_comment.push_str(doc);
            doc_comment.push('\n');
        }
        doc_comment.push_str(format!("{} {}\n", desc.name, class).as_str());
        if let Some(type_id) = self.ua_type_ids.get(&desc.name) {
            doc_comment.push_str(format!("    node id: \"{}\"\n", type_id.node_id_str()).as_str());
            doc_comment.push_str(format!("    node class: {}\n", type_id.class).as_str());
        }
        self.write_doc_comment(&doc_comment).await?;
        Ok(())
    }

    async fn write_builtin_types(&mut self) -> io::Result<()> {
        let builtins_code = include_str!("builtins.ts");
        self.cg.output_mut().write(builtins_code.as_bytes()).await?;
        Ok(())
    }

    async fn generate_type_def(&mut self, type_def: &Type) -> io::Result<()> {
        let name = type_def.name();
        if excluded_type(name) {
            return Ok(());
        }

        match type_def {
            Type::Opaque { desc, .. } => {
                self.generate_opaque(desc).await?;
            }
            Type::Enumerated {
                desc,
                values,
                is_option_set,
                ..
            } => {
                self.generate_enumeration(desc, values, *is_option_set)
                    .await?;
            }
            Type::Structured {
                desc,
                base_type,
                fields,
            } => {
                self.generate_structured(desc, base_type, fields).await?;
            }
        }

        Ok(())
    }

    async fn generate_opaque(&mut self, desc: &TypeDescription) -> io::Result<()>
    where
        W: io::AsyncWrite + Unpin,
    {
        self.write_line("").await?;
        self.write_type_doc_comment(desc, "opaque type").await?;
        self.write_line(&format!("export type {} = ByteString;", desc.name))
            .await?;
        Ok(())
    }

    async fn generate_enumeration(
        &mut self,
        desc: &TypeDescription,
        values: &[EnumValue],
        is_option_set: bool,
    ) -> io::Result<()> {
        self.write_line("").await?;

        let type_name = remove_namespace(&desc.name);
        let class = if is_option_set {
            "option set"
        } else {
            "enumeration"
        };
        self.write_type_doc_comment(desc, class).await?;

        self.start_block(&format!("export const enum {} ", type_name))
            .await?;

        for value in values {
            if let (Some(name), Some(value)) = (&value.name, &value.value) {
                if is_option_set {
                    self.write_line(&format!("{} = 0x{:x},", name, value))
                        .await?;
                } else {
                    self.write_line(&format!("{} = {},", name, value)).await?;
                }
            }
        }
        self.end_block().await?;
        Ok(())
    }

    async fn generate_structured(
        &mut self,
        desc: &TypeDescription,
        base_type: &Option<String>,
        fields: &[Field],
    ) -> io::Result<()> {
        self.write_line("").await?;

        self.write_type_doc_comment(desc, "structure").await?;

        let extends = if let Some(base_type) = base_type {
            format!(" extends {}", remove_namespace(base_type))
        } else {
            String::new()
        };
        self.start_block(&format!("export interface {}{} ", desc.name, extends))
            .await?;

        if base_type.as_deref() == Some("ua:ExtensionObject") {
            let node_id_str = self.ua_type_ids.get(&desc.name).map(|id| id.node_id_str());
            if let Some(node_id_str) = node_id_str {
                self.write_line(&format!("UaTypeId?: \"{}\",", node_id_str))
                    .await?;
            }
        }

        for i in 0..fields.len() {
            let field = &fields[i];
            let name = &field.name;
            let Some(type_name) = &field.type_name else {
                continue;
            };

            let is_length_field = {
                let mut found = false;
                for j in (i + 1..fields.len()).chain(0..i) {
                    let f = &fields[j];
                    if f.length_field.as_deref() == Some(name) {
                        found = true;
                        break;
                    }
                }
                found
            };

            if is_length_field {
                continue;
            }

            let is_array = field.length_field.is_some();
            let array_suffix = if is_array { "[]" } else { "" };
            self.write_line(&format!(
                "{}?: {}{};",
                name,
                remove_namespace(type_name),
                array_suffix
            ))
            .await?;
        }

        self.end_block().await?;
        Ok(())
    }
}

fn remove_namespace(type_name: &str) -> &str {
    let idx = type_name.find(':');
    if let Some(idx) = idx {
        &type_name[idx + 1..]
    } else {
        type_name
    }
}

const BUILTIN_TYPES: &[&str] = &[
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
    "ExpandedNodeId",
    "StatusCode",
    "QualifiedName",
    "LocalizedText",
    "ExtensionObject",
    "DataValue",
    "Variant",
    "DiagnosticInfo",
];

const EXCLUDED_TYPES: &[&str] = &[
    "NodeIdType",
    "TwoByteNodeId",
    "FourByteNodeId",
    "NumericNodeId",
    "StringNodeId",
    "GuidNodeId",
    "ByteStringNodeId",
];

fn excluded_type(type_name: &str) -> bool {
    let type_name = remove_namespace(type_name);
    BUILTIN_TYPES.contains(&type_name) || EXCLUDED_TYPES.contains(&type_name)
}
