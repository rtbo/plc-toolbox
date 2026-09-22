use std::ffi::c_void;
use std::mem;

use tokio::sync::oneshot;

use crate::node_id::NodeId;
use crate::{ffi, status_code};

impl super::ClientActor {
    pub fn handle_browse(
        &mut self,
        node_id: String,
        responder: oneshot::Sender<status_code::Result<()>>,
    ) {
        let node_id: NodeId = match node_id.parse() {
            Ok(n) => n,
            Err(e) => {
                let _ = responder.send(Err(e));
                return;
            }
        };

        extern "C" fn browse_callback(
            _client: *mut ffi::UA_Client,
            userdata: *mut ::core::ffi::c_void,
            _request_id: ffi::UA_UInt32,
            wr: *mut ffi::UA_BrowseResponse,
        ) {
            println!("Browse callback invoked: {} references", unsafe {
                (*wr).resultsSize
            });
            // Convert the userdata back into a oneshot::Sender
            let responder: Box<oneshot::Sender<status_code::Result<()>>> =
                unsafe { Box::from_raw(userdata as *mut _) };

            let _ = responder.send(Ok(()));
        }

        let mut browse_desc: ffi::UA_BrowseDescription = unsafe { mem::zeroed() };

        browse_desc.nodeId = unsafe { node_id.as_ffi() };
        browse_desc.referenceTypeId =
            unsafe { ffi::UA_NODEID_NUMERIC(0, ffi::UA_NS0ID_HIERARCHICALREFERENCES) };
        browse_desc.includeSubtypes = true;
        browse_desc.resultMask = ffi::UA_BrowseResultMask::UA_BROWSERESULTMASK_ALL.0;

        let mut req: ffi::UA_BrowseRequest = unsafe { mem::zeroed() };
        req.nodesToBrowse = &mut browse_desc;
        req.nodesToBrowseSize = 1;

        let responder = Box::into_raw(Box::new(responder));

        unsafe {
            ffi::UA_Client_sendAsyncBrowseRequest(
                self.client_ptr,
                &mut req,
                Some(browse_callback),
                responder as *mut c_void,
                std::ptr::null_mut(),
            );
        }
    }
}
