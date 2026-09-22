use std::ffi::CString;
use std::ptr;

use tokio::sync::{mpsc, oneshot, watch};

use crate::{StatusCode, ffi, status_code};

mod browse;

#[derive(Debug)]
pub struct Client {
    sender: mpsc::Sender<ClientCommand>,
    state_rx: watch::Receiver<ConnectionState>,
}

impl Client {
    pub fn new() -> Self {
        let (sender, receiver) = mpsc::channel(32);
        let (state_tx, state_rx) = watch::channel(ConnectionState::Disconnected);

        let mut actor = ClientActor::new(receiver, state_tx);

        tokio::spawn(async move {
            actor.run().await;
        });

        Client { sender, state_rx }
    }

    pub async fn connect(&self, url: impl Into<String>) -> status_code::Result<()> {
        self.sender
            .send(ClientCommand::Connect { url: url.into() })
            .await
            .expect("actor should be receptive");

        self.expect_connection_state(ConnectionState::Connected)
            .await
    }

    pub async fn disconnect(&self) -> status_code::Result<()> {
        self.sender
            .send(ClientCommand::Disconnect)
            .await
            .expect("actor should be receptive");

        self.expect_connection_state(ConnectionState::Disconnected)
            .await
    }

    async fn expect_connection_state(&self, expected: ConnectionState) -> status_code::Result<()> {
        let mut state_rx = self.state_rx.clone();
        loop {
            state_rx
                .changed()
                .await
                .expect("state receiver should be valid");

            match *state_rx.borrow() {
                state if state == expected => return Ok(()),
                ConnectionState::Error(status) => return Err(status),
                _ => continue,
            }
        }
    }

    pub fn watch_connection_state(&self) -> watch::Receiver<ConnectionState> {
        self.state_rx.clone()
    }

    pub async fn browse(&self, node_id: impl Into<String>) -> status_code::Result<()> {
        let (responder, receiver) = oneshot::channel();
        self.sender
            .send(ClientCommand::Browse {
                node_id: node_id.into(),
                responder,
            })
            .await
            .expect("actor should be receptive");
        receiver.await.expect("response should be received")?;
        Ok(())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConnectionState {
    Disconnected,
    Connecting,
    Connected,
    Error(StatusCode),
}

impl From<(StatusCode, ffi::UA_SessionState)> for ConnectionState {
    fn from((status, session_state): (StatusCode, ffi::UA_SessionState)) -> Self {
        match (status, session_state) {
            (StatusCode::Good, ffi::UA_SessionState::UA_SESSIONSTATE_CLOSED) => {
                ConnectionState::Disconnected
            }
            (StatusCode::Good, ffi::UA_SessionState::UA_SESSIONSTATE_ACTIVATED) => {
                ConnectionState::Connected
            }
            (StatusCode::Good, _) => ConnectionState::Connecting,
            (_, _) => ConnectionState::Error(status),
        }
    }
}

enum ClientCommand {
    Connect {
        url: String,
    },
    Disconnect,
    Browse {
        node_id: String,
        responder: oneshot::Sender<status_code::Result<()>>,
    },
}

struct ClientActor {
    client_ptr: *mut ffi::UA_Client,
    receiver: mpsc::Receiver<ClientCommand>,
    state_tx: watch::Sender<ConnectionState>,
}

unsafe impl Send for ClientActor {}

impl ClientActor {
    fn new(
        receiver: mpsc::Receiver<ClientCommand>,
        state_tx: watch::Sender<ConnectionState>,
    ) -> Self {
        let client_ptr = unsafe { ffi::UA_Client_new() };
        unsafe {
            ffi::UA_ClientConfig_setDefault(ffi::UA_Client_getConfig(client_ptr));
        }

        Self {
            client_ptr,
            receiver,
            state_tx,
        }
    }
}

impl Drop for ClientActor {
    fn drop(&mut self) {
        unsafe {
            ffi::UA_Client_delete(self.client_ptr);
        }
    }
}

impl ClientActor {
    async fn run(&mut self) {
        let mut last_state = ConnectionState::Disconnected;
        let mut last_sleep = std::time::Instant::now();

        'main: loop {
            let state = self.state();
            if state != last_state {
                last_state = state;
                let _ = self.state_tx.send(state);
                if matches!(state, ConnectionState::Error(_)) {
                    break;
                }
            }

            loop {
                match self.receiver.try_recv() {
                    Ok(command) => self.handle_command(command),
                    Err(mpsc::error::TryRecvError::Empty) => break,
                    Err(mpsc::error::TryRecvError::Disconnected) => break 'main,
                }
            }

            unsafe {
                ffi::UA_Client_run_iterate(self.client_ptr, 0);
            }

            let now = std::time::Instant::now();
            let elapsed = now.duration_since(last_sleep);
            last_sleep = now;
            let sleep_dur = std::time::Duration::from_millis(50).saturating_sub(elapsed);
            tokio::time::sleep(sleep_dur).await;
        }
    }

    fn state(&self) -> ConnectionState {
        let mut state = ffi::UA_STATUSCODE_BAD;
        let mut session_state = ffi::UA_SessionState::UA_SESSIONSTATE_CLOSED;
        unsafe {
            ffi::UA_Client_getState(
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
            ClientCommand::Connect { url } => {
                let _ = self.handle_connect(&url);
            }
            ClientCommand::Browse { node_id, responder } => {
                self.handle_browse(node_id, responder);
            }
            _ => unreachable!(),
        }
    }

    fn handle_connect(&mut self, url: &str) {
        let url_c = CString::new(url).expect("URL should not contain null bytes");

        unsafe {
            let code: StatusCode =
                ffi::UA_Client_connectAsync(self.client_ptr, url_c.as_ptr()).into();
            code.expect_good("UA_Client_connectAsync should return good status");
        }
    }

    fn handle_disconnect(&mut self) {
        unsafe {
            let code: StatusCode = ffi::UA_Client_disconnect(self.client_ptr).into();
            code.expect_good("UA_Client_disconnect should return good status");
        }
    }
}
