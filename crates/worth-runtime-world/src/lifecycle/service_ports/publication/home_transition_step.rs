//! Consume one reserved original attempt only after committed Home admission.
use super::*;
use crate::publication::{
    WorldHomeTransitionStepReservation, WorldOpenAttemptRefusal, WorldPendingOpenAttempt,
};
use worth_foundational::facade::{
    HomeProgramTransitionPlanDescription, HomeProgramTransitionStepAssociation,
};

impl<D, I, E, Ctx, T> RuntimeWorldPublicationPort<D, I, E, Ctx, T>
where
    D: Copy + Ord + std::fmt::Debug + Send + Sync + 'static,
    I: Copy + Ord + Send + Sync + 'static,
    E: Send + Sync + 'static,
    Ctx: Send + Sync + 'static,
    T: Copy + Ord + Send + Sync + 'static,
{
    pub fn prepare_home_transition_step(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        expected: ProductBranchObservation,
        program_source: worth_proof::AdmittedBranchProgramSource,
        change: worth_relational::facade::mvcc::RelationalTransactionIntent,
        plan: &HomeProgramTransitionPlanDescription,
        original: &worth_foundational::facade::CompositeAttemptAssociation,
        reservation: WorldHomeTransitionStepReservation,
        admission: &worth_proof::AdmittedHomeProgramTransitionStep<
            worth_relational::facade::branch::RelationalBranchBasisAdmissionIdentity,
        >,
        cancellation: &RuntimeWorldCancellationToken,
        deadline: Option<RuntimeWorldInstant>,
    ) -> Result<
        WorldPendingOpenAttempt,
        (WorldHomeTransitionStepReservation, WorldOpenAttemptRefusal),
    > {
        let prepared = (|| {
            let owner = self.service().map_err(|_| {
                WorldOpenAttemptRefusal::Reservation(NoEffectCompositePublication::new(
                    NoEffectCause::OwnerUnavailable,
                    Some(expected.clone()),
                ))
            })?;
            let binding = owner
                .state
                .durable
                .as_ref()
                .ok_or(WorldOpenAttemptRefusal::NotHomeBound)?;
            let step = HomeProgramTransitionStepAssociation::from_recorded(
                plan.identity(),
                reservation.step,
            );
            let commitment = plan
                .step_commitment(step, original)
                .map_err(|_| WorldOpenAttemptRefusal::IdentityScope)?;
            binding
                .binding()
                .transition_authority()
                .ok_or(WorldOpenAttemptRefusal::IdentityScope)?
                .admit_step(admission, &commitment, &reservation.caller_association)
                .map_err(WorldOpenAttemptRefusal::TransitionAdmission)?;
            if admission.source() != expected.basis().relational_basis().admission_identity() {
                return Err(WorldOpenAttemptRefusal::IdentityScope);
            }
            if reservation.plan != plan.commitment()
                || reservation.attempt.ordinal() != original.attempt().ordinal()
                || reservation.attempt.owner_identity().durable()
                    != Some(original.attempt().owner())
                || original.branch().name() != expected.branch_identity().name().as_str()
                || original.incarnation().ordinal() != expected.lifecycle_incarnation().ordinal()
                || original.branch().owner() != binding.binding().owner()
            {
                return Err(WorldOpenAttemptRefusal::IdentityScope);
            }
            owner.prepare_durable_open_with_assignment(
                execution,
                expected,
                program_source,
                change,
                cancellation,
                deadline,
                Some(crate::publication::HomeTransitionOpenAssignment {
                    attempt: reservation.attempt.clone(),
                    association: crate::durability::encode_home_transition_association(
                        execution, plan, step, original,
                    )
                    .map_err(WorldOpenAttemptRefusal::Durable)?,
                }),
            )
        })();
        match prepared {
            Ok(pending) => Ok(pending),
            Err(cause) => Err((reservation, cause)),
        }
    }
}
