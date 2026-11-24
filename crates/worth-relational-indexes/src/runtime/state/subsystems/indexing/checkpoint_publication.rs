//! The catalog settles authenticated indexes under its existing publication gates.
use super::{generation_publication_gate::GenerationPublicationPermit, *};
use std::sync::RwLockWriteGuard;
use worth_execution::ExecutionRequest;
use worth_relational_contracts::internal_port::execution::denial_cause::RelationalExecutionDenialCause;

pub struct CheckpointIndexCatalogReplacement<'a> {
    indexes: &'a IndexingSubsystem,
    _definitions: RwLockWriteGuard<'a, ()>,
    _generations: GenerationPublicationPermit,
}

impl IndexingSubsystem {
    pub fn prepare_checkpoint_replacement(&self) -> CheckpointIndexCatalogReplacement<'_> {
        CheckpointIndexCatalogReplacement {
            indexes: self,
            _definitions: self.definition_publication.write(),
            _generations: self.generation_publication.enter_uncontrolled(),
        }
    }
}

impl CheckpointIndexCatalogReplacement<'_> {
    pub fn install(&self, mut state: IndexingState) {
        Arc::make_mut(&mut state.catalog).renew_incarnation();
        *self.indexes.state.write() = state;
    }
}

pub struct PreparedCheckpointIndexMerge<'a> {
    binding: &'a IndexDefinitionReadBinding,
    _generations: GenerationPublicationPermit,
}

impl IndexDefinitionReadBinding {
    pub fn has_definitions(&self) -> bool {
        !self.state.read().catalog.definitions.is_empty()
    }

    pub fn prepare_checkpoint_merge(
        &self,
        execution: &ExecutionRequest<'_, '_>,
    ) -> Result<PreparedCheckpointIndexMerge<'_>, RelationalExecutionDenialCause> {
        Ok(PreparedCheckpointIndexMerge {
            binding: self,
            _generations: self
                .generation_publication
                .enter(|| worth_relational_contracts::internal_port::execution::request_custody::check_request(execution))?,
        })
    }
}

impl PreparedCheckpointIndexMerge<'_> {
    pub fn install(&self, admitted: &IndexingState) {
        self.binding.state.write().merge_checkpoint_image(admitted);
    }
}

impl IndexingState {
    /// A retained payload is not a publication at its authored version.
    pub(super) fn restore_generation_payload(&mut self, generation: DerivedIndexGeneration) {
        self.next_generation_id = self
            .next_generation_id
            .max(generation.generation_id.0.saturating_add(1));
        self.generations.insert(generation);
    }

    pub fn merge_checkpoint_image(&mut self, admitted: &Self) {
        for generation in admitted.generations.all() {
            self.restore_generation_payload(generation.as_ref().clone());
        }
        self.bases.merge(&admitted.bases);
    }
}
