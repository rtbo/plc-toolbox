use crate::ua;

pub enum NodeId {
    Numeric {
        namespace_index: u16,
        identifier: u32,
    },
    String {
        namespace_index: u16,
        identifier: String,
    },
    Guid {
        namespace_index: u16,
        identifier: ua::Guid,
    },
    Opaque{
        namespace_index: u16,
        identifier: Vec<u8>,
    },
}

impl NodeId {
    pub fn namespace_index(&self) -> u16 {
        match self {
            NodeId::Numeric { namespace_index, .. } => *namespace_index,
            NodeId::String { namespace_index, .. } => *namespace_index,
            NodeId::Guid { namespace_index, .. } => *namespace_index,
            NodeId::Opaque { namespace_index, .. } => *namespace_index,
        }
    }
}
