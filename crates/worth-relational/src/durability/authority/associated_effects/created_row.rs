//! Bind a source-readmitted effect to the shared original created-row read.
use super::created_row_read::{read_created_entity, ReadmittedRelationalCreatedEntity};
use super::{ReadmittedRelationalAssociatedEffect, RelationalAssociatedEffectRefusal as Refusal};

use crate::identity::data::EntityId;

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
