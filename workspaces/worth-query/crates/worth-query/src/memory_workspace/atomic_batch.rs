use std::collections::BTreeSet;

use worth_foundational::facade::PortableRecordAspectPatch;
use worth_relational::facade::{
    identity::PartitionId,
    symbols::ClientKey,
    transactions::{
        ApplyEntityAspectPatchIntent, CreateIntent, DeleteEntityIntent, EntityAspectCreateIntent,
        EntityMutationIntent, MutationIntent, RecordRef, WorkerIntentBatch,
    },
};

use super::{
    WorthQueryCommitIdentity, WorthQueryEntityIdentity, WorthQueryMemoryWorkspace,
    WorthQueryMutationDelta, WorthQueryMutationKind, WorthQueryMutationReceipt,
    WorthQueryWorkspaceError,
};

pub(crate) enum WorthQueryMemoryBatchMutation {
    Insert {
        patch: PortableRecordAspectPatch,
        touches: Vec<crate::runtime::WorthQueryAspectTouch>,
    },
    Update {
        entity: WorthQueryEntityIdentity,
        patch: PortableRecordAspectPatch,
        touches: Vec<crate::runtime::WorthQueryAspectTouch>,
    },
    Delete {
        entity: WorthQueryEntityIdentity,
        touches: Vec<crate::runtime::WorthQueryAspectTouch>,
    },
}

struct PreparedMutation {
    entity: Option<WorthQueryEntityIdentity>,
    kind: WorthQueryMutationKind,
    touches: Vec<crate::runtime::WorthQueryAspectTouch>,
}

type PreparedIntent = (MutationIntent, PreparedMutation);

impl WorthQueryMemoryWorkspace {
    pub(crate) fn apply_batch_atomically(
        &mut self,
        mutations: Vec<WorthQueryMemoryBatchMutation>,
    ) -> Result<Vec<WorthQueryMutationReceipt>, WorthQueryWorkspaceError> {
        if mutations.is_empty() {
            return Ok(Vec::new());
        }
        let insert_count = mutations
            .iter()
            .filter(|mutation| matches!(mutation, WorthQueryMemoryBatchMutation::Insert { .. }))
            .count();
        let next_key = self
            .next_client_key
            .checked_add(insert_count as u64)
            .ok_or_else(|| WorthQueryWorkspaceError::new("batch client-key space exhausted"))?;
        let (batch, prepared) = self.prepare_batch(mutations)?;
        let (result, snapshot) = self.commit_batch(batch)?;
        self.next_client_key = next_key;
        self.batch_receipts(result, snapshot, prepared)
    }

    fn prepare_batch(
        &self,
        mutations: Vec<WorthQueryMemoryBatchMutation>,
    ) -> Result<(WorkerIntentBatch, Vec<PreparedMutation>), WorthQueryWorkspaceError> {
        let mut insert_index = 0_u64;
        let mut batch = WorkerIntentBatch::new("query-memory-atomic-batch");
        let mut prepared = Vec::with_capacity(mutations.len());
        for mutation in mutations {
            let (intent, descriptor) = self.prepare_mutation(mutation, &mut insert_index)?;
            batch = batch.push(intent);
            prepared.push(descriptor);
        }
        Ok((batch, prepared))
    }

    fn prepare_mutation(
        &self,
        mutation: WorthQueryMemoryBatchMutation,
        insert_index: &mut u64,
    ) -> Result<PreparedIntent, WorthQueryWorkspaceError> {
        match mutation {
            WorthQueryMemoryBatchMutation::Insert { patch, touches } => {
                *insert_index += 1;
                Ok(self.prepare_insert(patch, touches, *insert_index))
            }
            WorthQueryMemoryBatchMutation::Update {
                entity,
                patch,
                touches,
            } => self.prepare_update(entity, patch, touches),
            WorthQueryMemoryBatchMutation::Delete { entity, touches } => {
                self.prepare_delete(entity, touches)
            }
        }
    }

    fn prepare_insert(
        &self,
        patch: PortableRecordAspectPatch,
        touches: Vec<crate::runtime::WorthQueryAspectTouch>,
        insert_index: u64,
    ) -> PreparedIntent {
        let intent =
            MutationIntent::Create(CreateIntent::EntityAspects(EntityAspectCreateIntent {
                partition_id: PartitionId::main(),
                kind_id: self.kind_id,
                client_key: ClientKey::raw(format!(
                    "{}:{}",
                    self.kind_name,
                    self.next_client_key + insert_index
                )),
                aspect_patch: patch,
            }));
        (
            intent,
            PreparedMutation::new(None, WorthQueryMutationKind::Created, touches),
        )
    }

