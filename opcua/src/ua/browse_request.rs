use crate::{Array, ua::BrowseDescription};

impl super::BrowseRequest {
    pub fn with_nodes_to_browse(mut self, nodes: Array<BrowseDescription>) -> Self {
        // Safety: We are taking ownership of the existing raw array and converting it into an Array
        // in otder to properly manage its memory.

        let _old = unsafe {
            Array::<BrowseDescription>::from_raw_parts(self.raw.nodesToBrowse, self.raw.nodesToBrowseSize)
        };
        self.raw.nodesToBrowse = nodes.as_mut_ptr();
        self.raw.nodesToBrowseSize = nodes.len();
        std::mem::forget(nodes);
        self
    }
}
