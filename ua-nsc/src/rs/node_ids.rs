//! Rust Namespace nodes code generation for OPC/UA.
use std::io;

use crate::parse::{NodeClass, NodeId};

const ALL_NODE_CLASSES: &[NodeClass] = &[
    NodeClass::ReferenceType,
    NodeClass::ObjectType,
    NodeClass::VariableType,
    NodeClass::Object,
    NodeClass::Variable,
    NodeClass::Method,
    NodeClass::View,
];

fn mod_name(node_class: &NodeClass) -> &'static str {
    match node_class {
        NodeClass::ReferenceType => "ref_type",
        NodeClass::ObjectType => "obj_type",
        NodeClass::VariableType => "var_type",
        NodeClass::Object => "obj",
        NodeClass::Variable => "var",
        NodeClass::Method => "method",
        NodeClass::View => "view",
        _ => unreachable!(),
    }
}

fn upper_snake_case(s: &str) -> String {
    // Each capital letter starts a new word
    let mut result = String::new();
    for (i, c) in s.chars().enumerate() {
        if c.is_uppercase() && i > 0 {
            result.push('_');
        }
        result.push(c.to_ascii_uppercase());
    }
    result.replace("__", "_").trim_matches('_').to_string()
    
}

pub fn generate<W>(node_ids: &[NodeId], ns0: bool, out: &mut W) -> std::io::Result<()>
where
    W: io::Write + Unpin,
{
    let mut first = true;
    for node_class in ALL_NODE_CLASSES.iter() {
        let cls_node_ids = node_ids
            .iter()
            .filter(|n| n.class == *node_class)
            .collect::<Vec<_>>();
        if cls_node_ids.is_empty() {
            continue;
        }

        if first {
            first = false;
        } else {
            out.write_all(b"\n")?;
        }

        out.write_all(format!("pub mod {} {{\n", mod_name(node_class)).as_bytes())?;

        if ns0 {
            out.write_all(b"use crate::ua;\n\n")?;
        }

        for node_id in cls_node_ids {
            let name = upper_snake_case(&node_id.name);
            if ns0 {
                out.write_all(
                    format!("    pub const {}: ua::NodeId = ua::NodeId::Numeric(0, {});\n", name, node_id.numeric_id).as_bytes(),
                )?;
            } else {
                out.write_all(
                    format!("    pub const {}_ID: u32 = {};\n", name, node_id.numeric_id).as_bytes(),
                )?;

            }
        }

        out.write_all(b"}\n")?;
    }

    out.flush()
}

