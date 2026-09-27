use std::ffi::CString;
use std::ptr;

use super::ConnectionState;
use crate::{StatusCode, client::Responder, ffi, status_code, ua};
use tokio::sync::{mpsc, oneshot, watch};

pub(super) struct ClientActor {
    raw: *mut ffi::UA_Client,
    receiver: mpsc::Receiver<Command>,
    state_tx: watch::Sender<ConnectionState>,
}

unsafe impl Send for ClientActor {}

impl ClientActor {
    pub(super) fn new(
        receiver: mpsc::Receiver<Command>,
        state_tx: watch::Sender<ConnectionState>,
    ) -> Self {
        let raw = unsafe { ffi::UA_Client_new() };
        unsafe {
            ffi::UA_ClientConfig_setDefault(ffi::UA_Client_getConfig(raw));
        }

        Self {
            raw,
            receiver,
            state_tx,
        }
    }

    pub(super) fn raw(&self) -> *mut ffi::UA_Client {
        self.raw
    }
}

pub(super) enum Command {
    Connect {
        url: String,
    },
    Disconnect,
    Browse {
        req: ua::BrowseRequest,
        responder: Responder<ua::BrowseResponse>,
    },
}

impl Drop for ClientActor {
    fn drop(&mut self) {
        unsafe {
            ffi::UA_Client_delete(self.raw);
        }
    }
}

impl ClientActor {
    pub(super) async fn run(&mut self) {
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
                ffi::UA_Client_run_iterate(self.raw, 0);
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
            ffi::UA_Client_getState(self.raw, ptr::null_mut(), &mut session_state, &mut state);
        }
        ConnectionState::from((state.into(), session_state))
    }

    fn handle_command(&mut self, command: Command) {
        match command {
            Command::Connect { url } => {
                let _ = self.handle_connect(&url);
            }
            Command::Disconnect => {
                self.handle_disconnect();
            }
            Command::Browse { req, responder } => {
                self.handle_browse(req, responder);
            }
        }
    }

    fn handle_connect(&mut self, url: &str) {
        let url_c = CString::new(url).expect("URL should not contain null bytes");

        unsafe {
            let code: StatusCode = ffi::UA_Client_connectAsync(self.raw, url_c.as_ptr()).into();
            code.expect_good("UA_Client_connectAsync should return good status");
        }
    }

    fn handle_disconnect(&mut self) {
        unsafe {
            let code: StatusCode = ffi::UA_Client_disconnect(self.raw).into();
            code.expect_good("UA_Client_disconnect should return good status");
        }
    }
}
