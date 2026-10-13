use bytes::BufMut;

use crate::DataType;
use crate::msg::binary;
use crate::ua::builtins::nodeid::{
    NodeIdRef, nodeid_decode, nodeidref_encode, nodeidref_encoded_len,
};
use crate::ua::{self, NodeId};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Ns {
    Index(u16),
    Uri(String),
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum ExpandedNodeId {
    Numeric(Ns, u32, Option<u32>),
    String(Ns, String, Option<u32>),
    Guid(Ns, ua::Guid, Option<u32>),
    Opaque(Ns, Vec<u8>, Option<u32>),
}

impl ExpandedNodeId {
    pub fn namespace(&self) -> &Ns {
        match self {
            ExpandedNodeId::Numeric(ns, ..) => ns,
            ExpandedNodeId::String(ns, ..) => ns,
            ExpandedNodeId::Guid(ns, ..) => ns,
            ExpandedNodeId::Opaque(ns, ..) => ns,
        }
    }

    pub fn namespace_index(&self) -> Option<u16> {
        match self.namespace() {
            Ns::Index(ns) => Some(*ns),
            Ns::Uri(_) => None,
        }
    }

    pub fn namespace_uri(&self) -> Option<&str> {
        match self.namespace() {
            Ns::Index(_) => None,
            Ns::Uri(uri) => Some(uri.as_str()),
        }
    }

    pub fn server_index(&self) -> Option<u32> {
        match self {
            ExpandedNodeId::Numeric(_, _, server_index) => *server_index,
            ExpandedNodeId::String(_, _, server_index) => *server_index,
            ExpandedNodeId::Guid(_, _, server_index) => *server_index,
            ExpandedNodeId::Opaque(_, _, server_index) => *server_index,
        }
    }

    pub fn as_nodeid(&self) -> Option<NodeId> {
        match self {
            ExpandedNodeId::Numeric(Ns::Index(ns), id, None) => Some(NodeId::Numeric(*ns, *id)),
            ExpandedNodeId::String(Ns::Index(ns), id, None) => {
                Some(NodeId::String(*ns, id.clone()))
            }
            ExpandedNodeId::Guid(Ns::Index(ns), id, None) => Some(NodeId::Guid(*ns, id.clone())),
            ExpandedNodeId::Opaque(Ns::Index(ns), id, None) => {
                Some(NodeId::Opaque(*ns, id.clone()))
            }
            _ => None,
        }
    }

    pub fn with_namespace_uri(self, namespace_uri: String) -> Self {
        match self {
            ExpandedNodeId::Numeric(_, id, server_index) => {
                ExpandedNodeId::Numeric(Ns::Uri(namespace_uri), id, server_index)
            }
            ExpandedNodeId::String(_, id, server_index) => {
                ExpandedNodeId::String(Ns::Uri(namespace_uri), id, server_index)
            }
            ExpandedNodeId::Guid(_, id, server_index) => {
                ExpandedNodeId::Guid(Ns::Uri(namespace_uri), id, server_index)
            }
            ExpandedNodeId::Opaque(_, id, server_index) => {
                ExpandedNodeId::Opaque(Ns::Uri(namespace_uri), id, server_index)
            }
        }
    }

    pub fn with_server_index(self, server_index: u32) -> Self {
        match self {
            ExpandedNodeId::Numeric(ns, id, _) => {
                ExpandedNodeId::Numeric(ns, id, Some(server_index))
            }
            ExpandedNodeId::String(ns, id, _) => ExpandedNodeId::String(ns, id, Some(server_index)),
            ExpandedNodeId::Guid(ns, id, _) => ExpandedNodeId::Guid(ns, id, Some(server_index)),
            ExpandedNodeId::Opaque(ns, id, _) => ExpandedNodeId::Opaque(ns, id, Some(server_index)),
        }
    }
}

impl From<NodeId> for ExpandedNodeId {
    fn from(node_id: NodeId) -> Self {
        match node_id {
            NodeId::Numeric(ns, id) => ExpandedNodeId::Numeric(Ns::Index(ns), id, None),
            NodeId::String(ns, id) => ExpandedNodeId::String(Ns::Index(ns), id, None),
            NodeId::Guid(ns, id) => ExpandedNodeId::Guid(Ns::Index(ns), id, None),
            NodeId::Opaque(ns, id) => ExpandedNodeId::Opaque(Ns::Index(ns), id, None),
        }
    }
}

impl DataType for ExpandedNodeId {
    fn type_id() -> ua::NodeId {
        ua::NodeId::Numeric(0, 18)
    }

    fn nullable() -> bool {
        true
    }

    fn default() -> Self {
        NodeId::Numeric(0, 0).into()
    }
}

/// Extract the NodeId that is encoded in the stream for this ExpandedNodeId.
fn encoded_nodeid(ex_nodeid: &ExpandedNodeId) -> NodeIdRef<'_> {
    match ex_nodeid {
        ExpandedNodeId::Numeric(Ns::Index(ns), id, _) => NodeIdRef::Numeric(*ns, *id),
        ExpandedNodeId::Numeric(Ns::Uri(_), id, _) => NodeIdRef::Numeric(0, *id),

        ExpandedNodeId::String(Ns::Index(ns), id, _) => NodeIdRef::String(*ns, id.as_str()),
        ExpandedNodeId::String(Ns::Uri(_), id, _) => NodeIdRef::String(0, id.as_str()),

        ExpandedNodeId::Guid(Ns::Index(ns), id, _) => NodeIdRef::Guid(*ns, *id),
        ExpandedNodeId::Guid(Ns::Uri(_), id, _) => NodeIdRef::Guid(0, *id),

        ExpandedNodeId::Opaque(Ns::Index(ns), id, _) => NodeIdRef::Opaque(*ns, id.as_slice()),
        ExpandedNodeId::Opaque(Ns::Uri(_), id, _) => NodeIdRef::Opaque(0, id.as_slice()),
    }
}

