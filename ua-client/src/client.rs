use std::sync::Arc;

use tokio::net;
use tokio::sync::RwLock;
use ua_core::encoding::{self, BinaryEncodingId, DataTypeId, binary};
use ua_core::{SecurityPolicyNone, ns0, ua};

use crate::{Config, config, uasc_binary};

/// An OPC/UA client.
///
/// This client supports connecting to an OPC UA server over TCP using the binary protocol.
///
/// This client manages interior mutability and is cheap to clone, so no need to put it into an `Arc` manually.
/// It can be sent across threads or tasks safely and be used concurrently "as-is".
#[derive(Debug, Clone)]
pub struct Client {
    inner: Arc<ClientInner>,
}

impl Client {
    pub async fn connect(config: Config) -> Result<Self, ua::StatusCode> {
        let timeout = config.timeout.connection;

        let connect_fut = Self::connect_inner(config);

        let connect_res = if timeout != 0 {
            tokio::time::timeout(
                std::time::Duration::from_millis(timeout as u64),
                connect_fut,
            )
            .await
            .map_err(|_| ua::StatusCode::BadTimeout)?
        } else {
            connect_fut.await
        };

        let client = connect_res?;

        if let Some(session) = &client.config().session {
            client.open_session(session).await?;
        }

        if !client.config().external_ns.is_empty() {
            let ns_array = client.read_ns_array().await?;
            match client.stack() {
                ClientStack::TcpUascBinary(stack) => stack.set_ns_array(ns_array).await,
            }
        }

        Ok(client)
    }

    pub fn config(&self) -> &Config {
        &*self.inner.config
    }

    pub async fn send_request<R>(&self, request: R) -> Result<R::Response, ua::StatusCode>
    where
        R: encoding::Request + binary::Encode + BinaryEncodingId + DataTypeId,
        R::Response: binary::Decode + BinaryEncodingId,
    {
        let timeout = self.config().timeout.request;

        let request_fut = async {
            match self.stack() {
                ClientStack::TcpUascBinary(inner) => inner.send_request(request).await,
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

    /// Get the server's namespace array.
    /// 
    /// The namespace array is cached in the client and a deep copy is returned. 
    /// If the cache is empty, it will be read from the server.
    /// 
    /// Note: When the client is configured with external namespace decoding function,
    /// the namespace array is cached during the connection establishment.
    pub async fn server_namespace_array(&self) -> Result<Vec<String>, ua::StatusCode> {
        let ns_array = match self.stack() {
            ClientStack::TcpUascBinary(inner) => inner.get_ns_array().await?,
        };
        if !ns_array.is_empty() {
            return Ok(ns_array);
        }

        let ns_array = self.read_ns_array().await?;
        match self.stack() {
            ClientStack::TcpUascBinary(inner) => inner.set_ns_array(ns_array.clone()).await,
        }
        Ok(ns_array)
    }

    pub async fn close(&self) -> Result<(), ua::StatusCode> {
        match self.stack() {
            ClientStack::TcpUascBinary(inner) => inner.close().await,
        }
    }
}

impl Client {
    fn stack(&self) -> &ClientStack {
        &(*self.inner).stack
    }

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
        let stack = uasc_binary::Client::connect(config, security_policy, write_half, read_half)
            .await
            .map_err(|_| ua::StatusCode::BadConnectionRejected)?;

        Ok(Self {
            inner: Arc::new(ClientInner {
                config: stack.config(),
                session: RwLock::new(None),
                stack: ClientStack::TcpUascBinary(stack),
            }),
        })
    }

    async fn open_session(&self, session: &config::Session) -> Result<(), ua::StatusCode> {
        let timeout = self.config().timeout.request;

        let req = ua::CreateSessionRequest {
            request_header: req_header(timeout),
            session_name: session.name.as_ref().cloned().unwrap_or_default(),
            client_description: ua::ApplicationDescription {
                application_uri: session.application_uri.clone(),
                ..Default::default()
            },
            requested_session_timeout: session.timeout as f64,
            ..Default::default()
        };

        let resp = self.send_request(req).await?;

        let auth_token = resp.authentication_token;

        let req = ua::ActivateSessionRequest {
            request_header: auth_req_header(auth_token.clone(), timeout),
            ..Default::default()
        };
        let resp = self.send_request(req).await?;

        let session_state = SessionState {
            auth_token,
            _timeout: session.timeout,
            _server_nonce: resp.server_nonce,
        };
        *self.inner.session.write().await = Some(session_state);
        Ok(())
    }

    async fn auth_req_header(&self) -> Result<ua::RequestHeader, ua::StatusCode> {
        let session = self.inner.session.read().await;
        let Some(session) = session.as_ref() else {
            return Err(ua::StatusCode::BadSessionNotActivated);
        };

        let timeout = self.config().timeout.request;

        Ok(auth_req_header(session.auth_token.clone(), timeout))
    }

    async fn read_ns_array(&self) -> Result<Vec<String>, ua::StatusCode> {
        let req = ua::ReadRequest {
            request_header: self.auth_req_header().await?,
            nodes_to_read: vec![ua::ReadValueId {
                node_id: ns0::var::SERVER_NAMESPACE_ARRAY,
                attribute_id: ua::AttributeId::Value as _,
                ..Default::default()
            }],
            ..Default::default()
        };

        let resp = self.send_request(req).await?;
        let ns_array = match resp.results.into_iter().next() {
            Some(ua::DataValue {
                status_code,
                value: ua::Variant::String(ua::variant::Value::Array(ns_array)),
                ..
            }) => {
                status_code.check_good()?;
                ns_array
            }
            Some(_) => return Err(ua::StatusCode::BadNoMatch),
            None => return Err(ua::StatusCode::BadNoData),
        };

        Ok(ns_array)
    }
}

#[derive(Debug, Default)]
struct SessionState {
    auth_token: ua::NodeId,
    _timeout: u32,
    _server_nonce: Vec<u8>,
}

#[derive(Debug)]
struct ClientInner {
    config: Arc<Config>,
    session: RwLock<Option<SessionState>>,
    stack: ClientStack,
}

#[derive(Debug)]
enum ClientStack {
    TcpUascBinary(uasc_binary::Client<SecurityPolicyNone, net::tcp::OwnedWriteHalf>),
}

pub(crate) fn req_header(timeout: u32) -> ua::RequestHeader {
    ua::RequestHeader {
        timestamp: ua::DateTime::now(),
        timeout_hint: timeout,
        ..Default::default()
    }
}

pub(crate) fn auth_req_header(auth: ua::NodeId, timeout: u32) -> ua::RequestHeader {
    ua::RequestHeader {
        timestamp: ua::DateTime::now(),
        timeout_hint: timeout,
        authentication_token: auth,
        ..Default::default()
    }
}
