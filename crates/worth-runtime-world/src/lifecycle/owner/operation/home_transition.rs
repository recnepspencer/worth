//! Reserve attempts without inventing future heads, commits or Native effects.
use super::*;
use crate::publication::{
    ReservedWorldHomeTransitionAssignments, WorldHomeTransitionReservationRefusal as Refusal,
};
use worth_execution::{ExecutionRequest, ExecutionRequestArrayBuilder};
use worth_foundational::facade::{
    CompositeAttemptAssociation, DurableCompositePublicationAttemptIdentity,
    HomeProgramTransitionPlanDescription,
};

impl<D, I, E, Ctx, T> RuntimeWorldOwnerRoot<D, I, E, Ctx, T>
where
    D: Copy + Ord + std::fmt::Debug + Send + Sync + 'static,
    I: Copy + Ord + Send + Sync + 'static,
    T: Copy + Ord + Send + Sync + 'static,
{
    pub(in crate::lifecycle) fn reserve_home_transition_assignments(
        &self,
        request: &ExecutionRequest<'_, '_>,
        expected: ProductBranchObservation,
        description: HomeProgramTransitionPlanDescription,
        proof: worth_proof::AdmittedHomeProgramTransitionPlan,
    ) -> Result<ReservedWorldHomeTransitionAssignments, Refusal> {
        let current = self
            .admit_publication_source(&expected)
            .map_err(Refusal::Source)?;
        let binding = self
            .state
            .durable
            .as_ref()
            .ok_or(Refusal::NotHomeBound)?
            .binding();
        binding
            .transition_authority()
            .ok_or(Refusal::UnboundTransitionOwner)?
            .admit_plan(&proof, &description.commitment())
            .map_err(Refusal::Admission)?;
        let source = description.predecessor();
        let owner = binding.owner();
        if description.identity().home() != owner.home()
            || source.branch().owner() != owner
            || source.branch().name() != expected.branch_identity().name().as_str()
            || source.incarnation().ordinal() != expected.lifecycle_incarnation().ordinal()
            || source.generation().get() != expected.reference_generation().get()
            || source.selected_commit().ordinal() != expected.selected_commit().ordinal()
            || expected.mismatch_against_snapshot(&current).is_some()
        {
            return Err(Refusal::SourceMismatch);
        }
        let count = description.steps() as usize;
        let names = count
            .checked_mul(source.branch().name().len())
            .and_then(|bytes| u64::try_from(bytes).ok())
            .ok_or(Refusal::CapacityOverflow)?;
        let work = names
            .checked_add(u64::from(description.steps()))
            .and_then(|work| work.checked_add(512))
            .ok_or(Refusal::CapacityOverflow)?;
        request
            .run_native_with_checkpoint(|context| {
                if context.has_accounting() {
                    context.checkpoint(work)
                } else {
                    context.poll()
                }
            })
            .map_err(Refusal::Request)?
            .0
            .map_err(|cause| {
                Refusal::Request(worth_execution::WorkCeilingDenial::Stopped(cause))
            })?;
        let names_memory = request
            .reserve_retained_memory(names)
            .map_err(Refusal::Request)?;
        let mut assignments =
            ExecutionRequestArrayBuilder::allocate(*request, count).map_err(Refusal::Backing)?;
        let mut steps =
            ExecutionRequestArrayBuilder::allocate(*request, count).map_err(Refusal::Backing)?;
        let operation = self
            .reserve_operation_if_open_and_bootstrapped()
            .map_err(|()| {
                Refusal::Source(NoEffectCompositePublication::new(
                    NoEffectCause::OwnerUnavailable,
                    Some(expected.clone()),
                ))
            })?;
        // Only World advances this issuer. Gaps after refusal are harmless;
        // reuse of any assignment, including one whose open is absent, is not.
        let mut issuer = self
            .state
            .identities
            .lock()
            .unwrap_or_else(|error| error.into_inner());
        for step in 0..count {
            request.consult().map_err(Refusal::Request)?;
            let issued = issuer.publication_attempt().map_err(Refusal::Identity)?;
            let association = CompositeAttemptAssociation::from_recorded(
                DurableCompositePublicationAttemptIdentity::from_recorded(owner, issued.ordinal()),
                source.branch().clone(),
                source.incarnation(),
            )
            .expect("all coordinates came from the same admitted World source");
            let caller_association =
                crate::publication::home_transition_caller_association_commitment(
                    request,
                    &description,
                    worth_foundational::facade::HomeProgramTransitionStepAssociation::from_recorded(
                        description.identity(),
                        step as u32,
                    ),
                    &association,
                )
                .map_err(Refusal::Association)?;
            steps
                .push(std::sync::Mutex::new(Some(
                    crate::publication::WorldHomeTransitionStepReservation {
                        attempt: issued,
                        plan: description.commitment(),
                        step: step as u32,
                        caller_association,
                    },
                )))
                .map_err(Refusal::Backing)?;
            assignments.push(association).map_err(Refusal::Backing)?;
        }
        drop(issuer);
        Ok(ReservedWorldHomeTransitionAssignments {
            description,
            assignments: assignments.seal().map_err(Refusal::Backing)?,
            predecessor: expected,
            _operation: operation,
            _names_memory: names_memory,
            steps: steps.seal().map_err(Refusal::Backing)?,
        })
    }
}
