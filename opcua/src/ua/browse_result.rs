use super::{BrowseResult, ReferenceDescription};
use crate::StatusCode;

impl BrowseResult {
    pub fn status_code(&self) -> StatusCode {
        unsafe {
            StatusCode::from_raw_unchecked(self.raw.statusCode)
        }
    }

    pub fn continuation_point(&self) -> &[u8] {
        let cp = &self.raw.continuationPoint;
        unsafe {
            std::slice::from_raw_parts(cp.data, cp.length)
        }
    }

    pub fn references(&self) -> &[ReferenceDescription] {
        unsafe {
            std::slice::from_raw_parts(self.raw.references.cast(), self.raw.referencesSize)
        }
    }


}
