//! The live receiver consumes a genuine current-Home recovered open claim.
use super::*;
use crate::publication::{
    WorldHomeTransitionOpenResumeDenial as Denial, WorldHomeTransitionOpenResumeRefused as Refused,
    WorldOpenedPublicationAttempt,
};

impl<D, I, E, Ctx, T> RuntimeWorldPublicationPort<D, I, E, Ctx, T>
where
    D: Copy + Ord + std::fmt::Debug + Send + Sync + 'static,
    I: Copy + Ord + Send + Sync + 'static,
    E: Send + Sync + 'static,
    Ctx: Send + Sync + 'static,
    T: Copy + Ord + Send + Sync + 'static,
{
    pub fn resume_home_transition_open(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        plan: &worth_foundational::facade::HomeProgramTransitionPlanDescription,
        step: worth_foundational::facade::HomeProgramTransitionStepAssociation,
        original: &worth_foundational::facade::CompositeAttemptAssociation,
        admission: &worth_proof::AdmittedHomeProgramTransitionStep<
            worth_relational::facade::branch::RelationalBranchBasisAdmissionIdentity,
        >,
        claim: crate::recovery::WorldRecoveredUnpublishedEffects,
        change: worth_relational::facade::mvcc::RelationalTransactionIntent,
        cancellation: &RuntimeWorldCancellationToken,
        deadline: Option<RuntimeWorldInstant>,
    ) -> Result<WorldOpenedPublicationAttempt, Refused> {
        match self.service() {
            Ok(owner) => owner.resume_home_transition_open(
                execution,
                plan,
                step,
                original,
                admission,
                claim,
                change,
                cancellation,
                deadline,
            ),
            Err(_) => Err(Refused {
                cause: Denial::Source(NoEffectCause::OwnerUnavailable),
                custody: claim,
            }),
        }
    }
}
