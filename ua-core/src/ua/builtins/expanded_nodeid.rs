use std::fmt::{self, Write as _};

use bytes::BufMut;

use crate::DataType;
use crate::encoding::binary;
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

impl Default for ExpandedNodeId {
    fn default() -> Self {
        ExpandedNodeId::Numeric(Ns::Index(0), 0, None)
    }
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
    fn type_id() -> u32 {
        18
    }

    fn nullable() -> bool {
        true
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
    fn decode<B: bytes::Buf>(
        buf: &mut B,
        ctx: &binary::DecodingCtx,
    ) -> Result<Self, binary::DecodingError> {
        let (nodeid, flags) = nodeid_decode(buf, ctx)?;
        let mut expanded_nodeid: ExpandedNodeId = nodeid.into();
        if flags & 0x80 != 0 {
            let ns_uri = super::String::decode(buf, ctx)?;
            expanded_nodeid = expanded_nodeid.with_namespace_uri(ns_uri);
        }
        if flags & 0x40 != 0 {
            let server_index = u32::decode(buf, ctx)?;
            expanded_nodeid = expanded_nodeid.with_server_index(server_index);
        }
        Ok(expanded_nodeid)
    }
}

fn escape_uri(f: &mut fmt::Formatter<'_>, uri: &str) -> fmt::Result {
    for c in uri.chars() {
        match c {
            ';' => f.write_str("%3B")?,
            '%' => f.write_str("%25")?,
            c => f.write_char(c)?,
        }
    }
    Ok(())
}

fn unescape_uri(s: &str) -> Result<String, &'static str> {
    let bytes = s.as_bytes();
    let mut out = Vec::with_capacity(bytes.len());
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i] == b'%' {
            let hex = s
                .get(i + 1..i + 3)
                .ok_or("Invalid escape in namespace URI")?;
            out.push(u8::from_str_radix(hex, 16).map_err(|_| "Invalid escape in namespace URI")?);
            i += 3;
        } else {
            out.push(bytes[i]);
            i += 1;
        }
    }
    String::from_utf8(out).map_err(|_| "Invalid UTF-8 in namespace URI")
}

/// Formats as `svr=<serverindex>;ns=<namespaceindex>;<type>=<value>` or
/// `svr=<serverindex>;nsu=<uri>;<type>=<value>` (OPC 10000-6, 5.1.12).
impl fmt::Display for ExpandedNodeId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if let Some(svr) = self.server_index().filter(|s| *s != 0) {
            write!(f, "svr={};", svr)?;
        }
        match self.namespace() {
            Ns::Index(0) => {}
            Ns::Index(ns) => write!(f, "ns={};", ns)?,
            Ns::Uri(uri) => {
                f.write_str("nsu=")?;
                escape_uri(f, uri)?;
                f.write_char(';')?;
            }
        }
        match self {
            ExpandedNodeId::Numeric(_, id, _) => write!(f, "i={}", id),
            ExpandedNodeId::String(_, id, _) => write!(f, "s={}", id),
            ExpandedNodeId::Guid(_, id, _) => write!(f, "g={}", id),
            ExpandedNodeId::Opaque(_, id, _) => {
                use base64::Engine;
                write!(
                    f,
                    "b={}",
                    base64::engine::general_purpose::STANDARD.encode(id)
                )
            }
        }
    }
}

impl std::str::FromStr for ExpandedNodeId {
    type Err = &'static str;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut rest = s;
        let mut server_index = None;
        if let Some(r) = rest.strip_prefix("svr=") {
            let (svr, r) = r.split_once(';').ok_or("Invalid ExpandedNodeId format")?;
            server_index = Some(svr.parse().map_err(|_| "Invalid server index")?);
            rest = r;
        }
        let mut uri = None;
        if let Some(r) = rest.strip_prefix("nsu=") {
            let (u, r) = r.split_once(';').ok_or("Invalid ExpandedNodeId format")?;
            uri = Some(unescape_uri(u)?);
            rest = r;
        }
        let mut result: ExpandedNodeId = rest.parse::<NodeId>()?.into();
        if uri.is_some() && result.namespace_index() != Some(0) {
            return Err("Both namespace index and URI specified");
        }
        if let Some(uri) = uri {
            result = result.with_namespace_uri(uri);
        }
        if let Some(svr) = server_index {
            result = result.with_server_index(svr);
        }
        Ok(result)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn display_and_parse() {
        let cases = [
            (ExpandedNodeId::Numeric(Ns::Index(0), 42, None), "i=42"),
            (
                ExpandedNodeId::Numeric(Ns::Index(2), 42, Some(3)),
                "svr=3;ns=2;i=42",
            ),
            (
                ExpandedNodeId::String(Ns::Uri("http://a/b;c%".into()), "x;y".into(), None),
                "nsu=http://a/b%3Bc%25;s=x;y",
            ),
            (
                ExpandedNodeId::Opaque(Ns::Uri("urn:x".into()), vec![1, 2, 3, 4], Some(1)),
                "svr=1;nsu=urn:x;b=AQIDBA==",
            ),
        ];
        for (id, s) in cases {
            assert_eq!(id.to_string(), s);
            assert_eq!(s.parse::<ExpandedNodeId>().unwrap(), id);
        }
        assert!("ns=1;nsu=urn:x;i=1".parse::<ExpandedNodeId>().is_err());
        assert!("svr=x;i=1".parse::<ExpandedNodeId>().is_err());
    }
}
