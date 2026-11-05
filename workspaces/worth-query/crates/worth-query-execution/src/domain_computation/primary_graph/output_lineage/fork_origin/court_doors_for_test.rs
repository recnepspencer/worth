//! Sealed test doors over this module's private members.
use super::*;

impl WorthQueryApplicationOutputLineage {
    #[doc(hidden)]
    pub fn register_fork_for_test(
        &mut self,
        source: &worth_runtime_world::facade::ProductBranchObservation,
        destination: &worth_runtime_world::facade::ProductBranchObservation,
    ) {
        Self::register_fork(self, source, destination)
    }
}
