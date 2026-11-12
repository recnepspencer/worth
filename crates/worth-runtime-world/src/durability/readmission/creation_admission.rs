//! Collect exact creation originals before resolving cross-journal dependencies.
use super::creation::{
    AdmittedWorldCreationIntent, AdmittedWorldCreationRelease, RecordedWorldBranchCreated,
};
use super::*;
use std::sync::Arc;
use worth_execution::ExecutionRequest;

impl WorldCheckpointImage {
    pub(in crate::durability) fn collect_creation_intent(
        &mut self,
        record: Arc<DecodedWorldRecord>,
        position: u64,
        digest: [u8; 32],
        request: &ExecutionRequest<'_, '_>,
    ) -> Result<(), WorldReadmissionDenial> {
        let WorldRecord::CreationIntent(intent) = &record.record else {
            unreachable!("creation intent class")
        };
        let attempt = intent.association.attempt();
        if self.opens.contains_key(&attempt) || self.creations.contains_key(&attempt) {
            return Err(WorldReadmissionDenial::DuplicateIdentity);
        }
        let row = AdmittedWorldCreationIntent::from_validated_journal(record, position, digest);
        super::journal::insert(&mut self.creations, &attempt, &row, request)
    }

    pub(in crate::durability) fn collect_created_branch(
        &mut self,
        record: Arc<DecodedWorldRecord>,
        position: u64,
        digest: [u8; 32],
        request: &ExecutionRequest<'_, '_>,
    ) -> Result<(), WorldReadmissionDenial> {
        let WorldRecord::BranchCreated(created) = &record.record else {
            unreachable!("selected creation class")
        };
        let attempt = created.intent.association.attempt();
        if created.publishing_incarnation > self.binding.incarnation() {
            return Err(WorldReadmissionDenial::FutureIncarnation);
        }
        if self.created.contains_key(&attempt) {
            return Err(WorldReadmissionDenial::DuplicateIdentity);
        }
        let coordinate = (
            created.intent.destination.incarnation,
            created.intent.destination.generation,
        );
        let row = RecordedWorldBranchCreated::from_validated_journal(record, position, digest);
        super::journal::insert(&mut self.created, &attempt, &row, request)?;
        self.queue_history_step(
            coordinate,
            super::CollectedWorldHistoryStep::BranchCreated(attempt),
            request,
        )
    }

    pub(in crate::durability) fn admit_creation_release(
        &mut self,
        association: &CompositeAttemptAssociation,
        custody: DurableProductUnpublishedOwnerEffectsIdentity,
        position: u64,
        digest: [u8; 32],
        request: &ExecutionRequest<'_, '_>,
    ) -> Result<(), WorldReadmissionDenial> {
        let attempt = association.attempt();
        let intent = self
            .creations
            .get(&attempt)
            .ok_or(WorldReadmissionDenial::MissingCreationIntent)?
            .clone();
        if &intent.intent().association != association || intent.intent().custody != custody {
            return Err(WorldReadmissionDenial::ConflictingCreationIntent);
        }
        if position <= intent.position()
            || self.creation_releases.contains_key(&attempt)
            || self.created.contains_key(&attempt)
        {
            return Err(WorldReadmissionDenial::ConflictingDisposition);
        }
        let row = AdmittedWorldCreationRelease::from_validated_journal(intent, position, digest);
        super::journal::insert(&mut self.creation_releases, &attempt, &row, request)
    }
}
