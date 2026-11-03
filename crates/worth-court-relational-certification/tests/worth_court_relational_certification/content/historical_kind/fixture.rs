//! Generic records drive only existing public Native transaction doors.
use crate::leased_preparation::ordinary_context::support;
use worth_execution::ExecutionRequest;
use worth_relational::facade::{
    identity::{EntityId, KindId, PartitionId, RelationId},
    mvcc::*,
    runtime::*,
    symbols::ClientKey,
    transactions::*,
};

pub(super) struct World {
    pub runtime: RelationalRuntime,
    pub entities: Vec<EntityId>,
    pub relations: Vec<RelationId>,
}

pub(super) fn runtime() -> RelationalRuntime {
    use worth_relational::facade::config::{
        MvccConfig, RelationalRuntimeProfile, RetentionBackend, SnapshotReleasePolicy,
    };
    RelationalRuntimeApi::builder()
        .profile(RelationalRuntimeProfile::CertificationCore)
        .mvcc(MvccConfig {
            track_visibility_metadata: true,
            snapshot_release_policy: SnapshotReleasePolicy::ExplicitRelease,
            auto_reclaim_deleted_records: true,
            reclaim_batch_size: 8,
            retention_backend: RetentionBackend::EpochChunkRetention,
        })
        .schema_registry(support::demo_schema_registry())
        .build()
        .unwrap()
}

pub(super) fn snapshot(
    runtime: &RelationalRuntime,
) -> worth_relational::facade::snapshots::SnapshotHandle {
    let (_, basis) = runtime
        .observe_branch(&runtime.main_branch_identity())
        .unwrap();
    runtime
        .snapshots()
        .snapshot_for_observation(&basis.observation())
        .unwrap()
}

impl World {
    pub fn new() -> Self {
        Self {
            runtime: runtime(),
            entities: vec![],
            relations: vec![],
        }
    }

    pub fn commit(
        &self,
        execution: &ExecutionRequest<'_, '_>,
        intent: MutationIntent,
    ) -> Result<CommitResult, TransactionCommitError> {
        let (_, basis) = self
            .runtime
            .observe_branch(&self.runtime.main_branch_identity())
            .unwrap();
        let mut tx = self
            .runtime
            .begin_branch_transaction(&basis, RelationalTransactionIntent::ordinary())
            .unwrap();
        tx.push_batch(
            WorkerIntentBatch::new("kind-history").push(intent),
            execution.allocation_policy(),
        )
        .unwrap();
        tx.commit(execution, &self.runtime)
    }

    pub fn release(&self, commit: &CommitResult) {
        self.runtime
            .snapshots()
            .release_snapshot(&commit.snapshot)
            .unwrap();
    }

    pub fn entity_spec(partition: u32, label: &str) -> EntitySpec {
        EntitySpec {
            partition_id: PartitionId(partition),
            kind_id: KindId(1),
            client_key: ClientKey::raw(label),
            fields: support::string_field_patch(
                support::aspect_key("name"),
                support::field_key("name"),
                label,
            ),
        }
    }

    pub fn create(&mut self, execution: &ExecutionRequest<'_, '_>, partition: u32, label: &str) {
        let commit = self
            .commit(
                execution,
                MutationIntent::Create(CreateIntent::Entity(Self::entity_spec(partition, label))),
            )
            .unwrap();
        self.entities
            .push(support::changed_entity(&commit).unwrap());
        self.release(&commit);
    }

    pub fn replace(
        &mut self,
        execution: &ExecutionRequest<'_, '_>,
        index: usize,
        partition: u32,
        label: &str,
    ) {
        let index = index.min(self.entities.len() - 1);
        let commit = self
            .commit(
                execution,
                MutationIntent::Entity(EntityMutationIntent::Replace(ReplaceEntityIntent {
                    entity_id: self.entities[index],
                    replacement: Self::entity_spec(partition, label),
                })),
            )
            .unwrap();
        self.release(&commit);
        self.refresh(execution);
    }

    pub fn relation_intent(
        &self,
        source: usize,
        target: usize,
        partition: u32,
        label: &str,
    ) -> MutationIntent {
        MutationIntent::Create(CreateIntent::Relation(RelationSpec {
            partition_id: PartitionId(partition),
            kind_id: KindId(2),
            client_key: ClientKey::raw(label),
            source: EntityReference::Existing(self.entities[source.min(self.entities.len() - 1)]),
            target: EntityReference::Existing(self.entities[target.min(self.entities.len() - 1)]),
            fields: support::string_field_patch(
                support::aspect_key("label"),
                support::field_key("label"),
                label,
            ),
        }))
    }

    pub fn relate(
        &mut self,
        execution: &ExecutionRequest<'_, '_>,
        source: usize,
        target: usize,
        partition: u32,
        label: &str,
    ) {
        let commit = self
            .commit(
                execution,
                self.relation_intent(source, target, partition, label),
            )
            .unwrap();
        self.relations
            .push(support::changed_relation(&commit).unwrap());
        self.release(&commit);
    }

    pub fn delete_entity(&mut self, execution: &ExecutionRequest<'_, '_>, index: usize) {
        let index = index.min(self.entities.len() - 1);
        let commit = self
            .commit(
                execution,
                MutationIntent::Entity(EntityMutationIntent::Delete(DeleteEntityIntent {
                    entity_id: self.entities[index],
                })),
            )
            .unwrap();
        self.release(&commit);
        self.refresh(execution);
    }

    pub fn delete_relation(&mut self, execution: &ExecutionRequest<'_, '_>, index: usize) {
        let index = index.min(self.relations.len() - 1);
        let commit = self
            .commit(
                execution,
                MutationIntent::Relation(RelationMutationIntent::Delete(DeleteRelationIntent {
                    relation_id: self.relations[index],
                })),
            )
            .unwrap();
        self.release(&commit);
        self.refresh(execution);
    }

    pub fn refresh(&mut self, execution: &ExecutionRequest<'_, '_>) {
        let snapshot = snapshot(&self.runtime);
        let read = self
            .runtime
            .read_truth()
            .read_snapshot(&snapshot, execution)
            .unwrap()
            .unwrap();
        self.entities = read.entities().iter().map(|r| r.entity_id).collect();
        self.relations = read.relations().iter().map(|r| r.relation_id).collect();
        drop(read);
        self.runtime
            .snapshots()
            .release_snapshot(&snapshot)
            .unwrap();
    }
}
