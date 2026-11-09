//! The same graph edge admission serves original streams and normalized rows.
use super::*;
use crate::durability::checkpoint::{WorldBranchImageKey, WorldImageHead};
use worth_execution::ExecutionRequest;
use worth_foundational::facade::{DurableCompositePublicationAttemptIdentity, HomeIncarnation};

impl WorldCheckpointImage {
    pub(in crate::durability) fn admit_occurrence(
        &mut self,
        attempt: DurableCompositePublicationAttemptIdentity,
        owners: super::super::artifact::WorldOccurrenceOwnerEvidence,
        publishing_incarnation: HomeIncarnation,
        position: u64,
        digest: [u8; 32],
        execution: &ExecutionRequest<'_, '_>,
    ) -> Result<(), WorldReadmissionDenial> {
        let open = self
            .opens
            .get(&attempt)
            .ok_or(WorldReadmissionDenial::MissingOpenAttempt)?
            .clone();
        if owners.component_intent() != open.component_intent {
            return Err(WorldReadmissionDenial::OccurrenceEvidenceMismatch);
        }
        if let Some(signal) = owners.signal() {
            if signal.point().descriptor().owner().home() != self.binding.owner().home() {
                return Err(WorldReadmissionDenial::ForeignOwner);
            }
        }
        if publishing_incarnation > self.binding.incarnation() {
            return Err(WorldReadmissionDenial::FutureIncarnation);
        }
        if matches!(
            self.dispositions.get(&attempt).map(|row| row.disposition()),
            Some(RecoveredCustodyDisposition::Released(_))
        ) {
            return Err(WorldReadmissionDenial::ConflictingDisposition);
        }
        let key = WorldBranchImageKey::new(open.record().clone(), position, digest);
        let current = self
            .heads
            .get(&key)
            .ok_or(WorldReadmissionDenial::MissingPredecessor)?;
        if !current.matches(&open.expected) {
            return Err(WorldReadmissionDenial::BranchLineage);
        }
        if self.history.contains_key(&open.commit) || self.occurrences.contains_key(&position) {
            return Err(WorldReadmissionDenial::DuplicateIdentity);
        }
        let successor = WorldImageHead::occurrence(open.record().clone())?;
        super::journal::insert(&mut self.history, &open.commit, &successor, execution)?;
        super::journal::insert(&mut self.heads, &key, &successor, execution)?;
        let occurrence = AdmittedWorldOccurrence::from_validated_source(
            open,
            owners,
            publishing_incarnation,
            position,
            digest,
        );
        super::journal::insert(&mut self.occurrences, &position, &occurrence, execution)
    }
}
