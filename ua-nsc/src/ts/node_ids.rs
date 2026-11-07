//! Typescript Namespace nodes code generation for OPC/UA.
use std::io;

use crate::parse::{NodeClass, NodeId};

const ALL_NODE_CLASSES: &[NodeClass] = &[
    NodeClass::DataType,
    NodeClass::ReferenceType,
    NodeClass::ObjectType,
    NodeClass::VariableType,
    NodeClass::Object,
    NodeClass::Variable,
    NodeClass::Method,
    NodeClass::View,
    NodeClass::Unspecified,
];

fn enum_name(node_class: &NodeClass) -> &'static str {
    match node_class {
        NodeClass::DataType => "DataTypeId",
        NodeClass::ReferenceType => "ReferenceTypeId",
        NodeClass::ObjectType => "ObjectTypeId",
        NodeClass::VariableType => "VariableTypeId",
        NodeClass::Object => "ObjectId",
        NodeClass::Variable => "VariableId",
        NodeClass::Method => "MethodId",
        NodeClass::View => "ViewId",
        NodeClass::Unspecified => "UnspecifiedId",
    }
}

pub fn generate<W>(node_ids: &[NodeId], out: &mut W) -> std::io::Result<()>
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

        out.write_all(format!("export const enum {} {{\n", enum_name(node_class)).as_bytes())?;

        for node_id in cls_node_ids {
            out.write_all(
                format!("    {} = \"i={}\",\n", node_id.name, node_id.numeric_id).as_bytes(),
            )?;
        }

        out.write_all(b"}\n")?;
    }

    out.flush()
}
