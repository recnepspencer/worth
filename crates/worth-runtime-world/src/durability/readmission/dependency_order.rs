//! Birth incarnations and reference generations form the structural dependency order.
use super::{WorldCheckpointImage, WorldReadmissionDenial as Denial};
use crate::durability::artifact::WorldRecordedRelationalCreationPlan as Native;
use crate::durability::checkpoint::{WorldBranchImageKey, WorldImageHead};
use worth_execution::ExecutionRequest;
use worth_foundational::facade::{
    DurableCompositePublicationAttemptIdentity, DurableProductBranchIncarnation,
    DurableProductReferenceGeneration,
};

#[derive(Clone, Copy, Debug)]
pub(crate) enum CollectedWorldHistoryStep {
    BranchCreated(DurableCompositePublicationAttemptIdentity),
    Occurrence(u64),
}

impl WorldCheckpointImage {
    pub(super) fn queue_history_step(
        &mut self,
        coordinate: (
            DurableProductBranchIncarnation,
            DurableProductReferenceGeneration,
        ),
        step: CollectedWorldHistoryStep,
        request: &ExecutionRequest<'_, '_>,
    ) -> Result<(), Denial> {
        if self.references.contains_key(&coordinate)
            || self.collected_steps.contains_key(&coordinate)
        {
            return Err(Denial::DuplicateIdentity);
        }
        super::journal::insert(&mut self.collected_steps, &coordinate, &step, request)
    }

    pub(in crate::durability) fn resolve_collected_history(
        &mut self,
        request: &ExecutionRequest<'_, '_>,
    ) -> Result<(), Denial> {
        // Creation always reserves a strictly later birth than its source;
        // an occurrence advances exactly one generation within that birth.
        // Sorting these existing identities is therefore a topological order,
        // not an assumption about out-of-order composite commit identities.
        for index in 0..self.collected_steps.len() {
            let step = *self
                .collected_steps
                .entry_at(index)
                .expect("retained step")
                .1;
            request.consult().map_err(|cause| {
                Denial::Medium(crate::durability::WorldDurableMediumDenial::Request(cause))
            })?;
            match step {
                CollectedWorldHistoryStep::BranchCreated(attempt) => {
                    self.admit_collected_creation(attempt, request)?
                }
                CollectedWorldHistoryStep::Occurrence(position) => {
                    let row = self
                        .collected_occurrences
                        .get(&position)
                        .expect("collected occurrence")
                        .clone();
                    self.admit_occurrence(
                        row.attempt,
                        row.owners,
                        row.publishing_incarnation,
                        row.position,
                        row.digest,
                        request,
                    )?;
                }
            }
        }
        self.collected_steps = Default::default();
        self.collected_occurrences = Default::default();
        Ok(())
    }

    fn admit_collected_creation(
        &mut self,
        attempt: DurableCompositePublicationAttemptIdentity,
        request: &ExecutionRequest<'_, '_>,
    ) -> Result<(), Denial> {
        let recorded = self
            .created
            .get(&attempt)
            .expect("collected creation")
            .clone();
        let created = recorded.created();
        let original = self
            .creations
            .get(&attempt)
            .ok_or(Denial::MissingCreationIntent)?;
        if original.intent() != &created.intent {
            return Err(Denial::ConflictingCreationIntent);
        }
        if self.creation_releases.contains_key(&attempt) {
            return Err(Denial::ConflictingDisposition);
        }
        let source = self
            .recorded_reference(&created.intent.source)
            .ok_or(Denial::MissingPredecessor)?;
        if source.origin_position() >= recorded.position() {
            return Err(Denial::BranchLineage);
        }
        if created
            .components
            .signal
            .point()
            .descriptor()
            .owner()
            .home()
            != self.binding.owner().home()
        {
            return Err(Denial::ForeignOwner);
        }
        let key = WorldBranchImageKey::new(
            recorded.record().clone(),
            recorded.position(),
            *recorded.digest(),
        );
        if self.branches.contains_key(&key) {
            return Err(Denial::BranchLineage);
        }
        let reuses = match created.intent.relational {
            Native::ReuseExact => created.intent.signal.is_reuse_exact(),
            Native::ForkExact { .. } => false,
        };
        if reuses {
            if !self
                .history
                .contains_key(&created.intent.destination.commit)
            {
                return Err(Denial::MissingPredecessor);
            }
        } else if self
            .history
            .contains_key(&created.intent.destination.commit)
        {
            return Err(Denial::DuplicateIdentity);
        }
        let head = WorldImageHead::branch(recorded.record().clone(), recorded.position());
        self.remember_reference(&head, request)?;
        super::journal::insert(&mut self.branches, &key, &(), request)?;
        super::journal::insert(&mut self.heads, &key, &head, request)?;
        if !reuses {
            super::journal::insert(
                &mut self.history,
                &created.intent.destination.commit,
                &head,
                request,
            )?;
        }
        Ok(())
    }
}
