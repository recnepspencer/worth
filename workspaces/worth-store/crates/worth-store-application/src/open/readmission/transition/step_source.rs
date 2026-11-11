//! The original ordered World source of a Home intent step.
use super::{ApplicationHomeTransitionIntentDenial as Denial, HomeProgramTransitionIntent};
use worth_foundational::facade::CompositeAttemptAssociation;
use worth_runtime_world::facade::{ProductBranchObservation, RuntimeWorldOwnerIdentity};

pub(super) fn require_current_owner(
    world: RuntimeWorldOwnerIdentity,
    expected: &ProductBranchObservation,
) -> Result<(), Denial> {
    let presented = expected.owner_identity();
    world
        .admit_incarnation(presented)
        .map_err(|cause| match cause {
            worth_proof::HomeIncarnationDenial::StaleIncarnation => Denial::StaleIncarnation,
            worth_proof::HomeIncarnationDenial::ForeignHome => Denial::ForeignWorld,
        })?;
    if world != presented || world.get() != presented.get() {
        return Err(Denial::ForeignWorld);
    }
    Ok(())
}

pub(super) fn require_original_source<'intent>(
    intent: &'intent HomeProgramTransitionIntent,
    step: u32,
    expected: &ProductBranchObservation,
) -> Result<&'intent CompositeAttemptAssociation, Denial> {
    let original = intent
        .assignments()
        .get(step as usize)
        .ok_or(Denial::AssignmentCount)?;
    let predecessor = intent.plan().predecessor();
    if expected.owner_identity().durable() != Some(original.attempt().owner())
        || expected.branch_identity().name().as_str() != original.branch().name()
        || expected.lifecycle_incarnation().ordinal() != original.incarnation().ordinal()
        || predecessor.generation().get().checked_add(u64::from(step))
            != Some(expected.reference_generation().get())
    {
        return Err(Denial::SourceMismatch);
    }
    if step == 0 {
        if expected.selected_commit().ordinal() != predecessor.selected_commit().ordinal() {
            return Err(Denial::SourceMismatch);
        }
    } else {
        let prior = &intent.assignments()[step as usize - 1];
        match expected.publication_attempt() {
            Some(attempt)
                if attempt.ordinal() == prior.attempt().ordinal()
                    && attempt.owner_identity().durable() == Some(prior.attempt().owner()) => {}
            Some(_) | None => return Err(Denial::SourceMismatch),
        }
    }
    Ok(original)
}
