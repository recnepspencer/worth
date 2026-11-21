//! Batch reads share the original native snapshot and its release lifecycle.
use super::*;
impl WorthQueryApplicationBasisLease {
    pub fn retain_in_batch(
        &mut self,
    ) -> Result<Self, CompanionPreflightStop> {
        self.prepare_shared()?;
        self.retain_shared()
    }
}
