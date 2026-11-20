//! A readmitted pending intent's durable prefix, read from the actual World head.
use super::*;
use worth_foundational::facade::{
    DurableCompositePublicationAttemptIdentity, HomeProgramTransitionBodyDigest,
    HomeProgramTransitionPlanDigest, HomeProgramTransitionTargetMeaning,
};
use worth_runtime_world::facade::{ProductBranchObservation, RuntimeWorldOwnerIdentity};

/// How far the original plan is durable. Only the original assignment of the
/// first nondurable step may be admitted next; durable steps never return.
pub(crate) enum DurableHomeTransitionPrefix {
    Complete,
    Next {
        step: u32,
        original: DurableCompositePublicationAttemptIdentity,
    },
}

/// The recomputed plan must equal the recorded one before the prefix is read.
/// The head must be the original chain's occurrence of the last durable step.
pub(crate) fn durable_prefix(
    context: &HomeContext,
    world: RuntimeWorldOwnerIdentity,
    expected: &ProductBranchObservation,
    digest: HomeProgramTransitionPlanDigest,
    steps: u32,
    target: HomeProgramTransitionTargetMeaning,
    bodies: &[HomeProgramTransitionBodyDigest],
    request: &ExecutionRequest<'_, '_>,
) -> Result<DurableHomeTransitionPrefix, ApplicationHomeTransitionIntentDenial> {
    use ApplicationHomeTransitionIntentDenial as Denial;
    request.consult().map_err(Denial::Execution)?;
    super::step_source::require_current_owner(world, expected)?;
    let intent = context.transition.admission.require_resumed_plan(
        &context.descriptor.binding,
        digest,
        steps,
        target,
        bodies,
    )?;
    let durable = expected
        .reference_generation()
        .get()
        .checked_sub(intent.plan().predecessor().generation().get())
        .and_then(|durable| u32::try_from(durable).ok())
        .ok_or(Denial::SourceMismatch)?;
    let prefix = match intent.assignments().get(durable as usize) {
        Some(original) => {
            super::step_source::require_original_source(intent, durable, expected)?;
            DurableHomeTransitionPrefix::Next {
                step: durable,
                original: original.attempt(),
            }
        }
        None => {
            super::step_source::require_completed_source(intent, expected)?;
            DurableHomeTransitionPrefix::Complete
        }
    };
    super::completion::resume(context, durable, expected)?;
    Ok(prefix)
}
