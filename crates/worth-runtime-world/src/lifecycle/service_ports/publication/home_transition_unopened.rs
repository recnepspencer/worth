//! The publication receiver supplies the current owner to original-open admission.
use super::*;
use crate::publication::{
    WorldHomeTransitionUnopenedDenial as Denial, WorldOpenAttemptRefusal as Open,
    WorldPendingOpenAttempt,
};
use worth_foundational::facade::{
    CompositeAttemptAssociation, HomeProgramTransitionPlanDescription,
};
use worth_proof::{AdmittedHomeProgramTransitionExecution, AdmittedHomeProgramTransitionStep};
use worth_relational::facade::branch::RelationalBranchBasisAdmissionIdentity;

impl<D, I, E, Ctx, T> RuntimeWorldPublicationPort<D, I, E, Ctx, T>
where
    D: Copy + Ord + std::fmt::Debug + Send + Sync + 'static,
    I: Copy + Ord + Send + Sync + 'static,
    E: Send + Sync + 'static,
    Ctx: Send + Sync + 'static,
    T: Copy + Ord + Send + Sync + 'static,
{
    pub fn prepare_readmitted_home_transition_step(
        &self,
        request: &worth_execution::ExecutionRequest<'_, '_>,
        expected: ProductBranchObservation,
        change: worth_relational::facade::mvcc::RelationalTransactionIntent,
        plan: &HomeProgramTransitionPlanDescription,
        step: u32,
        original: &CompositeAttemptAssociation,
        admission: &AdmittedHomeProgramTransitionStep<RelationalBranchBasisAdmissionIdentity>,
        execution: AdmittedHomeProgramTransitionExecution,
        cancellation: &RuntimeWorldCancellationToken,
        deadline: Option<RuntimeWorldInstant>,
    ) -> Result<WorldPendingOpenAttempt, (AdmittedHomeProgramTransitionExecution, Denial)> {
        let owner = match self.service() {
            Ok(owner) => owner,
            Err(_) => {
                return Err((
                    execution,
                    Denial::Open(Open::Reservation(NoEffectCompositePublication::new(
                        NoEffectCause::OwnerUnavailable,
                        Some(expected),
                    ))),
                ))
            }
        };
        owner.prepare_readmitted_home_transition_step(
            request,
            expected,
            change,
            plan,
            step,
            original,
            admission,
            execution,
            cancellation,
            deadline,
        )
    }
}
