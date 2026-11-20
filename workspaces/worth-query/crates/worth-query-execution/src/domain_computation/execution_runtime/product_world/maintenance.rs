//! Directory composition installs its SAME Home operation before service.
use std::any::Any;
use std::sync::Arc;
use worth_execution::ExecutionRequest;
use worth_query_execution_runtime::facade::ports::HomeMaintenanceAfterAdvancement;

/// The closed Host implementation owns the actual Store maintenance operation.
/// It retains no Home lifecycle owner. Automatic upkeep uses the Host's own
/// allowance; the invoking request is context, never its execution funding.
/// Failures remain actual Home attempt/coverage custody; they cannot replace a
/// completed business result or become a process-local last-error ledger.
pub trait WorthQueryHomeMaintenance: std::any::Any + std::fmt::Debug + Send + Sync {
    fn after_advancement(&self, request: &ExecutionRequest<'_, '_>);
    /// The closed Host must provide the consuming retirement of its actual
    /// operation. Unknown maintenance adapters cannot certify that release.
    fn preflight_home_retirement(
        &mut self,
        _: &ExecutionRequest<'_, '_>,
    ) -> Result<(), crate::domain_computation::primary_graph::WorthQueryHomeRetirementDenial> {
        Err(
            crate::domain_computation::primary_graph::WorthQueryHomeRetirementDenial::Unsupported(
                crate::domain_computation::primary_graph::WorthQueryHomeRetirementOwner::HomeMaintenance,
            ),
        )
    }
}

/// Bridges `WorthQueryHomeMaintenance` to the lower runtime maintenance port.
#[derive(Debug)]
pub struct HomeMaintenanceAdapter(pub(crate) Arc<dyn WorthQueryHomeMaintenance>);

impl HomeMaintenanceAdapter {
    pub(crate) fn wrap(maintenance: Arc<dyn WorthQueryHomeMaintenance>) -> Arc<dyn HomeMaintenanceAfterAdvancement> {
        Arc::new(Self(maintenance))
    }

    pub(crate) fn inner_mut(&mut self) -> &mut Arc<dyn WorthQueryHomeMaintenance> {
        &mut self.0
    }

    pub(crate) fn into_inner(self) -> Arc<dyn WorthQueryHomeMaintenance> {
        self.0
    }
}

impl HomeMaintenanceAfterAdvancement for HomeMaintenanceAdapter {
    fn after_advancement(&self, request: &ExecutionRequest<'_, '_>) {
        self.0.after_advancement(request);
    }

    fn as_any(&self) -> &(dyn Any + Send + Sync) {
        self
    }

    fn as_any_mut(&mut self) -> &mut (dyn Any + Send + Sync) {
        self
    }
}
