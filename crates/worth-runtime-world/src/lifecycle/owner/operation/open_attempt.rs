use super::reservation_steps::{
    issue_publication_identities_for_attempt, reserve_publication_resources,
    resources_to_capacities,
};
use super::*;
use crate::durability::{
    WorldArtifactInput, WorldDurableArtifact, WorldHeadInput, WorldOpenAttemptInput,
};
use crate::publication::{
    WorldOpenAttemptOutcome, WorldOpenAttemptRefusal, WorldPendingOpenAttempt,
};
use worth_foundational::facade::{
    DurableCompositeCommitIdentity, DurableCompositePublicationAttemptIdentity,
    DurableProductBranchIncarnation, DurableProductUnpublishedOwnerEffectsIdentity,
};
impl<D, I, E, Ctx, T> RuntimeWorldOwnerRoot<D, I, E, Ctx, T>
where
    D: Copy + Ord + std::fmt::Debug + Send + Sync + 'static,
    I: Copy + Ord + Send + Sync + 'static,
    T: Copy + Ord + Send + Sync + 'static,
{
    pub fn begin_open_without_signal(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        expected: ProductBranchObservation,
        source: worth_proof::AdmittedBranchProgramSource,
        change: worth_relational::facade::mvcc::RelationalTransactionIntent,
        cancellation: &RuntimeWorldCancellationToken,
        deadline: Option<RuntimeWorldInstant>,
    ) -> WorldOpenAttemptOutcome {
        match self.prepare_durable_open(execution, expected, source, change, cancellation, deadline)
        {
            Ok(pending) => pending.advance(execution),
            Err(cause) => WorldOpenAttemptOutcome::Refused(cause),
        }
    }
    pub(in crate::lifecycle) fn prepare_durable_open(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        expected: ProductBranchObservation,
        source: worth_proof::AdmittedBranchProgramSource,
        change: worth_relational::facade::mvcc::RelationalTransactionIntent,
        cancellation: &RuntimeWorldCancellationToken,
        deadline: Option<RuntimeWorldInstant>,
    ) -> Result<WorldPendingOpenAttempt, WorldOpenAttemptRefusal> {
        self.prepare_durable_open_with_assignment(
            execution,
            expected,
            source,
            change,
            cancellation,
            deadline,
            None,
        )
    }

    pub(in crate::lifecycle) fn prepare_durable_open_with_assignment(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        expected: ProductBranchObservation,
        source: worth_proof::AdmittedBranchProgramSource,
        change: worth_relational::facade::mvcc::RelationalTransactionIntent,
        cancellation: &RuntimeWorldCancellationToken,
        deadline: Option<RuntimeWorldInstant>,
        assigned: Option<crate::publication::HomeTransitionOpenAssignment>,
    ) -> Result<WorldPendingOpenAttempt, WorldOpenAttemptRefusal> {
        self.prepare_component_durable_open(
            execution,
            expected,
            source,
            crate::publication::CompositeComponentIntent::relational_only(change),
            cancellation,
            deadline,
            assigned,
        )
    }

    pub(in crate::lifecycle) fn prepare_component_durable_open(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        expected: ProductBranchObservation,
        source: worth_proof::AdmittedBranchProgramSource,
        intent: crate::publication::CompositeComponentIntent,
        cancellation: &RuntimeWorldCancellationToken,
        deadline: Option<RuntimeWorldInstant>,
        assigned: Option<crate::publication::HomeTransitionOpenAssignment>,
    ) -> Result<WorldPendingOpenAttempt, WorldOpenAttemptRefusal> {
        use WorldOpenAttemptRefusal as Refusal;
        let current = self
            .admit_publication_source(&expected)
            .map_err(Refusal::Reservation)?;
        let binding = self
            .state
            .durable
            .as_ref()
            .ok_or(Refusal::NotHomeBound)?
            .clone();
        let selected_source_program = source.program_revision();
        let commitment = crate::publication::world_branch_program_source_commitment(
            &expected,
            selected_source_program,
        )
        .map_err(Refusal::Durable)?;
        binding
            .binding()
            .branch_program_source_authority()
            .ok_or(Refusal::ProgramSourceOwnerRequired)?
            .admit(&source, &commitment)
            .map_err(Refusal::ProgramSource)?;
        if binding.occurrence_unresolved() {
            return Err(Refusal::Durable(
                crate::durability::WorldDurableMediumDenial::PublicationUnresolved,
            ));
        }
        let owner = self
            .owner_identity()
            .durable()
            .ok_or(Refusal::IdentityScope)?;
        if owner != binding.binding().owner() {
            return Err(Refusal::IdentityScope);
        }
        let resolved =
            ResolvedExpectedProductHead::from_current(intent.clone(), expected.clone(), &current)
                .map_err(|_| Refusal::IdentityScope)?;
        if let Some(denied) = self.reservation_denial(&expected, cancellation, deadline) {
            return Err(Refusal::Reservation(denied));
        }
        if expected.reference_generation().advance().is_err() {
            return Err(Refusal::IdentityScope);
        }
        let operation = self
            .reserve_operation_if_open_and_bootstrapped()
            .map_err(|()| {
                Refusal::Reservation(NoEffectCompositePublication::new(
                    NoEffectCause::OwnerUnavailable,
                    Some(expected.clone()),
                ))
            })?;
        let (assigned, home_association, original_program_revision) = match assigned {
            Some(assigned) => (
                Some(assigned.attempt),
                Some(assigned.association),
                assigned.original_program_revision,
            ),
            None => (None, None, selected_source_program),
        };
        let identities =
            issue_publication_identities_for_attempt(self, assigned).map_err(|cause| {
                Refusal::Reservation(NoEffectCompositePublication::new(
                    cause,
                    Some(expected.clone()),
                ))
            })?;
        let resources = reserve_publication_resources(
            self,
            &expected,
            &identities.commit_identity,
            Some(&identities.attempt_identity),
            true,
            None,
        )
        .map_err(|cause| {
            Refusal::Reservation(NoEffectCompositePublication::new(
                cause,
                Some(expected.clone()),
            ))
        })?;
        let subject = WorldDurableArtifact::from_input(
            WorldArtifactInput::OpenAttempt(WorldOpenAttemptInput {
                attempt: DurableCompositePublicationAttemptIdentity::from_recorded(
                    owner,
                    identities.attempt_identity.ordinal(),
                ),
                name: expected.branch_identity().name().as_str(),
                incarnation: DurableProductBranchIncarnation::from_recorded(
                    owner,
                    expected.lifecycle_incarnation().ordinal(),
                ),
                commit: DurableCompositeCommitIdentity::from_recorded(
                    owner,
                    identities.commit_identity.ordinal(),
                ),
                custody: DurableProductUnpublishedOwnerEffectsIdentity::from_recorded(
                    owner,
                    identities.product_unpublished_identity.ordinal(),
                ),
                expected: WorldHeadInput::from_observation(&expected).map_err(Refusal::Durable)?,
                program_revision: original_program_revision,
                component_intent: match intent {
                    crate::publication::CompositeComponentIntent::RelationalOnly(_) => {
                        crate::durability::WorldOpenComponentIntent::RelationalOnly
                    }
                    crate::publication::CompositeComponentIntent::SignalOnly => {
                        crate::durability::WorldOpenComponentIntent::SignalOnly
                    }
                    crate::publication::CompositeComponentIntent::RelationalAndSignal(_) => {
                        crate::durability::WorldOpenComponentIntent::RelationalAndSignal
                    }
                },
            }),
            execution,
        )
        .map_err(Refusal::Durable)?;
        let original = &subject
            .open_attempt()
            .expect("the owned input was an open")
            .association;
        let association_bytes = match home_association {
            Some(bytes) => {
                if crate::durability::decode_association(bytes.elements(), owner.home())
                    .map_err(|cause| Refusal::Durable(cause.into()))?
                    != *original
                {
                    return Err(Refusal::IdentityScope);
                }
                bytes
            }
            None => crate::durability::encode_association(execution, original)
                .map_err(Refusal::Durable)?,
        };
        let capacities = resources_to_capacities(
            resources,
            self.state.clock.now(),
            identities.commit_identity,
            identities.product_unpublished_identity,
            operation,
        );
        let mut custody = crate::publication::ActiveAttemptCustody::register(
            identities.attempt_identity.clone(),
            expected,
            deadline,
            capacities,
        );
        // Installation precedes adapter submission and survives every unwind.
        let durable_custody = crate::durability::WorldDurableOpenCustody::submitted(
            binding.clone(),
            subject.clone(),
            execution,
        )
        .map_err(Refusal::Durable)?;
        custody.retain_durable_open(Some(durable_custody.clone()));
        Ok(WorldPendingOpenAttempt {
            original: crate::publication::WorldOpenAttemptOriginal {
                binding,
                durable_custody,
                subject,
                association_bytes,
                identity: identities.attempt_identity,
                resolved,
                custody,
                deadline,
            },
        })
    }
}
