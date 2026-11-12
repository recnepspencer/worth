//! Continue an actual original Native step through the current live receiver.
use super::*;
use crate::publication::{
    WorldHomeTransitionEffectDenial as Denial, WorldHomeTransitionEffectOutcome as Outcome,
};

impl<D, I, E, Ctx, T> RuntimeWorldPublicationPort<D, I, E, Ctx, T>
where
    D: Copy + Ord + std::fmt::Debug + Send + Sync + 'static,
    I: Copy + Ord + Send + Sync + 'static,
    E: Send + Sync + 'static,
    Ctx: Send + Sync + 'static,
    T: Copy + Ord + Send + Sync + 'static,
{
    pub fn continue_home_transition_effect(
        &self,
        request: &worth_execution::ExecutionRequest<'_, '_>,
        plan: &worth_foundational::facade::HomeProgramTransitionPlanDescription,
        step: worth_foundational::facade::HomeProgramTransitionStepAssociation,
        original: &worth_foundational::facade::CompositeAttemptAssociation,
        admission: &worth_proof::AdmittedHomeProgramTransitionStep<
            worth_relational::facade::branch::RelationalBranchBasisAdmissionIdentity,
        >,
        claim: crate::recovery::WorldRecoveredUnpublishedEffects,
        cancellation: &RuntimeWorldCancellationToken,
        deadline: Option<RuntimeWorldInstant>,
    ) -> Outcome {
        match self.service() {
            Ok(owner) => owner.continue_home_transition_effect(
                request,
                plan,
                step,
                original,
                admission,
                claim,
                cancellation,
                deadline,
            ),
            Err(_) => Outcome::Refused {
                cause: Denial::Source(NoEffectCause::OwnerUnavailable),
                custody: claim,
            },
        }
    }
}
