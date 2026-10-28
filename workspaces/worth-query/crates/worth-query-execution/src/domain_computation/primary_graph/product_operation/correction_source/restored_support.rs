//! Correction-only restoration: complete authored overlap, exact cancellation,
//! then actual retained original-root support. Ordinary reuse stays unchanged.
mod cancellation;
mod eligibility;
mod overlap;
pub(super) use eligibility::require;

#[derive(Debug)]
/// Why the selected correction's restored support cannot be established.
/// Unsupported facts and unavailable canonical evidence never mean no writes.
pub enum WorthQueryCorrectionRestoredSupportDenial {
    UnsupportedFact,
    CanonicalEvidenceUnavailable,
    CanonicalAssociationMismatch,
    Native(worth_relational::facade::history::RelationalAuthoredWriteDenial),
    Selection(worth_relational::facade::change_source::RelationalCommitSelectionDenial),
    FieldSupport(worth_relational::facade::runtime::RelationalFieldSupportComparisonDenial),
    AspectPresence(worth_relational::facade::runtime::RelationalAspectPresenceSupportDenial),
    Allocation(worth_execution::ExecutionAllocationDenial),
    Storage(worth_execution::ExecutionOrderedMapDenial),
    Request(worth_execution::WorkCeilingDenial),
    Interrupted(worth_execution::MapKernelStop),
    Execution(worth_relational::facade::transactions::RelationalExecutionDenialCause),
    SizeOverflow,
}

// Ordered keys are fixed CommitIds here. Admit the immutable AVL descent before
// each new map operation; canonical/World discovery keeps its own work owner.
fn ordered_visit(
    execution: worth_execution::ExecutionRequest<'_, '_>,
    entries: usize,
) -> Result<(), WorthQueryCorrectionRestoredSupportDenial> {
    use WorthQueryCorrectionRestoredSupportDenial as Stop;
    let height = 2 * u64::from(usize::BITS - entries.leading_zeros()) + 2;
    let work = height
        .checked_mul(std::mem::size_of::<worth_relational::facade::history::CommitId>() as u64)
        .ok_or(Stop::Interrupted(
            worth_execution::MapKernelStop::WorkCounterOverflow,
        ))?;
    admit_work(execution, work)
}

fn admit_work(
    execution: worth_execution::ExecutionRequest<'_, '_>,
    work: u64,
) -> Result<(), WorthQueryCorrectionRestoredSupportDenial> {
    use worth_execution::{MapKernelStop, WorkCeilingDenial};
    use WorthQueryCorrectionRestoredSupportDenial as Stop;
    let mut held = None;
    let outcome = execution.run_native_with_checkpoint(|context| {
        held = Some(if context.has_accounting() {
            context.checkpoint(work)
        } else {
            context.poll()
        });
    });
    match (outcome, held) {
        (Ok(_), Some(result)) => result.map_err(Stop::Interrupted),
        (Err(WorkCeilingDenial::Stopped(MapKernelStop::NestedStopped)), Some(Err(cause))) => {
            Err(Stop::Interrupted(cause))
        }
        (Err(cause), _) => Err(Stop::Request(cause)),
        (Ok(_), None) => unreachable!("completed correction work admission"),
    }
}
