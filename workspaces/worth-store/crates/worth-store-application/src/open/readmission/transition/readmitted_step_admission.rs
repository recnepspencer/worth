//! The actual protected original intent admits a current recovered World claim.
use super::*;
use worth_foundational::facade::{
    HomeProgramTransitionPlanDigest, HomeProgramTransitionStepAssociation,
    HomeProgramTransitionTargetMeaning,
};
use worth_runtime_world::facade::{RuntimeWorldOwnerIdentity, WorldRecoveredUnpublishedEffects};

pub(crate) fn admit(
    context: &HomeContext,
    world: RuntimeWorldOwnerIdentity,
    step: u32,
    claim: &WorldRecoveredUnpublishedEffects,
    digest: HomeProgramTransitionPlanDigest,
    steps: u32,
    target: HomeProgramTransitionTargetMeaning,
    bodies: &[worth_foundational::facade::HomeProgramTransitionBodyDigest],
    request: &ExecutionRequest<'_, '_>,
) -> Result<AdmittedApplicationHomeProgramStep, ApplicationHomeTransitionIntentDenial> {
    use ApplicationHomeTransitionIntentDenial as Denial;
    request.consult().map_err(Denial::Execution)?;
    let presented = claim.attempt_identity().owner_identity();
    world
        .admit_incarnation(presented)
        .map_err(|cause| match cause {
            worth_proof::HomeIncarnationDenial::StaleIncarnation => Denial::StaleIncarnation,
            worth_proof::HomeIncarnationDenial::ForeignHome => Denial::ForeignWorld,
        })?;
    if world != presented
        || world.get() != presented.get()
        || claim.expected_head().owner_identity() != world
        || claim.expected_head().owner_identity().get() != world.get()
        || claim.claim_incarnation() != context.descriptor.incarnation
    {
        return Err(Denial::ForeignWorld);
    }
    let intent = context.transition.admission.require_resumed_plan(
        &context.descriptor.binding,
        digest,
        steps,
        target,
        bodies,
    )?;
    let original = intent
        .assignments()
        .get(step as usize)
        .ok_or(Denial::AssignmentCount)?;
    let expected = claim.expected_head();
    if claim.attempt_identity().ordinal() != original.attempt().ordinal() {
        return Err(Denial::SourceMismatch);
    }
    super::step_source::require_original_source(intent, step, expected)?;
    let step = HomeProgramTransitionStepAssociation::from_recorded(intent.plan().identity(), step);
    let commitment = intent
        .plan()
        .step_commitment(step, original)
        .map_err(|_| Denial::SourceMismatch)?;
    let prepared = super::step_admission::PreparedStepPermission::prepare(
        intent,
        step.step(),
        original,
        request,
    )?;
    let association = claim
        .caller_association_commitment(request, intent.plan(), step)
        .map_err(Denial::WorldAssociation)?;
    Ok(prepared.issue(
        context,
        commitment,
        expected
            .basis()
            .relational_basis()
            .admission_identity()
            .clone(),
        association,
        None,
    ))
}
