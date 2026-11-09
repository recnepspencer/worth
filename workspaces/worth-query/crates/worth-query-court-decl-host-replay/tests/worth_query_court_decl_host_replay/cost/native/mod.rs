mod reads;
mod schema;
use super::observation::*;
use worth_execution::{CancellationSource, ExecutionRequest, ExecutionWorkCeiling, SerialRequest};
use worth_foundational::{
    DeterminismContract, ExecutionBudget, ExecutionPosture, ExecutionRequestPolicy,
};
use worth_relational::facade::{
    history::BranchId, identity::*, indexes::*, inspection::*, runtime::*, symbols::ClientKey,
    transactions::*,
};
use worth_testkit_relational::aspect_storage::publications::{
    begin_main_transaction, release_test_commit_snapshot,
};

pub fn request() -> SerialRequest {
    SerialRequest::from_policy(
        &ExecutionRequestPolicy::new(
            ExecutionPosture::Serial,
            DeterminismContract::CanonicalBitwise,
            ExecutionBudget::new(
                std::num::NonZeroUsize::MIN,
                Some(1024 * 1024 * 1024),
                Some(1_000_000_000),
            ),
        ),
        CancellationSource::new().token(),
        None,
    )
}

struct World {
    runtime: RelationalRuntime,
    entities: Vec<EntityId>,
    relation: RelationId,
    indexes: Vec<DerivedIndexId>,
    head: CommitResult,
    history_pin: worth_relational::facade::snapshots::SnapshotHandle,
}

impl World {
    fn build(execution: &ExecutionRequest<'_, '_>, d: Dimensions) -> Self {
        let (runtime, indexes) = schema::runtime(d.installed);
        let mut batch = WorkerIntentBatch::new("scale-model");
        let mut keys = Vec::new();
        for i in 0..d.records {
            let spec = EntitySpec {
                partition_id: PartitionId(1 + (i % 2) as u32),
                kind_id: KindId(1),
                client_key: ClientKey::raw(format!("record-{i}")),
                fields: schema::patch("name", &format!("record-{i}")),
            };
            keys.push(CreatedEntityRef {
                partition_id: spec.partition_id,
                kind_id: spec.kind_id,
                client_key: spec.client_key.clone(),
            });
            batch = batch.push(MutationIntent::Create(CreateIntent::Entity(spec)));
        }
        let extra = AspectFieldPatch::new(
            (0..d.installed)
                .map(|i| {
                    (
                        schema::locator(&format!("bystander-{i}")),
                        worth_foundational::facade::AspectValue::String("unrelated".into()),
                    )
                })
                .collect(),
        );
        batch = batch.push(MutationIntent::Create(CreateIntent::Entity(EntitySpec {
            partition_id: PartitionId(3),
            kind_id: KindId(3),
            client_key: ClientKey::raw("bystander"),
            fields: extra,
        })));
        let created = commit(&runtime, execution, batch);
        let entities: Vec<_> = keys
            .iter()
            .map(|key| created.created_entity(key).unwrap())
            .collect();
        release_test_commit_snapshot(&runtime, &created);
        let mut batch = WorkerIntentBatch::new("scale-relations");
        let mut first = None;
        for i in (0..d.records).step_by(2) {
            let key = ClientKey::raw(format!("edge-{i}"));
            first.get_or_insert(CreatedRelationRef {
                partition_id: PartitionId(1),
                kind_id: KindId(2),
                client_key: key.clone(),
                source: EntityReference::Existing(entities[i]),
                target: EntityReference::Existing(entities[i + 1]),
            });
            batch = batch.push(MutationIntent::Create(CreateIntent::Relation(
                RelationSpec {
                    partition_id: PartitionId(1),
                    kind_id: KindId(2),
                    client_key: key,
                    source: EntityReference::Existing(entities[i]),
                    target: EntityReference::Existing(entities[i + 1]),
                    fields: schema::patch("label", "edge"),
                },
            )));
        }
        batch = batch.push(MutationIntent::Create(CreateIntent::Relation(
            RelationSpec {
                partition_id: PartitionId(2),
                kind_id: KindId(2),
                client_key: ClientKey::raw("join-bridge"),
                source: EntityReference::Existing(entities[1]),
                target: EntityReference::Existing(entities[2]),
                fields: schema::patch("label", "bridge"),
            },
        )));
        let mut head = commit(&runtime, execution, batch);
        let relation = head.created_relation(&first.unwrap()).unwrap();
        let history_relation = head
            .created_relation(&CreatedRelationRef {
                partition_id: PartitionId(1),
                kind_id: KindId(2),
                client_key: ClientKey::raw("edge-2"),
                source: EntityReference::Existing(entities[2]),
                target: EntityReference::Existing(entities[3]),
            })
            .unwrap();
        let (_, initial) = runtime
            .observe_branch(&runtime.main_branch_identity())
            .unwrap();
        let history_pin = runtime
            .snapshots()
            .snapshot_for_observation(&initial.observation())
            .unwrap();
        // One explicit cold initialization at the setup trust boundary. The
        // history below follows the ordinary indexed commit path, so H grows
        // the real generation catalog as well as canonical record history.
        let built = runtime
            .index_authority()
            .build_for_commit(execution, build_request(&indexes, &head));
        assert!(
            built.failed_indexes.is_empty(),
            "initial index build: {built:?}"
        );
        for i in 0..d.history {
            let next = indexed_commit(
                &runtime,
                execution,
                update(entities[2], &format!("history-{i}")).push(MutationIntent::Relation(
                    RelationMutationIntent::UpdateEndpoints(UpdateRelationEndpointsIntent {
                        relation_id: history_relation,
                        kind_id: KindId(2),
                        source: EntityReference::Existing(entities[2]),
                        target: EntityReference::Existing(entities[if i % 2 == 0 { 5 } else { 3 }]),
                    }),
                )),
                &indexes,
                &head,
            );
            release_test_commit_snapshot(&runtime, &head);
            head = next;
        }
        Self {
            runtime,
            entities,
            relation,
            indexes,
            head,
            history_pin,
        }
    }

