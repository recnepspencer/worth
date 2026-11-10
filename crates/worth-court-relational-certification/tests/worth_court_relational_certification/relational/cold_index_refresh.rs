//! Selected index refresh uses Head while an independently refused History stays cold.
use std::sync::{atomic::Ordering, Arc};
use worth_execution::ExecutionRequest;
use worth_relational::facade::{
    durability::*, history::CommitId, identity::PartitionId, indexes::*,
    runtime::RelationalRuntime, storage::RelationalStorageUnavailableCause, transactions::*,
};
use worth_testkit_relational::{
    aspect_storage::publications::{
        begin_main_transaction, changed_entities, release_test_commit_snapshot,
    },
    durable_medium::{
        immutable_members::{ColdRead, ImmutableCheckpointMemberSource},
        TestDurableMedium,
    },
    durable_publication::{create_entity_outcome, durable_runtime, field_patch},
};

#[derive(serde::Deserialize)]
struct Checkpoint {
    manifest: Manifest,
}
#[derive(serde::Deserialize)]
struct Manifest {
    roots: Vec<Root>,
}
#[derive(serde::Deserialize)]
struct Root {
    commit_id: CommitId,
    regions: Vec<Region>,
}
#[derive(serde::Deserialize)]
struct Region {
    partition_id: PartitionId,
    head: RelationalDurableMemberLocator,
    history: RelationalDurableMemberLocator,
}

struct Reopened {
    runtime: RelationalRuntime,
    medium: Arc<TestDurableMedium>,
    source: Arc<ImmutableCheckpointMemberSource>,
    entity: worth_relational::facade::identity::EntityId,
    index: DerivedIndexId,
    generation: Arc<DerivedIndexGeneration>,
    manifest: Manifest,
}

fn update(
    execution: &ExecutionRequest<'_, '_>,
    runtime: &RelationalRuntime,
    entity: worth_relational::facade::identity::EntityId,
    value: &str,
) {
    let mut transaction = begin_main_transaction(runtime);
    transaction
        .push_batch(
            WorkerIntentBatch::new("index-current-value").push(MutationIntent::Entity(
                EntityMutationIntent::UpdateFields(UpdateEntityFieldsIntent {
                    entity_id: entity,
                    fields: field_patch("name", "name", value),
                }),
            )),
            execution.allocation_policy(),
        )
        .unwrap();
    let committed = transaction.commit(execution, runtime).unwrap();
    release_test_commit_snapshot(runtime, &committed);
}

fn reopened(execution: &ExecutionRequest<'_, '_>) -> Reopened {
    use sha2::{Digest, Sha256};
    let original_medium = TestDurableMedium::new();
    let producer = durable_runtime(&original_medium);
    let index = producer
        .index_authority()
        .register(DerivedIndexDefinition {
            index_id: DerivedIndexId(0),
            name: "matrix-name".into(),
            branch_scoped: true,
            kind: DerivedIndexKind::EntityField {
                field_locator: planned_single_field_locator(
                    worth_foundational::facade::AspectKey::new("name").unwrap(),
                    worth_foundational::facade::FieldKey::new("name").unwrap(),
                ),
            },
        })
        .unwrap()
        .index_id;
    let first = create_entity_outcome(execution, &producer, "before");
    let entity = changed_entities(&first)[0];
    release_test_commit_snapshot(&producer, &first);
    update(execution, &producer, entity, "indexed");
    let (_, basis) = producer
        .observe_branch(&producer.main_branch_identity())
        .unwrap();
    let built = producer.index_authority().build_for_basis(
        execution,
        DerivedIndexBuildRequest {
            source_commit_id: basis.observation().commit_id().unwrap(),
            branch_id: worth_relational::facade::history::BranchId("main".into()),
            index_ids: vec![index],
        },
        &basis,
    );
    assert!(built.failed_indexes.is_empty(), "{built:?}");
    assert!(built.execution_denial.is_none(), "{built:?}");
    let generation = producer
        .index_access()
        .read_latest_generation(
            execution,
            index,
            &worth_relational::facade::history::BranchId("main".into()),
        )
        .unwrap()
        .unwrap();
    drop(basis);
    update(execution, &producer, entity, "after-index");
    let checkpoint = producer
        .durability_authority()
        .native_checkpoint_with_policy(
            execution,
            execution.allocation_policy(),
            RelationalCheckpointArtifactPolicy::ExternalMembers {
                current_members: &[],
                source: None,
            },
        )
        .unwrap();
    let manifest = rmp_serde::from_slice::<Checkpoint>(
        checkpoint
            .bytes()
            .strip_prefix(b"WORTH-REL-CHK22\n")
            .unwrap(),
    )
    .unwrap()
    .manifest;
    let artifacts = original_medium.offered();
    let source = Arc::new(ImmutableCheckpointMemberSource {
        members: checkpoint
            .members()
            .iter()
            .map(|member| match member {
                RelationalCheckpointMember::New { locator, bytes } => (*locator, bytes.clone()),
                RelationalCheckpointMember::Carried { .. } => {
                    panic!("the first capture owns its members")
                }
            })
            .collect(),
        commits: artifacts
            .iter()
            .map(|artifact| (artifact.position(), artifact.as_bytes().to_vec()))
            .collect(),
        canonical_members: artifacts
            .iter()
            .map(|artifact| {
                (
                    RelationalDurableMemberLocator {
                        sha256: Sha256::digest(artifact.as_bytes()).into(),
                        length: artifact.as_bytes().len() as u64,
                    },
                    artifact.position(),
                )
            })
            .collect(),
        canonical_member_reads: Default::default(),
        journal_reads: Default::default(),
        damaged: Default::default(),
        reads: Default::default(),
        member_reads: Default::default(),
        commit_reads: Default::default(),
        commits_damaged: Default::default(),
        failed_member: Default::default(),
        failures: Default::default(),
    });
    let input = RelationalDurableRecoverySource::new(
        Some(RelationalDurableCheckpointSource::new(
            checkpoint.covered_position().unwrap(),
            checkpoint.covered_branch_transitions().unwrap(),
            checkpoint,
        )),
        Vec::new(),
        Vec::new(),
        source.clone(),
    );
    let medium = TestDurableMedium::new();
    for artifact in &artifacts {
        assert!(matches!(
            medium.attachment().publish(artifact),
            DurablePublicationFate::Committed
        ));
    }
    let mut runtime = durable_runtime(&medium);
    let plan = runtime
        .durability()
        .recovery_plan_from_source(
            execution,
            input,
            RecoveryVerificationMode::NormalRecoveryVerification,
        )
        .unwrap();
    runtime
        .durability_recovery()
        .recover(execution, plan)
        .unwrap();
    assert_eq!(source.reads.load(Ordering::Relaxed), 0);
    Reopened {
        runtime,
        medium,
        source,
        entity,
        index,
        generation,
        manifest,
    }
}