impl binary::Encode for ExpandedNodeId {
    fn encoded_len(&self) -> u32 {
        let nodeid_ref = encoded_nodeid(self);
        let ns_uri = self.namespace_uri();
        let srv_idx = self.server_index();
        let mut len = nodeidref_encoded_len(&nodeid_ref);
        if let Some(ns_uri) = ns_uri {
            len += ns_uri.encoded_len();
        }
        if let Some(srv_idx) = srv_idx {
            len += srv_idx.encoded_len();
        }
        len
    }

    fn encode<B: BufMut>(&self, buf: &mut B) {
        let nodeid_ref = encoded_nodeid(self);
        let ns_uri = self.namespace_uri();
        let srv_idx = self.server_index();
        let flags = {
            let mut fl = 0;
            if ns_uri.is_some() {
                fl |= 0x80;
            }
            if srv_idx.is_some() {
                fl |= 0x40;
            }
            fl
        };
        nodeidref_encode(&nodeid_ref, buf, flags);
        if let Some(ns_uri) = ns_uri {
            ns_uri.encode(buf);
        }
        if let Some(srv_idx) = srv_idx {
            srv_idx.encode(buf);
        }
    }
}

impl binary::Decode for ExpandedNodeId {
    fn decode<B: bytes::Buf>(buf: &mut B) -> Result<Self, binary::DecodingError> {
        let (nodeid, flags) = nodeid_decode(buf)?;
        let mut expanded_nodeid: ExpandedNodeId = nodeid.into();
        if flags & 0x80 != 0 {
            let ns_uri = super::String::decode(buf)?;
            expanded_nodeid = expanded_nodeid.with_namespace_uri(ns_uri);
        }
        if flags & 0x40 != 0 {
            let server_index = u32::decode(buf)?;
            expanded_nodeid = expanded_nodeid.with_server_index(server_index);
        }
        Ok(expanded_nodeid)
    }
}