    fn measure(self, execution: &ExecutionRequest<'_, '_>, d: Dimensions) -> Sample {
        let mut sample = Sample::new(d);
        let edits = update(self.entities[d.records - 1], "edited").push(MutationIntent::Relation(
            RelationMutationIntent::UpdateEndpoints(UpdateRelationEndpointsIntent {
                relation_id: self.relation,
                kind_id: KindId(2),
                source: EntityReference::Existing(self.entities[0]),
                target: EntityReference::Existing(self.entities[3]),
            }),
        ));
        let mut transaction = begin_main_transaction(&self.runtime);
        transaction
            .push_batch(edits, execution.allocation_policy())
            .unwrap();
        let (((result, observation), report), schema_work) = worth_relational::facade::schema::RelationalSchemaRegistry::observe_authority_projection_work(|| execution.run(ExecutionWorkCeiling::new(None), |_| observe_maintenance_work(64, || {
            let (candidate, prepare) = execution.run(ExecutionWorkCeiling::new(None), |_| self.runtime.prepare_branch_transaction(execution, transaction).unwrap()).unwrap();
            sample.record(Phase::CommitPreparation, "charged_work", prepare.charged_work());
            let work = candidate.prepared_index_maintenance_work().expect("native receipt");
            sample.record(Phase::IndexMaintenance, "entry_edits", work.entry_edits as u64);
            sample.record(Phase::IndexMaintenance, "record_reads", work.record_reads as u64);
            let (result, publication) = execution.run(ExecutionWorkCeiling::new(None), |_| {
                let worth_relational::facade::mvcc::RelationalPublicationOutcome::Performed(performed) = self.runtime.publication_port().compare_and_publish(execution, candidate) else { panic!("candidate must publish") };
                self.runtime.settle_performed_publication(performed).unwrap()
            }).unwrap();
            sample.record(Phase::Publication, "charged_work", publication.charged_work());
            result
        })).unwrap());
        sample.record(Phase::Invariants, "schema_projection_rows", schema_work);
        sample.record(Phase::WholeCommit, "charged_work", report.charged_work());
        sample.observe_phase_work(&observation);
        sample.record(
            Phase::RootAdmission,
            "content_index_hash_bytes",
            observation.content_index_hash_bytes,
        );
        sample.record(
            Phase::RootAdmission,
            "partition_value_hash_bytes",
            observation.partition_value_hash_bytes,
        );
        sample.record(
            Phase::Publication,
            "commit_payload_hash_bytes",
            observation.commit_payload_hash_bytes,
        );
        sample.commits = 1;
        sample.touched.push(result.changed_records.len());
        let counters = &result.execution().complexity_delta;
        sample.record(
            Phase::Invariants,
            "entity_scans",
            counters.invariant_entity_slot_scans as u64,
        );
        sample.record(
            Phase::Invariants,
            "relation_scans",
            counters.invariant_relation_slot_scans as u64,
        );
        sample.record(
            Phase::Invariants,
            "executions",
            result
                .commit_log()
                .events()
                .iter()
                .filter(|event| matches!(event, CommitTraceEvent::InvariantEvaluated { .. }))
                .count() as u64,
        );
        sample.record(
            Phase::Publication,
            "live_history_visits",
            (counters.live_history_slots_visited + counters.live_history_entries_visited) as u64,
        );
        reads::verify(
            &self.runtime,
            execution,
            &result,
            &self.indexes,
            self.entities[d.records - 1],
            &mut sample,
        );
        release_test_commit_snapshot(&self.runtime, &self.head);
        release_test_commit_snapshot(&self.runtime, &result);
        self.runtime
            .snapshots()
            .release_snapshot(&self.history_pin)
            .unwrap();
        sample
    }
}

