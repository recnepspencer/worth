//! Minimal performed/canonical witness custody under its original request.
use crate::domain_computation::application_aftermath::WorthQueryCommittedAftermathCausality;
use std::{alloc::Layout, sync::Arc};
use worth_execution::{
    ExecutionMemoryReservation, ExecutionRequest, MapKernelContext, MapKernelStop,
    WorkCeilingDenial,
};
use worth_runtime_world::facade::CompositeCommitIdentity;

pub struct CorrectionPerformedWitness {
    pub world: CompositeCommitIdentity,
    pub aftermath: Option<WorthQueryCommittedAftermathCausality>,
    // Last: copied causality and the Arc allocation drop before their payer.
    pub(super) _memory: Option<ExecutionMemoryReservation>,
}

#[derive(Debug)]
pub enum CorrectionWitnessDenial {
    Request(WorkCeilingDenial),
    CopyAdmission(WorkCeilingDenial),
    Interrupted(MapKernelStop),
    SizeOverflow,
    Unavailable,
    Basis(worth_relational::facade::snapshots::RelationalSnapshotAdmissionDenial),
    Execution(worth_relational::facade::transactions::RelationalExecutionDenialCause),
    /// A durable Home's original native effect could not be discovered or read.
    OriginalSource(Box<worth_relational::facade::durability::RelationalAssociatedEffectRefusal>),
    /// A durable Home's World original history refused the native effect.
    OriginalHistory(worth_runtime_world::facade::WorldReadmissionDenial),
    /// A canonical fact disagrees with the occurrence that carries it.
    InconsistentCausality,
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

/// Copies one witness under its request: the Arc root and every owned receipt
/// byte are reserved before the copy, on both the session and restored paths.
pub(super) fn retain(
    execution: ExecutionRequest<'_, '_>,
    context: &mut MapKernelContext<'_, '_>,
    world: &CompositeCommitIdentity,
    causality: Option<&WorthQueryCommittedAftermathCausality>,
) -> Result<Arc<CorrectionPerformedWitness>, CorrectionWitnessDenial> {
    use CorrectionWitnessDenial as Denial;
    let backing = |causality: Option<&WorthQueryCommittedAftermathCausality>| match causality {
        Some(causality) => causality
            .owned_receipt_backing_bytes()
            .ok_or(Denial::SizeOverflow),
        None => Ok(0),
    };
    let root = Layout::new::<(usize, usize)>()
        .extend(Layout::new::<CorrectionPerformedWitness>())
        .map_err(|_| Denial::SizeOverflow)?
        .0
        .pad_to_align()
        .size() as u64;
    let bytes = root
        .checked_add(backing(causality)?)
        .ok_or(Denial::SizeOverflow)?;
    admit(context, bytes)?;
    let mut memory = execution
        .reserve_retained_memory(bytes)
        .map_err(Denial::CopyAdmission)?;
    let aftermath = causality.cloned();
    let actual = root
        .checked_add(backing(aftermath.as_ref())?)
        .ok_or(Denial::SizeOverflow)?;
    // These fixed String/Vec<CommitId> clones copy their initialized
    // payload, covered by the source's admitted capacities above.
    if actual > bytes {
        return Err(Denial::SizeOverflow);
    }
    if let Some(memory) = &mut memory {
        memory.resize(actual).map_err(|cause| {
            Denial::Request(WorkCeilingDenial::Admission(
                worth_execution::LeaseDenial::MemoryExhausted(cause),
            ))
        })?;
    }
    Ok(Arc::new(CorrectionPerformedWitness {
        world: world.clone(),
        aftermath,
        _memory: memory,
    }))
}

pub mod court_doors_for_test;
