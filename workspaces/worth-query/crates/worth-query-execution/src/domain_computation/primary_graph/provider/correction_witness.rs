//! Minimal performed/canonical witness custody under its original request.
use crate::domain_computation::application_aftermath::WorthQueryCommittedAftermathCausality;
use worth_execution::{
    ExecutionMemoryReservation, ExecutionRequest, MapKernelContext, MapKernelStop,
    WorkCeilingDenial,
};
use worth_runtime_world::facade::CompositeCommitIdentity;

pub(in crate::domain_computation::primary_graph) struct CorrectionPerformedWitness {
    pub world: CompositeCommitIdentity,
    pub aftermath: Option<WorthQueryCommittedAftermathCausality>,
    // Last: copied causality and the Arc allocation drop before their payer.
    pub(super) _memory: Option<ExecutionMemoryReservation>,
}

#[derive(Debug)]
pub(in crate::domain_computation::primary_graph) enum CorrectionWitnessDenial {
    Request(WorkCeilingDenial),
    CopyAdmission(WorkCeilingDenial),
    Interrupted(MapKernelStop),
    SizeOverflow,
    Unavailable,
    Basis(worth_relational::facade::snapshots::RelationalSnapshotAdmissionDenial),
}

pub(super) fn run<T>(
    execution: ExecutionRequest<'_, '_>,
    operation: impl FnOnce(&mut MapKernelContext<'_, '_>) -> Result<T, CorrectionWitnessDenial>,
) -> Result<T, CorrectionWitnessDenial> {
    let mut result = None;
    let outcome = execution.run_native_with_checkpoint(|context| {
        result = Some(operation(context));
    });
    match (outcome, result) {
        (Ok(_), Some(result)) => result,
        (Err(WorkCeilingDenial::Stopped(MapKernelStop::NestedStopped)), Some(Err(cause))) => {
            Err(cause)
        }
        (Err(cause), _) => Err(CorrectionWitnessDenial::Request(cause)),
        (Ok(_), None) => unreachable!("completed correction witness callback"),
    }
}

pub(super) fn admit(
    context: &mut MapKernelContext<'_, '_>,
    work: u64,
) -> Result<(), CorrectionWitnessDenial> {
    if context.has_accounting() {
        context
            .checkpoint(work)
            .map_err(CorrectionWitnessDenial::Interrupted)
    } else {
        context.poll().map_err(CorrectionWitnessDenial::Interrupted)
    }
}

#[cfg(test)]
mod tests;
