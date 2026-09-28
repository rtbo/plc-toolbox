use super::{BrowseResponse, BrowseResult};

impl BrowseResponse {
    pub fn results(&self) -> &[BrowseResult] {
        unsafe {
            std::slice::from_raw_parts(self.raw.results.cast(), self.raw.resultsSize)
        }
    }

    pub fn results_mut(&mut self) -> &mut [BrowseResult] {
        unsafe {
            std::slice::from_raw_parts_mut(self.raw.results.cast(), self.raw.resultsSize)
        }
    }
}
