//! Bind a source-readmitted effect to the shared original created-row read.
use worth_relational_branch::internal_port::durability::authority::associated_effects::created_row_read::{ReadmittedRelationalCreatedEntity, read_created_entity};
use super::ReadmittedRelationalAssociatedEffect;
use worth_relational_contracts::internal_port::durability::authority::associated_effects::refusal::RelationalAssociatedEffectRefusal as Refusal;

use worth_relational_contracts::internal_port::identity::data::record_ids::EntityId;

impl ReadmittedRelationalAssociatedEffect {
    /// Read the original creation delta, not a later mutable current row.
    /// This performs bounded admitted-artifact metadata work only; it opens
    /// no additional snapshot and cannot reconstruct a live result.
    pub fn created_entity(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        entity: EntityId,
    ) -> Result<Option<ReadmittedRelationalCreatedEntity<'_>>, Refusal> {
        read_created_entity(
            execution,
            self.semantic_result(),
            match &self.origin {
                super::ReadmittedEffectOrigin::Original(positioned) => {
                    &positioned.envelope().patch.authoritative_record_patches
                }
                super::ReadmittedEffectOrigin::Checkpoint { data, .. } => &data.creations,
            },
            entity,
        )
    }
}