fn commit(
    runtime: &RelationalRuntime,
    execution: &ExecutionRequest<'_, '_>,
    batch: WorkerIntentBatch,
) -> CommitResult {
    let mut transaction = begin_main_transaction(runtime);
    transaction
        .push_batch(batch, execution.allocation_policy())
        .unwrap();
    transaction.commit(execution, runtime).unwrap()
}
fn update(entity_id: EntityId, value: &str) -> WorkerIntentBatch {
    WorkerIntentBatch::new("fixed-scalar-edit").push(MutationIntent::Entity(
        EntityMutationIntent::UpdateFields(UpdateEntityFieldsIntent {
            entity_id,
            fields: schema::patch("name", value),
        }),
    ))
}
fn indexed_commit(
    runtime: &RelationalRuntime,
    execution: &ExecutionRequest<'_, '_>,
    batch: WorkerIntentBatch,
    _indexes: &[DerivedIndexId],
    _before: &CommitResult,
) -> CommitResult {
    let mut transaction = begin_main_transaction(runtime);
    transaction
        .push_batch(batch, execution.allocation_policy())
        .unwrap();
    let candidate = runtime
        .prepare_branch_transaction(execution, transaction)
        .unwrap();
    assert_eq!(
        candidate
            .prepared_index_maintenance_work()
            .unwrap()
            .cold_record_slots,
        0
    );
    let worth_relational::facade::mvcc::RelationalPublicationOutcome::Performed(performed) =
        runtime
            .publication_port()
            .compare_and_publish(execution, candidate)
    else {
        panic!("indexed history must publish")
    };
    runtime.settle_performed_publication(performed).unwrap()
}
fn build_request(indexes: &[DerivedIndexId], head: &CommitResult) -> DerivedIndexBuildRequest {
    DerivedIndexBuildRequest {
        source_commit_id: head.commit.commit_id,
        branch_id: BranchId("main".into()),
        index_ids: indexes.to_vec(),
    }
}

#[test]
fn native_fixed_edit_exposes_each_independent_growth_axis() {
    let host = request();
    let execution = ExecutionRequest::serial(&host);
    let samples = (Dimensions {
        records: 128,
        history: 64,
        installed: 8,
    })
    .points()
    .map(|d| World::build(&execution, d).measure(&execution, d));
    for sample in &samples {
        eprintln!("NATIVE_SCALE {sample:?}");
    }
    super::ratchet::verify("native", &samples);
}
