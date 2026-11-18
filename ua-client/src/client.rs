use std::sync::Arc;

use tokio::net;
use ua_core::encoding::binary;
use ua_core::encoding::{self, BinaryEncodingId, DataTypeId};
use ua_core::{SecurityPolicyNone, ua};

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
        let timeout = config.timeout.connection;

        let connect_fut = Self::connect_inner(config);

        if timeout != 0 {
            tokio::time::timeout(
                std::time::Duration::from_millis(timeout as u64),
                connect_fut,
            )
            .await
            .map_err(|_| ua::StatusCode::BadTimeout)?
        } else {
            connect_fut.await
        }
    }

    pub fn config(&self) -> &Config {
        match &*self.inner {
            ClientImpl::TcpUascBinary(inner) => inner.config(),
        }
    }

    pub async fn send_request<R>(&self, request: R) -> Result<R::Response, ua::StatusCode>
    where
        R: encoding::Request + binary::Encode + BinaryEncodingId + DataTypeId,
        R::Response: binary::Decode + BinaryEncodingId,
    {
        let timeout = self.config().timeout.request;

        let request_fut = async {
            match &*self.inner {
                ClientImpl::TcpUascBinary(inner) => inner.send_request(request).await,
            }
        };

        if timeout != 0 {
            tokio::time::timeout(
                std::time::Duration::from_millis(timeout as u64),
                request_fut,
            )
            .await
            .map_err(|_| ua::StatusCode::BadTimeout)?
        } else {
            request_fut.await
        }
    }

    pub async fn close(&self) -> Result<(), ua::StatusCode> {
        match &*self.inner {
            ClientImpl::TcpUascBinary(inner) => inner.close().await,
        }
    }
}

impl Client {
    async fn connect_inner(config: Config) -> Result<Self, ua::StatusCode> {
        // Implementation for connecting internally can be added here.
        let security_policy = SecurityPolicyNone;

        if !config.endpoint.starts_with("opc.tcp://") {
            return Err(ua::StatusCode::BadInvalidArgument);
        }

        let url = &config.endpoint["opc.tcp://".len()..];
        let stream = tokio::net::TcpStream::connect(url)
            .await
            .map_err(|_| ua::StatusCode::BadConnectionRejected)?;
        let (read_half, write_half) = stream.into_split();
        let inner = uasc_binary::Client::connect(config, security_policy, write_half, read_half)
            .await
            .map_err(|_| ua::StatusCode::BadConnectionRejected)?;

        Ok(Self {
            inner: Arc::new(ClientImpl::TcpUascBinary(inner)),
        })
    }
}

#[derive(Debug)]
enum ClientImpl {
    TcpUascBinary(uasc_binary::Client<SecurityPolicyNone, net::tcp::OwnedWriteHalf>),
}
