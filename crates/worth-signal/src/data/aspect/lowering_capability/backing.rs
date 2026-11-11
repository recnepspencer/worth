//! The node capability owner measures the complete contract it will clone.
use super::{InstalledSignalNodeCapability, NodeId, SignalGraph};
use crate::data::error::SignalError;
use crate::data::retained_storage::{
    arc_allocation_charge, RetainedStorageMeasurement, RetainedStoragePreparation,
    RetainedStoragePreparationDenial as D,
};
use worth_execution::ExecutionRequest;

impl SignalGraph {
    /// Backing bound for an Arc-held installed node capability, including its
    /// complete cloned contract. This report grants no capability or retention.
    pub fn installed_node_capability_backing_bytes(
        &self,
        node: NodeId,
        execution: &ExecutionRequest<'_, '_>,
    ) -> Result<u64, SignalError> {
        let contract = self.get_contract(node)?;
        let mut measured = None;
        let outcome = execution.run_native_with_checkpoint(|context| {
            let mut work = RetainedStoragePreparation::new(None);
            let mut checkpoint = |units: usize| {
                let units = u64::try_from(units).map_err(|_| D::ChargeOverflow)?;
                let result = if context.has_accounting() {
                    context.checkpoint(units)
                } else {
                    context.poll()
                };
                result.map_err(|cause| D::ExecutionStopped(cause.into()))
            };
            let mut observed = work.reborrow_with_checkpoint(&mut checkpoint);
            let mut report = || {
                arc_allocation_charge::<InstalledSignalNodeCapability>()?
                    .checked_add(contract.retained_heap_charge(&mut observed)?)
            };
            measured = Some(
                report()
                    .map(|charge| charge.bytes())
                    .map_err(backing_denied),
            );
        });
        match (outcome, measured) {
            (
                Err(worth_execution::WorkCeilingDenial::Stopped(
                    worth_execution::MapKernelStop::NestedStopped,
                )),
                Some(Err(cause)),
            ) => Err(cause),
            (Err(cause), _) => Err(SignalError::execution_scope_denied(cause)),
            (Ok(_), Some(result)) => result,
            (Ok(_), None) => {
                unreachable!("completed capability report retains its measured result")
            }
        }
    }
}
fn backing_denied(cause: D) -> SignalError {
    match cause {
        D::ExecutionStopped(cause) => SignalError::ExecutionCheckpointStopped(cause),
        D::WorkExhausted { maximum_visits } => {
            SignalError::ConditionalEvaluationWorkExhausted { maximum_visits }
        }
        D::ChargeOverflow => SignalError::RetainedStorageChargeOverflow,
        D::ChargeUnderflow => SignalError::RetainedStorageChargeUnderflow,
        D::RetainedExtentHistoryUnavailable => SignalError::RetainedStorageHistoryUnavailable,
    }
}
