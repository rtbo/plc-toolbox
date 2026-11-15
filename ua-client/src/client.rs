use std::sync::Arc;

use tokio::net;
use ua_core::encoding::binary;
use ua_core::{SecurityPolicyNone, msg, ua};

use crate::{Config, uasc_binary};

/// An OPC/UA client.
///
/// This client supports connecting to an OPC UA server over TCP using the binary protocol.
///
/// This client manages interior mutability and is cheap to clone, so no need to put it into an `Arc` manually.
/// It can be sent across threads or tasks safely and be used concurrently "as-is".
#[derive(Debug, Clone)]
pub struct Client {
    inner: Arc<ClientImpl>,
}

impl Client {
    pub async fn connect(config: Config) -> Result<Self, ua::StatusCode> {
        let security_policy = SecurityPolicyNone;

        let client = if config.endpoint.starts_with("opc.tcp://") {
            let url = &config.endpoint["opc.tcp://".len()..];
            let stream = tokio::net::TcpStream::connect(url)
                .await
                .map_err(|_| ua::StatusCode::BadConnectionRejected)?;
            let (read_half, write_half) = stream.into_split();
            let inner =
                uasc_binary::Client::connect(config, security_policy, write_half, read_half)
                    .await
                    .map_err(|_| ua::StatusCode::BadConnectionRejected)?;
            Self {
                inner: Arc::new(ClientImpl::TcpUascBinary(inner)),
            }
        } else {
            return Err(ua::StatusCode::BadInvalidArgument);
        };

        Ok(client)
    }

    pub fn config(&self) -> &Config {
        match &*self.inner {
            ClientImpl::TcpUascBinary(inner) => inner.config(),
        }
    }

    pub async fn send_request<R>(&self, request: R) -> Result<R::Response, ua::StatusCode>
    where
        R: msg::Request + binary::Encode + msg::BinaryEncodingId,
        R::Response: binary::Decode + msg::BinaryEncodingId,
    {
        match &*self.inner {
            ClientImpl::TcpUascBinary(inner) => inner.send_request(request).await,
        }
    }

    pub async fn close(&self) -> Result<(), ua::StatusCode> {
        match &*self.inner {
            ClientImpl::TcpUascBinary(inner) => inner.close().await,
        }
    }
}

#[derive(Debug)]
enum ClientImpl {
    TcpUascBinary(uasc_binary::Client<SecurityPolicyNone, net::tcp::OwnedWriteHalf>),
}
