use std::ffi::c_void;

use tokio::sync::oneshot;

use crate::{DataType, ua};
use crate::{client, ffi, status_code};

pub(super) type Responder = oneshot::Sender<status_code::Result<ua::BrowseResponse>>;

impl client::ClientActor {
    pub fn handle_browse(&mut self, node_id: String, responder: Responder) {
        let node_id: ua::NodeId = match node_id.parse() {
            Ok(n) => n,
            Err(e) => {
                let _ = responder.send(Err(e));
                return;
            }
        };

        let mut browse_desc = ffi::UA_BrowseDescription::default();

        browse_desc.nodeId = node_id.into_raw();
        browse_desc.referenceTypeId =
            unsafe { ffi::UA_NODEID_NUMERIC(0, ffi::UA_NS0ID_HIERARCHICALREFERENCES) };
        browse_desc.includeSubtypes = true;
        browse_desc.resultMask = ffi::UA_BrowseResultMask::UA_BROWSERESULTMASK_ALL.0;

        let mut req = ffi::UA_BrowseRequest::default();
        req.nodesToBrowse = &mut browse_desc;
        req.nodesToBrowseSize = 1;

        let responder = Box::into_raw(Box::new(responder));

        unsafe {
            ffi::UA_Client_sendAsyncBrowseRequest(
                self.raw(),
                &mut req,
                Some(browse_callback),
                responder as *mut c_void,
                std::ptr::null_mut(),
            );
        }
    }
}

extern "C" fn browse_callback(
    _client: *mut ffi::UA_Client,
    userdata: *mut c_void,
    _request_id: ffi::UA_UInt32,
    wr: *mut ffi::UA_BrowseResponse,
) {
    // Convert the userdata back into a oneshot::Sender
    let responder: Box<Responder> = unsafe { Box::from_raw(userdata as *mut _) };

    // SAFETY: wr is a pointer to a valid UA_BrowseResponse structure.
    // wr ownership is transferred to the BrowseResponse
    let response = unsafe { ua::BrowseResponse::from_raw(std::ptr::read(wr)) };

    let _ = responder.send(Ok(response));
}
