//! Compiled native scheduling door; the returned guard belongs to this provider.
use super::super::WorthQueryPrimaryGraphApplicationRuntime;
use crate::domain_computation::primary_graph::provider::managed_cleanup_control::{
    WorthQueryManagedCleanupRegistrationDenial, WorthQueryManagedCleanupScheduleForTest,
};
use worth_query_installation::facade::ApplicationSchema;
impl<S: ApplicationSchema> WorthQueryPrimaryGraphApplicationRuntime<S> {
    /// Schedules one actual managed terminal cleanup on a foreign worker.
    /// The native Signal owner returns its genuine finalization failure and custody.
    pub fn schedule_next_managed_cleanup_on_worker_for_test(
        &self,
    ) -> Result<WorthQueryManagedCleanupScheduleForTest, WorthQueryManagedCleanupRegistrationDenial>
    {
        self.primary_provider.managed_cleanup_control.arm()
    }
}
