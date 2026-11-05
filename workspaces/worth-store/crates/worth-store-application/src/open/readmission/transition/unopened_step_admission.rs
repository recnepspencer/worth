//! Claim an original intent assignment whose World open still has to be admitted.
use super::*;
use worth_foundational::facade::{
    HomeProgramTransitionPlanDigest, HomeProgramTransitionStepAssociation,
    HomeProgramTransitionTargetMeaning,
};
use worth_runtime_world::facade::{ProductBranchObservation, RuntimeWorldOwnerIdentity};

pub(crate) fn admit(
    context: &HomeContext,
    world: RuntimeWorldOwnerIdentity,
    step: u32,
    expected: &ProductBranchObservation,
    digest: HomeProgramTransitionPlanDigest,
    steps: u32,
    target: HomeProgramTransitionTargetMeaning,
    bodies: &[worth_foundational::facade::HomeProgramTransitionBodyDigest],
    request: &ExecutionRequest<'_, '_>,
) -> Result<AdmittedApplicationHomeProgramStep, ApplicationHomeTransitionIntentDenial> {
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
    let original = super::step_source::require_original_source(intent, step, expected)?;
    let described =
        HomeProgramTransitionStepAssociation::from_recorded(intent.plan().identity(), step);
    let association = worth_runtime_world::facade::home_transition_caller_association_commitment(
        request,
        intent.plan(),
        described,
        original,
    )
    .map_err(Denial::WorldAssociation)?;
    let commitment = intent
        .plan()
        .step_commitment(described, original)
        .map_err(|_| Denial::SourceMismatch)?;
    let prepared =
        super::step_admission::PreparedStepPermission::prepare(intent, step, original, request)?;
    let execution = context.transition.claim_resumed_execution(
        step,
        steps,
        commitment,
        association,
        request,
    )?;
    Ok(prepared
        .issue(
            context,
            commitment,
            expected
                .basis()
                .relational_basis()
                .admission_identity()
                .clone(),
            association,
            None,
        )
        .with_execution(execution))
}