    fn prepare_update(
        &self,
        entity: WorthQueryEntityIdentity,
        patch: PortableRecordAspectPatch,
        touches: Vec<crate::runtime::WorthQueryAspectTouch>,
    ) -> Result<PreparedIntent, WorthQueryWorkspaceError> {
        let entity_id = super::runtime_identity::entity_id_from_identity(entity.clone())?;
        self.ensure_entity_exists(entity_id)?;
        let intent = MutationIntent::Entity(EntityMutationIntent::ApplyAspectPatch(
            ApplyEntityAspectPatchIntent {
                entity_id,
                aspect_patch: patch,
            },
        ));
        Ok((
            intent,
            PreparedMutation::new(Some(entity), WorthQueryMutationKind::Updated, touches),
        ))
    }

    fn prepare_delete(
        &self,
        entity: WorthQueryEntityIdentity,
        touches: Vec<crate::runtime::WorthQueryAspectTouch>,
    ) -> Result<PreparedIntent, WorthQueryWorkspaceError> {
        let entity_id = super::runtime_identity::entity_id_from_identity(entity.clone())?;
        self.ensure_entity_exists(entity_id)?;
        let intent = MutationIntent::Entity(EntityMutationIntent::Delete(DeleteEntityIntent {
            entity_id,
        }));
        Ok((
            intent,
            PreparedMutation::new(Some(entity), WorthQueryMutationKind::Deleted, touches),
        ))
    }

    fn batch_receipts(
        &self,
        result: worth_relational::facade::transactions::CommitResult,
        snapshot_identity: super::WorthQuerySnapshotIdentity,
        mut prepared: Vec<PreparedMutation>,
    ) -> Result<Vec<WorthQueryMutationReceipt>, WorthQueryWorkspaceError> {
        self.attach_inserted_entities(&result, &mut prepared)?;
        let changed = changed_entity_ids(&result);
        let commit_identity =
            WorthQueryCommitIdentity::from_runtime_receipt_commit(result.commit.commit_id.0);
        Ok(prepared
            .into_iter()
            .map(|mutation| {
                self.receipt_from_prepared(mutation, &changed, &commit_identity, &snapshot_identity)
            })
            .collect())
    }

    fn attach_inserted_entities(
        &self,
        result: &worth_relational::facade::transactions::CommitResult,
        prepared: &mut [PreparedMutation],
    ) -> Result<(), WorthQueryWorkspaceError> {
        let known = known_entity_ids(prepared)?;
        let mut inserted = result
            .changed_records
            .iter()
            .filter_map(|record| match record {
                RecordRef::Entity(entity) if !known.contains(entity) => {
                    Some(super::runtime_identity::entity_identity(*entity))
                }
                _ => None,
            });
        for mutation in prepared
            .iter_mut()
            .filter(|mutation| mutation.entity.is_none())
        {
            mutation.entity = inserted.next();
        }
        if inserted.next().is_some() || prepared.iter().any(|mutation| mutation.entity.is_none()) {
            return Err(WorthQueryWorkspaceError::new(
                "atomic batch commit did not return one identity per inserted row",
            ));
        }
        Ok(())
    }

    fn receipt_from_prepared(
        &self,
        mutation: PreparedMutation,
        changed: &BTreeSet<worth_relational::facade::identity::EntityId>,
        commit_identity: &WorthQueryCommitIdentity,
        snapshot_identity: &super::WorthQuerySnapshotIdentity,
    ) -> WorthQueryMutationReceipt {
        let entity = mutation
            .entity
            .expect("insert identities were admitted above");
        let entity_id = super::runtime_identity::entity_id_from_identity(entity.clone())
            .expect("prepared batch identity remains current");
        let deltas =
            if changed.contains(&entity_id) || mutation.kind == WorthQueryMutationKind::Deleted {
                vec![WorthQueryMutationDelta::from_collection_identity(
                    self.mutation_delta_collection_identity(),
                    entity,
                    mutation.kind,
                    mutation.touches,
                )]
            } else {
                Vec::new()
            };
        WorthQueryMutationReceipt {
            commit_identity: commit_identity.clone(),
            snapshot_identity: snapshot_identity.clone(),
            deltas,
            bridge_authority: None,
        }
    }
}

fn known_entity_ids(
    prepared: &[PreparedMutation],
) -> Result<BTreeSet<worth_relational::facade::identity::EntityId>, WorthQueryWorkspaceError> {
    prepared
        .iter()
        .filter_map(|mutation| mutation.entity.as_ref())
        .map(|identity| super::runtime_identity::entity_id_from_identity(identity.clone()))
        .collect()
}

fn changed_entity_ids(
    result: &worth_relational::facade::transactions::CommitResult,
) -> BTreeSet<worth_relational::facade::identity::EntityId> {
    result
        .changed_records
        .iter()
        .filter_map(|record| match record {
            RecordRef::Entity(entity) => Some(*entity),
            RecordRef::Relation(_) => None,
        })
        .collect()
}

impl PreparedMutation {
    fn new(
        entity: Option<WorthQueryEntityIdentity>,
        kind: WorthQueryMutationKind,
        touches: Vec<crate::runtime::WorthQueryAspectTouch>,
    ) -> Self {
        Self {
            entity,
            kind,
            touches,
        }
    }
}
