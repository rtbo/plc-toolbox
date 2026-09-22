use open62541_sys::{
    UA_Client, UA_Client_connectAsync, UA_Client_delete, UA_Client_disconnect, UA_Client_getConfig,
    UA_Client_getState, UA_Client_new, UA_Client_run_iterate, UA_ClientConfig_setDefault,
    UA_SessionState,
};
use std::ffi::CString;
use std::ptr;
use tokio::sync::{mpsc, oneshot, watch};

use crate::{StatusCode, error::Error};

mod browse;

#[derive(Debug)]
pub struct Client {
    sender: mpsc::Sender<ClientCommand>,
    state_rx: watch::Receiver<ConnectionState>,
}

impl Client {
    pub async fn connect(endpoint_url: &str) -> Result<Self, Error> {
        let (sender, receiver) = mpsc::channel(32);
        let (state_tx, mut state_rx) = watch::channel(ConnectionState::Disconnected);

        let endpoint_url_c = CString::new(endpoint_url)
            .map_err(|_| Error::from(StatusCode::BadInvalidArgument))?;

        let client_ptr = unsafe {
            let client_ptr = UA_Client_new();
            UA_ClientConfig_setDefault(UA_Client_getConfig(client_ptr));

            let retval: StatusCode = UA_Client_connectAsync(client_ptr, endpoint_url_c.as_ptr()).into();
            retval.check()?;

            client_ptr
        };

        let mut actor = ClientActor {
            client_ptr,
            receiver,
            state_tx,
        };
        tokio::spawn(async move {
            actor.run().await;
        });

        let state = loop {
            state_rx
                .changed()
                .await
                .expect("state receiver should be valid");
            match *state_rx.borrow() {
                ConnectionState::Disconnected | ConnectionState::Connecting => continue,
                state => break state,
            }
        };

        match state {
            ConnectionState::Connected => Ok(Client { sender, state_rx }),
            ConnectionState::Error(status) => Err(Error::from(status)),
            _ => unreachable!(),
        }
    }

    pub fn watch_connection_state(&self) -> watch::Receiver<ConnectionState> {
        self.state_rx.clone()
    }

    pub async fn disconnect(&self) -> Result<(), Error> {
        let (responder, receiver) = oneshot::channel();
        self.sender
            .send(ClientCommand::Disconnect { responder })
            .await
            .map_err(|_| Error::NotConnected)?;
        receiver.await.map_err(|_| Error::NotConnected)?
    }

    pub async fn browse(&self, node_id: impl Into<String>) -> Result<(), Error> {
        let (responder, receiver) = oneshot::channel();
        self.sender
            .send(ClientCommand::Browse {
                node_id: node_id.into(),
                responder,
            })
            .await
            .map_err(|_| Error::NotConnected)?;
        receiver.await.map_err(|_| Error::NotConnected)?
    }
}

impl Drop for Client {
    fn drop(&mut self) {
        if !self.sender.is_closed() {
            let _ = self.sender.send(ClientCommand::Drop);
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConnectionState {
    Disconnected,
    Connecting,
    Connected,
    Error(StatusCode),
}

impl From<(StatusCode, UA_SessionState)> for ConnectionState {
    fn from((status, session_state): (StatusCode, UA_SessionState)) -> Self {
        match (status, session_state) {
            (StatusCode::Good, UA_SessionState::UA_SESSIONSTATE_CLOSED) => {
                ConnectionState::Disconnected
            }
            (StatusCode::Good, UA_SessionState::UA_SESSIONSTATE_ACTIVATED) => {
                ConnectionState::Connected
            }
            (StatusCode::Good, _) => ConnectionState::Connecting,
            (_, _) => ConnectionState::Error(status),
        }
    }
}

enum ClientCommand {
    Browse {
        node_id: String,
        responder: oneshot::Sender<Result<(), Error>>,
    },
    Disconnect {
        responder: oneshot::Sender<Result<(), Error>>,
    },
    Drop,
}

struct ClientActor {
    client_ptr: *mut UA_Client,
    receiver: mpsc::Receiver<ClientCommand>,
    state_tx: watch::Sender<ConnectionState>,
}

unsafe impl Send for ClientActor {}

impl ClientActor {
    async fn run(&mut self) {
        let mut last_state = ConnectionState::Disconnected;
        let mut disconnecting: Option<oneshot::Sender<Result<(), Error>>> = None;

        loop {
            let state = self.state();
            if state != last_state {
                last_state = state;
                let _ = self.state_tx.send(state);
                if matches!(state, ConnectionState::Error(_)) {
                    break;
                }
            }

            if disconnecting.is_some() {
                match state {
                    ConnectionState::Disconnected => {
                        let responder = disconnecting.take().unwrap();
                        let _ = responder.send(Ok(()));
                        break;
                    }
                    ConnectionState::Error(err) => {
                        let responder = disconnecting.take().unwrap();
                        let _ = responder.send(Err(err.into()));
                        break;
                    }
                    _ => {}
                }
            }

            while let Ok(command) = self.receiver.try_recv() {
                match command {
                    ClientCommand::Disconnect { responder } => {
                        disconnecting = Some(responder);
                        if let Err(err) = self.handle_disconnect() {
                            let responder = disconnecting.take().unwrap();
                            let _ = responder.send(Err(err));
                            break;
                        }
                    }
                    ClientCommand::Drop => {
                        break;
                    }
                    _ => {
                        self.handle_command(command);
                    }
                }
            }

            unsafe {
                UA_Client_run_iterate(self.client_ptr, 0);
            }

            tokio::time::sleep(std::time::Duration::from_millis(50)).await;
        }

        unsafe {
            UA_Client_delete(self.client_ptr);
        }
    }

    fn state(&self) -> ConnectionState {
        let mut state = open62541_sys::UA_STATUSCODE_BAD;
        let mut session_state: UA_SessionState = UA_SessionState::UA_SESSIONSTATE_CLOSED;
        unsafe {
            UA_Client_getState(
                self.client_ptr,
                ptr::null_mut(),
                &mut session_state,
                &mut state,
            );
        }
        ConnectionState::from((state.into(), session_state))
    }

    fn handle_command(&mut self, command: ClientCommand) {
        match command {
            ClientCommand::Browse { node_id, responder } => {
                // Implement the browse logic here
                let result = Ok(()); // Placeholder
                let _ = responder.send(result);
            }
            _ => unreachable!(),
        }
    }

    fn handle_disconnect(&mut self) -> Result<(), Error> {
        unsafe {
            let retval: StatusCode = UA_Client_disconnect(self.client_ptr).into();
            retval.check()?;
        }
        Ok(())
    }
}