#[test]
fn cold_selected_index_refresh_preserves_budget_gate_and_reads_head_without_history() {
    let serial = worth_testkit_relational::shared_relational_certification_test_execution_policy::serial_request();
    let execution = &ExecutionRequest::serial(&serial);
    let fixture = reopened(execution);
    let runtime = &fixture.runtime;
    let (descriptor, basis) = runtime
        .observe_branch(&runtime.main_branch_identity())
        .unwrap();
    let commit = basis.observation().commit_id().unwrap();
    assert_ne!(commit, fixture.generation.source_commit_id);
    let selected = fixture
        .manifest
        .roots
        .iter()
        .find(|root| root.commit_id == commit)
        .unwrap();
    let region = selected
        .regions
        .iter()
        .find(|region| region.partition_id == fixture.entity.partition_id)
        .unwrap();
    let head = region.head;
    let history = region.history;
    fixture.source.failures.lock().unwrap().insert(
        ColdRead::Member(history),
        RelationalStorageUnavailableCause::MissingMember,
    );
    let position = runtime.history().latest_patch_stream_position();
    let offered = fixture.medium.offered().len();
    let request = DerivedIndexBuildRequest {
        source_commit_id: commit,
        branch_id: descriptor.branch_id().clone(),
        index_ids: vec![fixture.index],
    };
    let budget = DerivedIndexMaintenanceBudget {
        maximum_work_units: None,
        maximum_cold_record_slots: Some(0),
        maximum_derived_rows: None,
    };
    let denied = runtime
        .index_authority()
        .refresh_for_basis(request.clone(), &basis, None, budget, execution)
        .unwrap_err();
    assert_eq!(
        denied.kind,
        DerivedIndexMaintenanceDenialKind::ColdReconstructionRequired
    );
    assert!(!fixture.source.member_reads.lock().unwrap().contains(&head));
    assert!(!fixture
        .source
        .member_reads
        .lock()
        .unwrap()
        .contains(&history));
    let result = runtime
        .index_authority()
        .refresh_for_basis(
            request,
            &basis,
            None,
            DerivedIndexMaintenanceBudget {
                maximum_cold_record_slots: None,
                ..budget
            },
            execution,
        )
        .unwrap();
    let generation = result
        .generations
        .iter()
        .find(|generation| generation.index_id == fixture.index)
        .unwrap();
    assert_eq!(generation.source_commit_id, commit);
    assert_ne!(
        generation.entries, fixture.generation.entries,
        "current projection uses the changed field"
    );
    let DerivedIndexEntries::EntityField(entries) = &generation.entries else {
        panic!("the declared entity field index keeps its family");
    };
    assert!(entries
        .values()
        .any(|owners| owners.iter().any(|owner| owner == &fixture.entity)));
    assert!(fixture.source.member_reads.lock().unwrap().contains(&head));
    assert!(!fixture
        .source
        .member_reads
        .lock()
        .unwrap()
        .contains(&history));
    assert_eq!(runtime.history().latest_patch_stream_position(), position);
    assert_eq!(fixture.medium.offered().len(), offered);
}
