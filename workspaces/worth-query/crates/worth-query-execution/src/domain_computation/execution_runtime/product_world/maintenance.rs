//! Directory composition installs its SAME Home operation before service.
use worth_execution::ExecutionRequest;

/// The closed Host implementation owns the actual Store maintenance operation.
/// It retains no Home lifecycle owner and must use the original invoking meter.
/// Failures remain actual Home attempt/coverage custody; they cannot replace a
/// completed business result or become a process-local last-error ledger.
pub trait WorthQueryHomeMaintenance: std::fmt::Debug + Send + Sync {
    fn after_advancement(&self, request: &ExecutionRequest<'_, '_>);
}
impl super::WorthQueryProductRuntime {
    pub(in crate::domain_computation) fn home_maintenance(
        &self,
    ) -> Option<&dyn WorthQueryHomeMaintenance> {
        self.home_maintenance.as_deref()
    }
}
