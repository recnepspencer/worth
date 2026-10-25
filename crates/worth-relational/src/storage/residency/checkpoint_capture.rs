use std::sync::{Arc, Weak};

use super::RelationalRegionResidency;
use crate::branch::RelationalBranchRootSchemaAuthority;
use crate::durability::data::{RelationalCheckpointMember, RelationalCheckpointRegion};
use crate::storage::data::{
    RelationalStorageSubject, RelationalStorageUnavailable,
    RelationalStorageUnavailableCause as Cause,
};
use crate::storage::overlay::PartitionContentCommitment;
use crate::symbols::data::StringInterner;

#[derive(Debug, Clone)]
pub(crate) struct RelationalCheckpointRegionPublication {
    cell: Weak<RelationalRegionResidency>,
    schema_digest: [u8; 32],
    leaf: RelationalCheckpointRegion,
}

impl RelationalCheckpointRegionPublication {
    pub(crate) fn acknowledge(&self) {
        if let Some(cell) = self.cell.upgrade() {
            cell.checkpoint_bindings
                .lock()
                .unwrap_or_else(|poisoned| poisoned.into_inner())
                .insert(self.schema_digest, self.leaf.clone());
        }
    }
}

pub(crate) struct RelationalCapturedRegionCheckpoint {
    pub(crate) leaf: RelationalCheckpointRegion,
    pub(crate) members: Vec<RelationalCheckpointMember>,
    pub(crate) publication: RelationalCheckpointRegionPublication,
}

impl RelationalRegionResidency {
    /// Capture admits no cold demand. Unchanged cells contribute only their
    /// directory leaves; changed cells must already be resident from the write.
    pub(crate) fn capture_checkpoint(
        self: &Arc<Self>,
        schema: &RelationalBranchRootSchemaAuthority,
        schema_digest: [u8; 32],
        symbols: &StringInterner,
    ) -> Result<RelationalCapturedRegionCheckpoint, RelationalStorageUnavailable> {
        let binding = self
            .checkpoint_bindings
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .get(&schema_digest)
            .cloned();
        let (leaf, members) = if let Some(leaf) = binding {
            let members = vec![
                RelationalCheckpointMember::Carried { locator: leaf.head },
                RelationalCheckpointMember::Carried {
                    locator: leaf.history,
                },
            ];
            (leaf, members)
        } else {
            let lease = self
                .lease_checkpoint_contents()?
                .ok_or_else(|| self.capture_failure(Cause::SourceUnavailable))?;
            let partition = lease.checkpoint_contents();
            let (commitment, _) = PartitionContentCommitment::capture(partition, symbols, None)
                .map_err(|_| self.capture_failure(Cause::IntegrityMismatch))?;
            let captured =
                super::capture_region_members(partition, &commitment, schema, schema_digest)
                    .map_err(|cause| self.capture_failure(cause))?;
            let leaf = RelationalCheckpointRegion {
                partition_id: partition.partition_id,
                content_digest: commitment.digest(partition.partition_id),
                entity_slots: partition.entity_arena.slot_count(),
                relation_slots: partition.relation_arena.slot_count(),
                unavailable_records: partition.entity_arena.lifecycle_counts().unavailable
                    + partition.relation_arena.lifecycle_counts().unavailable,
                head: captured.head.0,
                history: captured.history.0,
            };
            let members = [captured.head, captured.history]
                .into_iter()
                .map(|(locator, bytes)| RelationalCheckpointMember::New {
                    locator,
                    bytes: bytes.into(),
                })
                .collect();
            (leaf, members)
        };
        Ok(RelationalCapturedRegionCheckpoint {
            publication: RelationalCheckpointRegionPublication {
                cell: Arc::downgrade(self),
                schema_digest,
                leaf: leaf.clone(),
            },
            leaf,
            members,
        })
    }

    fn capture_failure(&self, cause: Cause) -> RelationalStorageUnavailable {
        RelationalStorageUnavailable {
            subject: RelationalStorageSubject::Region(self.partition_id),
            cause,
        }
    }
}
