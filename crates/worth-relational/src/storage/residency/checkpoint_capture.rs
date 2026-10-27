use std::sync::{Arc, Weak};

use super::RelationalRegionResidency;
use crate::branch::RelationalBranchRootSchemaAuthority;
use crate::durability::data::{
    RelationalCheckpointHistoryDependency, RelationalHistoryMemberDirectory,
};
use crate::durability::data::{RelationalCheckpointMember, RelationalCheckpointRegion};
use crate::storage::data::{
    RelationalStorageSubject, RelationalStorageUnavailable,
    RelationalStorageUnavailableCause as Cause,
};
use crate::storage::overlay::{PartitionContentCommitment, PartitionContentDigestError};
use crate::symbols::data::StringInterner;

#[derive(Debug, Clone)]
pub(super) struct RecordedRegionCheckpoint {
    pub(super) leaf: RelationalCheckpointRegion,
    pub(super) history_members: Arc<RelationalHistoryMemberDirectory>,
}

#[derive(Debug, Clone)]
pub(crate) struct RelationalCheckpointRegionPublication {
    cell: Weak<RelationalRegionResidency>,
    schema_digest: [u8; 32],
    recorded: RecordedRegionCheckpoint,
}

impl RelationalCheckpointRegionPublication {
    pub(crate) fn acknowledge(&self) {
        if let Some(cell) = self.cell.upgrade() {
            cell.checkpoint_bindings
                .lock()
                .unwrap_or_else(|poisoned| poisoned.into_inner())
                .insert(self.schema_digest, self.recorded.clone());
        }
    }
}

pub(crate) struct RelationalCapturedRegionCheckpoint {
    pub(crate) leaf: RelationalCheckpointRegion,
    pub(crate) members: Vec<RelationalCheckpointMember>,
    pub(crate) publication: RelationalCheckpointRegionPublication,
    pub(crate) history_dependencies: Vec<RelationalCheckpointHistoryDependency>,
}

impl RelationalRegionResidency {
    /// Capture admits no cold demand. Unchanged cells contribute only their
    /// directory leaves; changed cells must already be resident from the write.
    pub(crate) fn capture_checkpoint(
        self: &Arc<Self>,
        schema: &RelationalBranchRootSchemaAuthority,
        schema_digest: [u8; 32],
        symbols: &StringInterner,
        current_members: Option<
            &std::collections::BTreeSet<crate::durability::medium::RelationalDurableMemberLocator>,
        >,
    ) -> Result<RelationalCapturedRegionCheckpoint, RelationalStorageUnavailable> {
        self.check_damage()?;
        let binding = self
            .checkpoint_bindings
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .get(&schema_digest)
            .filter(|recorded| {
                current_members.is_none_or(|members| {
                    members.contains(&recorded.leaf.head)
                        && recorded
                            .history_members
                            .members(recorded.leaf.history)
                            .iter()
                            .all(|member| members.contains(member))
                })
            })
            .cloned();
        let (recorded, members) = if let Some(recorded) = binding {
            let mut required = recorded.history_members.members(recorded.leaf.history);
            required.insert(recorded.leaf.head);
            let members = required
                .into_iter()
                .map(|locator| RelationalCheckpointMember::Carried { locator })
                .collect();
            (recorded, members)
        } else {
            let lease = self
                .lease_checkpoint_contents()?
                .ok_or_else(|| self.capture_failure(Cause::SourceUnavailable))?;
            let partition = lease.checkpoint_contents();
            let (commitment, _) = PartitionContentCommitment::capture_layered(partition, symbols)
                .map_err(|error| match error {
                PartitionContentDigestError::UnresolvedContentSymbol(symbol) => {
                    RelationalStorageUnavailable {
                        subject: RelationalStorageSubject::Symbols {
                            first: symbol,
                            last: symbol,
                        },
                        cause: Cause::SourceUnavailable,
                    }
                }
                PartitionContentDigestError::InvalidHistoryCoverage => {
                    self.capture_failure(Cause::IntegrityMismatch)
                }
            })?;
            let captured =
                super::capture_region_members(partition, &commitment, schema, schema_digest)
                    .map_err(|cause| self.capture_failure(cause))?;
            let leaf = RelationalCheckpointRegion {
                kind_authority: captured.kind_authority,
                partition_id: partition.partition_id,
                content_digest: commitment.digest(partition.partition_id),
                entity_slots: partition.entity_arena.slot_count(),
                relation_slots: partition.relation_arena.slot_count(),
                unavailable_records: partition.entity_arena.lifecycle_counts().unavailable
                    + partition.relation_arena.lifecycle_counts().unavailable,
                head: captured.head.0,
                history: captured.history.0,
            };
            let required = captured.history_members.members(captured.history.0);
            let mut members = Vec::new();
            for locator in required
                .into_iter()
                .filter(|locator| *locator != captured.history.0)
            {
                if current_members.is_some_and(|members| !members.contains(&locator)) {
                    return Err(self.capture_failure(Cause::SourceUnavailable));
                }
                members.push(RelationalCheckpointMember::Carried { locator });
            }
            members.extend([captured.head, captured.history].into_iter().map(
                |(locator, bytes)| RelationalCheckpointMember::New {
                    locator,
                    bytes: bytes.into(),
                },
            ));
            (
                RecordedRegionCheckpoint {
                    leaf,
                    history_members: captured.history_members,
                },
                members,
            )
        };
        let required = recorded.history_members.members(recorded.leaf.history);
        let history_dependencies = recorded.history_members.declarations_for(&required);
        Ok(RelationalCapturedRegionCheckpoint {
            publication: RelationalCheckpointRegionPublication {
                cell: Arc::downgrade(self),
                schema_digest,
                recorded: recorded.clone(),
            },
            leaf: recorded.leaf,
            members,
            history_dependencies,
        })
    }

    fn capture_failure(&self, cause: Cause) -> RelationalStorageUnavailable {
        RelationalStorageUnavailable {
            subject: RelationalStorageSubject::Region(self.partition_id),
            cause,
        }
    }
}
