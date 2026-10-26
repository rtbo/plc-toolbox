#![allow(dead_code)]
use ua_core::transport::tcp;
use ua_core::ua;

#[derive(Debug)]
pub struct SecureChannel {
    channel_id: u32,
}

impl SecureChannel {
    pub fn open(_transport: tcp::Transport) -> Result<Self, ua_core::transport::Error> {
        let _msg = ua::OpenSecureChannelRequest {
            request_header: ua::RequestHeader {
                timestamp: ua::DateTime::now(),
                ..Default::default()
            },
            client_protocol_version: 0,
            request_type: ua::SecurityTokenRequestType::Issue,
            security_mode: ua::MessageSecurityMode::None,
            client_nonce: vec![],
            requested_lifetime: 60000,
        };

        todo!()
    }
}
