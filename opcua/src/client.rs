use tokio::sync::{mpsc, oneshot, watch};

use crate::{StatusCode, ffi, status_code};

mod browse;
mod actor;

use actor::{Command, ClientActor}; 


#[derive(Debug)]
pub struct Client {
    sender: mpsc::Sender<Command>,
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
        self.send_command(Command::Connect { url: url.into() }).await;

        self.expect_connection_state(ConnectionState::Connected)
            .await
    }

    pub async fn disconnect(&self) -> status_code::Result<()> {
        self.send_command(Command::Disconnect).await;

        self.expect_connection_state(ConnectionState::Disconnected)
            .await
    }

    async fn send_command(&self, command: Command) {
        self.sender.send(command).await.expect("actor should be receptive");
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
            .send(Command::Browse {
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

