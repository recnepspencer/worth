//! An arming guard for the installed provider's managed cleanup boundary.
use super::*;
pub(in crate::domain_computation::primary_graph) struct ManagedCleanupFaultForTest(
    std::sync::Arc<PrimaryGraphFaultController>,
);
impl PrimaryGraphFaultController {
    pub(in crate::domain_computation::primary_graph) fn reject_managed_cleanup_for_test(
        self: &std::sync::Arc<Self>,
    ) -> ManagedCleanupFaultForTest {
        self.schedule(WorthQueryPrimaryGraphFault::RejectedManagedCleanup);
        ManagedCleanupFaultForTest(std::sync::Arc::clone(self))
    }
}
impl Drop for ManagedCleanupFaultForTest {
    fn drop(&mut self) {
        self.0
            .take(WorthQueryPrimaryGraphFault::RejectedManagedCleanup);
    }
}
