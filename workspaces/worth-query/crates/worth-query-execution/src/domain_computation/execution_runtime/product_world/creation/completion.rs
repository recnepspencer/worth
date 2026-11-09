//! The original Query reservation lives with World's original sibling custody.
use super::*;
use crate::domain_computation::execution_runtime::product_world::activation::registry::WorthQueryProductActivationReservation;
pub(super) struct QueryBranchCreationCompletion {
    reservation: WorthQueryProductActivationReservation,
    lineage_completed: bool,
    lineage: Option<std::sync::Weak<std::sync::Mutex<crate::domain_computation::primary_graph::output_lineage::WorthQueryApplicationOutputLineage>>>,
    _funding: Option<worth_execution::ExecutionMemoryReservation>,
}
impl QueryBranchCreationCompletion {
    pub(super) fn new(
        request: &worth_execution::ExecutionRequest<'_, '_>,
        reservation: WorthQueryProductActivationReservation,
        lineage: Option<std::sync::Arc<std::sync::Mutex<crate::domain_computation::primary_graph::output_lineage::WorthQueryApplicationOutputLineage>>>,
    ) -> Result<Self, WorthQueryProductBranchCreationDenial> {
        let funding = request
            .reserve_retained_memory(std::mem::size_of::<Self>() as u64)
            .map_err(|_| WorthQueryProductBranchCreationDenial::CoordinationCapacityExhausted)?;
        Ok(Self {
            reservation,
            lineage_completed: false,
            lineage: lineage.as_ref().map(std::sync::Arc::downgrade),
            _funding: funding,
        })
    }
}
impl std::fmt::Debug for QueryBranchCreationCompletion {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("QueryBranchCreationCompletion")
            .finish_non_exhaustive()
    }
}
impl worth_runtime_world::facade::RuntimeWorldBranchCreationCompletion
    for QueryBranchCreationCompletion
{
    fn completed(
        &mut self,
        source: &worth_runtime_world::facade::ProductBranchObservation,
        destination: &worth_runtime_world::facade::ProductBranchObservation,
    ) {
        self.reservation.commit_retained(destination);
        if self.lineage_completed {
            return;
        }
        if let Some(lineage) = self.lineage.as_ref().and_then(std::sync::Weak::upgrade) {
            lineage
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner)
                .register_fork(source, destination);
        }
        self.lineage_completed = true;
    }
}
