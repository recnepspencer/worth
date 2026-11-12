//! Original occurrence fields remain unadmitted until their source graph closes.
use crate::durability::artifact::WorldOccurrenceOwnerEvidence;
use worth_foundational::facade::{DurableCompositePublicationAttemptIdentity, HomeIncarnation};

#[derive(Clone, Debug)]
pub(crate) struct CollectedWorldOccurrence {
    pub(crate) attempt: DurableCompositePublicationAttemptIdentity,
    pub(crate) owners: WorldOccurrenceOwnerEvidence,
    pub(crate) publishing_incarnation: HomeIncarnation,
    pub(crate) position: u64,
    pub(crate) digest: [u8; 32],
}

impl super::WorldCheckpointImage {
    pub(in crate::durability) fn collect_occurrence(
        &mut self,
        attempt: DurableCompositePublicationAttemptIdentity,
        owners: WorldOccurrenceOwnerEvidence,
        publishing_incarnation: HomeIncarnation,
        position: u64,
        digest: [u8; 32],
        request: &worth_execution::ExecutionRequest<'_, '_>,
    ) -> Result<(), super::WorldReadmissionDenial> {
        use super::WorldReadmissionDenial as Denial;
        let open = self.opens.get(&attempt).ok_or(Denial::MissingOpenAttempt)?;
        if owners.component_intent() != open.component_intent {
            return Err(Denial::OccurrenceEvidenceMismatch);
        }
        if let Some(signal) = owners.signal() {
            if signal.point().descriptor().owner().home() != self.binding.owner().home() {
                return Err(Denial::ForeignOwner);
            }
        }
        if publishing_incarnation > self.binding.incarnation() {
            return Err(Denial::FutureIncarnation);
        }
        if self.occurrences.contains_key(&position)
            || self.collected_occurrences.contains_key(&position)
        {
            return Err(Denial::DuplicateIdentity);
        }
        let generation = open
            .expected
            .generation
            .get()
            .checked_add(1)
            .ok_or(Denial::BranchLineage)?;
        let coordinate = (
            open.expected.incarnation,
            worth_foundational::facade::DurableProductReferenceGeneration::from_recorded(
                generation,
            ),
        );
        let row = CollectedWorldOccurrence {
            attempt,
            owners,
            publishing_incarnation,
            position,
            digest,
        };
        super::journal::insert(&mut self.collected_occurrences, &position, &row, request)?;
        self.queue_history_step(
            coordinate,
            super::CollectedWorldHistoryStep::Occurrence(position),
            request,
        )
    }
}
