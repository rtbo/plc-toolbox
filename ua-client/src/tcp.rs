use ua_core::encoding::binary;
use ua_core::transport::tcp;
use ua_core::transport::uacp::{self, Uacp};
use ua_core::{AnySecurityPolicy, SecurityPolicyNone, ua};

use crate::{SecureChannel, secure_channel};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BufferConfig {
    pub send_size: usize,
    pub recv_size: usize,
    pub max_msg_size: usize,
}

impl Default for BufferConfig {
    fn default() -> Self {
        Self {
            send_size: 8192,
            recv_size: 8192,
            max_msg_size: 8192 * 16,
        }
    }
}

impl BufferConfig {
    async fn hello_ack(
        self,
        endpoint: &str,
        uacp: &mut Uacp,
        decoding_ctx: &binary::DecodingCtx,
    ) -> Result<Self, ua_core::transport::Error> {
        let hello_msg = uacp::Hello {
            protocol_version: 0,
            receive_buffer_size: self.recv_size as u32,
            send_buffer_size: self.send_size as u32,
            max_message_size: self.max_msg_size as u32,
            max_chunk_count: 16,
            endpoint_url: endpoint.to_string(),
        };
        uacp.send(hello_msg).await?;
        uacp.flush().await?;

        let ack_msg: uacp::Acknowledge = uacp.recv_typed(decoding_ctx).await?;

        Ok(Self {
            send_size: ack_msg.send_buffer_size as usize,
            recv_size: ack_msg.receive_buffer_size as usize,
            max_msg_size: ack_msg.max_message_size as usize,
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Config {
    pub buffer: BufferConfig,
}

pub struct TcpClient {
    sc: SecureChannel<AnySecurityPolicy>,
    decoding_ctx: binary::DecodingCtx,
}

impl TcpClient {
    pub async fn connect(endpoint: &str, config: Config) -> Result<Self, secure_channel::Error> {
        let transport =
            tcp::Transport::connect(endpoint, config.buffer.send_size, config.buffer.recv_size)
                .await?;

        let decoding_ctx = binary::DecodingCtx::default();

        let mut uacp = Uacp::new(transport);

        let buffer_config = config
            .buffer
            .hello_ack(endpoint, &mut uacp, &decoding_ctx)
            .await?;
        let _config = Config {
            buffer: buffer_config,
        };
        println!("completed hello/ack: {:?}", _config);

        uacp.transport_mut()
            .adjust_buffer_sizes(config.buffer.send_size, config.buffer.recv_size);

        let sp = AnySecurityPolicy::None(SecurityPolicyNone);
        let sc = SecureChannel::open(
            uacp,
            sp,
            1,
            ua::OpenSecureChannelRequest {
                request_header: ua::RequestHeader {
                    timestamp: ua::DateTime::now(),
                    ..Default::default()
                },
                client_protocol_version: 0,
                security_mode: ua::MessageSecurityMode::None,
                request_type: ua::SecurityTokenRequestType::Issue,
                client_nonce: vec![],
                requested_lifetime: 60000,
            },
            &decoding_ctx,
        )
        .await?;

        println!("Secure channel established");

        Ok(Self { sc, decoding_ctx })
    }
}
