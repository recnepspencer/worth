use super::reservation_steps::{
    issue_publication_identities, reserve_publication_resources, resources_to_capacities,
};
use super::*;
use crate::durability::{
    DurableWorldHead, WorldDurableArtifact, WorldOpenAttemptRecord, WorldRecord,
};
use crate::publication::{
    WorldOpenAttemptOutcome, WorldOpenAttemptRefusal, WorldPendingOpenAttempt,
};
use worth_foundational::facade::{
    CompositeAttemptAssociation, DurableCompositeCommitIdentity,
    DurableCompositePublicationAttemptIdentity, DurableProductBranchIdentity,
    DurableProductBranchIncarnation, DurableProductReferenceGeneration,
    DurableProductUnpublishedOwnerEffectsIdentity,
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
        change: worth_relational::facade::mvcc::RelationalTransactionIntent,
        cancellation: &RuntimeWorldCancellationToken,
        deadline: Option<RuntimeWorldInstant>,
    ) -> WorldOpenAttemptOutcome {
        match self.prepare_durable_open(execution, expected, change, cancellation, deadline) {
            Ok(pending) => pending.advance(execution),
            Err(cause) => WorldOpenAttemptOutcome::Refused(cause),
        }
    }
    pub(in crate::lifecycle) fn prepare_durable_open(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        expected: ProductBranchObservation,
        change: worth_relational::facade::mvcc::RelationalTransactionIntent,
        cancellation: &RuntimeWorldCancellationToken,
        deadline: Option<RuntimeWorldInstant>,
    ) -> Result<WorldPendingOpenAttempt, WorldOpenAttemptRefusal> {
        use WorldOpenAttemptRefusal as Refusal;
        let binding = self
            .state
            .durable
            .as_ref()
            .ok_or(Refusal::NotHomeBound)?
            .clone();
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
        let current = self
            .admit_publication_source(&expected)
            .map_err(Refusal::Reservation)?;
        let resolved = ResolvedExpectedProductHead::from_current(
            crate::publication::CompositeComponentIntent::relational_only(change),
            expected.clone(),
            &current,
        )
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
        let identities = issue_publication_identities(self).map_err(|cause| {
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
        )
        .map_err(|cause| {
            Refusal::Reservation(NoEffectCompositePublication::new(
                cause,
                Some(expected.clone()),
            ))
        })?;
        let branch = DurableProductBranchIdentity::from_recorded(
            owner,
            expected.branch_identity().name().as_str().to_owned(),
        )
        .map_err(|_| Refusal::IdentityScope)?;
        let incarnation = DurableProductBranchIncarnation::from_recorded(
            owner,
            expected.lifecycle_incarnation().ordinal(),
        );
        let association = CompositeAttemptAssociation::from_recorded(
            DurableCompositePublicationAttemptIdentity::from_recorded(
                owner,
                identities.attempt_identity.ordinal(),
            ),
            branch.clone(),
            incarnation,
        )
        .map_err(|_| Refusal::IdentityScope)?;
        let record = WorldOpenAttemptRecord {
            component_intent: crate::durability::WorldOpenComponentIntent::RelationalOnly,
            association,
            commit: DurableCompositeCommitIdentity::from_recorded(
                owner,
                identities.commit_identity.ordinal(),
            ),
            custody: DurableProductUnpublishedOwnerEffectsIdentity::from_recorded(
                owner,
                identities.product_unpublished_identity.ordinal(),
            ),
            expected: DurableWorldHead {
                branch,
                incarnation,
                generation: DurableProductReferenceGeneration::from_recorded(
                    expected.reference_generation().get(),
                ),
                commit: DurableCompositeCommitIdentity::from_recorded(
                    owner,
                    expected.selected_commit().ordinal(),
                ),
            },
            program_revision: binding.binding().program_revision(),
        };
        let association_bytes =
            crate::durability::encode_association(execution, &record.association)
                .map_err(Refusal::Durable)?;
        let subject =
            WorldDurableArtifact::from_record(WorldRecord::OpenAttempt(record.clone()), execution)
                .map_err(Refusal::Durable)?;
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
        let durable_custody =
            crate::durability::WorldDurableOpenCustody::submitted(binding.clone(), subject.clone());
        custody.retain_durable_open(Some(durable_custody.clone()));
        Ok(WorldPendingOpenAttempt {
            binding,
            durable_custody,
            subject,
            record,
            association_bytes,
            identity: identities.attempt_identity,
            resolved,
            custody,
            deadline,
        })
    }
}
