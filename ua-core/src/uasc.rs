//! OPC/UA Secure Conversation (UASC) implementation

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MsgType {
    Open,
    Close,
    Msg,
}

pub trait Msg {
    const MSG_TYPE: MsgType;
}
