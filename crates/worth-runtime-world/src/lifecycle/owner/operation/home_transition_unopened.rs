//! Start the absent original open under a single Home-issued execution witness.
use super::*;
use crate::publication::{
    WorldHomeTransitionUnopenedDenial as Denial, WorldOpenAttemptRefusal as Open,
    WorldPendingOpenAttempt,
};
use worth_foundational::facade::{
    CompositeAttemptAssociation, HomeProgramTransitionPlanDescription,
    HomeProgramTransitionStepAssociation,
};
use worth_proof::{AdmittedHomeProgramTransitionExecution, AdmittedHomeProgramTransitionStep};
use worth_relational::facade::branch::RelationalBranchBasisAdmissionIdentity;

impl<D, I, E, Ctx, T> RuntimeWorldOwnerRoot<D, I, E, Ctx, T>
where
    D: Copy + Ord + std::fmt::Debug + Send + Sync + 'static,
    I: Copy + Ord + Send + Sync + 'static,
    T: Copy + Ord + Send + Sync + 'static,
{
    pub(in crate::lifecycle) fn prepare_readmitted_home_transition_step(
        &self,
        request: &worth_execution::ExecutionRequest<'_, '_>,
        expected: ProductBranchObservation,
        program_source: worth_proof::AdmittedBranchProgramSource,
        change: worth_relational::facade::mvcc::RelationalTransactionIntent,
        plan: &HomeProgramTransitionPlanDescription,
        step: u32,
        original: &CompositeAttemptAssociation,
        admission: &AdmittedHomeProgramTransitionStep<RelationalBranchBasisAdmissionIdentity>,
        execution: AdmittedHomeProgramTransitionExecution,
        cancellation: &RuntimeWorldCancellationToken,
        deadline: Option<RuntimeWorldInstant>,
    ) -> Result<WorldPendingOpenAttempt, (AdmittedHomeProgramTransitionExecution, Denial)> {
        let result = (|| {
            let binding = self
                .state
                .durable
                .as_ref()
                .ok_or(Denial::Open(Open::NotHomeBound))?;
            let described =
                HomeProgramTransitionStepAssociation::from_recorded(plan.identity(), step);
            let commitment = plan
                .step_commitment(described, original)
                .map_err(|_| Denial::Open(Open::IdentityScope))?;
            let authority = binding
                .binding()
                .transition_authority()
                .ok_or(Denial::Open(Open::IdentityScope))?;
            // Both attached seals/fences are checked before inspecting the live
            // source or journal. A retained validation grant is not execution.
            authority
                .admit_step(
                    admission,
                    &commitment,
                    &admission.caller_association_commitment(),
                )
                .map_err(Denial::Admission)?;
            authority
                .admit_execution(
                    &execution,
                    &commitment,
                    &admission.caller_association_commitment(),
                )
                .map_err(Denial::Admission)?;
            if admission.source() != expected.basis().relational_basis().admission_identity() {
                return Err(Denial::Open(Open::IdentityScope));
            }
            let current = self
                .admit_publication_source(&expected)
                .map_err(|cause| Denial::Open(Open::Reservation(cause)))?;
            if expected.mismatch_against_snapshot(&current).is_some()
                || original.branch().owner() != binding.binding().owner()
                || original.branch().name() != expected.branch_identity().name().as_str()
                || original.incarnation().ordinal() != expected.lifecycle_incarnation().ordinal()
                || plan
                    .predecessor()
                    .generation()
                    .get()
                    .checked_add(u64::from(step))
                    != Some(expected.reference_generation().get())
            {
                return Err(Denial::Open(Open::IdentityScope));
            }
            if let Some(cause) = self.reservation_denial(&expected, cancellation, deadline) {
                return Err(Denial::Open(Open::Reservation(cause)));
            }
            let association = crate::publication::home_transition_caller_association_commitment(
                request, plan, described, original,
            )
            .map_err(|cause| Denial::Open(Open::Durable(cause)))?;
            authority
                .admit_step(admission, &commitment, &association)
                .map_err(Denial::Admission)?;
            authority
                .admit_execution(&execution, &commitment, &association)
                .map_err(Denial::Admission)?;
            let source = binding
                .capture_checkpoint_source(request)
                .map_err(|cause| Denial::Open(Open::Durable(cause)))?;
            let (image, _) = crate::durability::WorldCheckpointImage::read_protected_source(
                source.as_ref(),
                binding.binding(),
                request,
                None,
            )
            .map_err(Denial::Readmission)?;
            let attempt = original.attempt();
            if image.opens.get(&attempt).is_some()
                || image.dispositions.get(&attempt).is_some()
                || image
                    .occurrences
                    .values()
                    .any(|occurrence| occurrence.open().association.attempt() == attempt)
            {
                return Err(Denial::OriginalOpenAlreadyDurable);
            }
            drop((image, source));
            let attempt =
                crate::identity::readmission::readmit_attempt(self.owner_identity(), attempt)
                    .ok_or(Denial::Open(Open::IdentityScope))?;
            self.prepare_durable_open_with_assignment(
                request,
                expected,
                program_source,
                change,
                cancellation,
                deadline,
                Some(crate::publication::HomeTransitionOpenAssignment {
                    original_program_revision: binding.binding().program_revision(),
                    attempt,
                    association: crate::durability::encode_home_transition_association(
                        request, plan, described, original,
                    )
                    .map_err(|cause| Denial::Open(Open::Durable(cause)))?,
                }),
            )
            .map_err(Denial::Open)
        })();
        match result {
            Ok(pending) => Ok(pending),
            Err(cause) => Err((execution, cause)),
        }
    }
}
