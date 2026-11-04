//! The closed Directory Host installs the actual same-Home Store operation.
use super::runtime::DirectoryHome;
use std::sync::Arc;
use worth_execution::{ExecutionMemoryReservation, ExecutionRequest};
use worth_query_execution::facade::application_home_integration::WorthQueryHomeMaintenance;
use worth_store_application::{
    ApplicationHomeCheckpointDenial as Denial, ApplicationHomeCheckpointLimits,
    ApplicationHomeCheckpointOperation,
};

#[derive(Debug)]
pub(super) struct HomeMaintenance {
    pub(super) operation: Option<ApplicationHomeCheckpointOperation>,
    pub(super) retired_operation:
        Option<worth_store_application::RetiredApplicationHomeCheckpointOperation>,
    _memory: Option<ExecutionMemoryReservation>,
}
impl WorthQueryHomeMaintenance for HomeMaintenance {
    fn preflight_home_retirement(
        &mut self,
        request: &ExecutionRequest<'_, '_>,
    ) -> Result<(), worth_query_execution::facade::primary_graph::WorthQueryHomeRetirementDenial>
    {
        if self
            .operation
            .as_ref()
            .is_some_and(|operation| operation.checkpoint_maintenance_is_active())
        {
            use worth_query_execution::facade::primary_graph::{
                WorthQueryHomeRetirementDenial, WorthQueryHomeRetirementOwner,
            };
            return Err(WorthQueryHomeRetirementDenial::Busy(
                WorthQueryHomeRetirementOwner::HomeMaintenance,
            ));
        }
        request.consult().map_err(
            worth_query_execution::facade::primary_graph::WorthQueryHomeRetirementDenial::Execution,
        )
    }
    fn after_advancement(&self, _: &ExecutionRequest<'_, '_>) {
        // Query retains the exact business value. Actual work/refusal stays in
        // this SAME Store operation's attempt/coverage; there is no error ledger.
        if let Some(operation) = self.operation.as_ref() {
            let _ = operation.automatic_checkpoint_maintenance();
        }
    }
}
pub(super) fn prepare(
    home: &DirectoryHome,
    limits: ApplicationHomeCheckpointLimits,
    request: &ExecutionRequest<'_, '_>,
) -> Result<Arc<dyn WorthQueryHomeMaintenance>, Denial> {
    let operation = home.checkpoint_operation(limits)?;
    let bytes = std::alloc::Layout::new::<(usize, usize)>()
        .extend(std::alloc::Layout::new::<HomeMaintenance>())
        .map_err(|_| Denial::SizeLimit)?
        .0
        .pad_to_align()
        .size();
    let memory = request
        .reserve_retained_memory(bytes as u64)
        .map_err(Denial::Execution)?;
    operation.install_home_checkpoint_maintenance()?;
    Ok(Arc::new(HomeMaintenance {
        operation: Some(operation),
        retired_operation: None,
        _memory: memory,
    }))
}
