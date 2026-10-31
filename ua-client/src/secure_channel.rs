use ua_core::encoding::binary::Encode;
use ua_core::msg::{self, BinaryEncodingId, uasc};
use ua_core::security_policy::HeaderInfo;
use ua_core::{SecurityPolicy, SecurityPolicyNone, ua};

#[derive(Debug)]
pub struct SecureChannel<P: SecurityPolicy = SecurityPolicyNone> {
    security_policy: P,
    channel_id: u32,
    sequence_number: u32,
}

impl<P: SecurityPolicy> SecureChannel<P> {
    pub async fn open(
        mut security_policy: P,
        request_id: u32,
    ) -> Result<Self, ua_core::transport::Error> {
        let msg = ua::OpenSecureChannelRequest {
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
        let encoding_node_id = ua::OpenSecureChannelRequest::binary_encoding_node_id(0);
        let body_len = encoding_node_id.encoded_len() + msg.encoded_len();

        security_policy
            .send_msg_chunk(
                HeaderInfo {
                    msg_type: uasc::MsgType::Open,
                    is_final: uasc::IsFinal::Final,
                    body_len,
                    channel_id: 0,
                    seq_number: 1,
                    request_id,
                },
                |buf| {
                    encoding_node_id.encode(buf);
                    msg.encode(buf);
                },
            )
            .await?;

        todo!()
    }
}
