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
        for i in 0..d.history {
            let next = commit(
                &runtime,
                execution,
                update(entities[2], &format!("history-{i}")),
            );
            release_test_commit_snapshot(&runtime, &head);
            head = next;
        }
        // This explicit initial cold build is fixture setup, outside every
        // observed ordinary interval. Every measured edit has a real predecessor.
        let built = runtime
            .index_authority()
            .build_for_commit(execution, build_request(&indexes, &head));
        assert!(
            built.failed_indexes.is_empty(),
            "initial index build: {built:?}"
        );
        Self {
            runtime,
            entities,
            relation,
            indexes,
            head,
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
        let ((result, observation), report) = execution.run(ExecutionWorkCeiling::new(None), |_| observe_maintenance_work(64, || {
            let (mut candidate, prepare) = execution.run(ExecutionWorkCeiling::new(None), |_| self.runtime.prepare_branch_transaction(execution, transaction).unwrap()).unwrap();
            sample.record(Phase::CommitPreparation, "charged_work", prepare.charged_work());
            let (work, index_report) = execution.run(ExecutionWorkCeiling::new(None), |_| self.runtime.index_authority().prepare_incremental_indexes_for_candidate(
                &mut candidate, &self.indexes, &self.head.snapshot, DerivedIndexMaintenanceBudget {
                    maximum_work_units: None, maximum_cold_record_slots: Some(0), maximum_derived_rows: None,
                }, execution).unwrap()).unwrap();
            sample.record(Phase::IndexMaintenance, "charged_work", index_report.charged_work());
            sample.record(Phase::IndexMaintenance, "entry_edits", work.entry_edits as u64);
            sample.record(Phase::IndexMaintenance, "record_reads", work.record_reads as u64);
            let (result, publication) = execution.run(ExecutionWorkCeiling::new(None), |_| {
                let worth_relational::facade::mvcc::RelationalPublicationOutcome::Performed(performed) = self.runtime.publication_port().compare_and_publish(execution, candidate) else { panic!("candidate must publish") };
                self.runtime.settle_performed_publication(performed).unwrap()
            }).unwrap();
            sample.record(Phase::Publication, "charged_work", publication.charged_work());
            result
        })).unwrap();
        sample.record(Phase::WholeCommit, "charged_work", report.charged_work());
        sample.observe_phase_work(&observation);
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
        sample
    }
}

impl PhaseWorkSource for MaintenanceWorkObservation {
    fn phase_work(&self) -> Vec<PhaseWork> {
        assert_eq!(self.omitted_phases, 0, "phase evidence must be complete");
        [
            ("root_admission", Phase::RootAdmission),
            ("kind_commitment", Phase::KindCommitment),
        ]
        .map(|(name, phase)| {
            let rows: Vec<_> = self.phases.iter().filter(|row| row.phase == name).collect();
            assert!(
                !rows.is_empty() && rows.iter().all(|row| row.accounted),
                "missing accounted {name}: {self:?}"
            );
            PhaseWork {
                phase,
                charged_work: rows.iter().map(|row| row.charged_work).sum(),
            }
        })
        .into()
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
