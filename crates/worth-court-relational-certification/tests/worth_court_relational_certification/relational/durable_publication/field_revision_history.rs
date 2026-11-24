//! Certify revision identity and field presence across native exact rooted reads.
use super::branch_history_publications::{delete_entity, fork, update_entity_on_branch};
use super::checkpoint_recovery_observation::snapshot_for_branch;
use worth_foundational::facade::{
    AspectFieldLocator, CanonicalFieldPath, LocatorAuthority, PortableAspectContractBasis,
    PortableAspectPatchOperation, PortableRecordAspectPatch,
};
use worth_relational::facade::{
    history::BranchId,
    identity::{KindId, PartitionId},
    runtime::RelationalFieldPresence,
    schema::DeclaredAspectContractBinding,
    transactions::{
        ApplyEntityAspectPatchIntent, CreateIntent, EntityMutationIntent, EntitySpec,
        MutationIntent, UpdateEntityFieldsIntent, WorkerIntentBatch,
    },
};
use worth_testkit_relational::{
    aspect_storage::{
        aspect_key,
        definition::entity_summary_struct_aspect,
        field_key,
        publications::{
            begin_main_transaction, changed_entities, release_test_commit_snapshot,
            string_aspect_field_patch,
        },
        AspectSchemaFixture,
    },
    durable_publication::{create_entity_outcome, memory_runtime},
    shared_relational_certification_test_execution_policy::serial_request,
};

fn summary_binding() -> DeclaredAspectContractBinding {
    let (binding, contract) =
        entity_summary_struct_aspect(aspect_key("summary"), field_key("summary"));
    DeclaredAspectContractBinding { binding, contract }
}

fn locator(aspect: &str, field: &str) -> AspectFieldLocator {
    AspectFieldLocator::new(
        LocatorAuthority::Authoritative,
        aspect_key(aspect),
        CanonicalFieldPath::single(field_key(field)),
    )
}

#[test]
fn exact_field_revisions_keep_unrelated_sibling_and_absence_stable() {
    let host_request = serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let runtime = AspectSchemaFixture {
        entity_aspects: vec![summary_binding()],
        ..AspectSchemaFixture::default()
    }
    .build_runtime();
    let mut create = begin_main_transaction(&runtime);
    create
        .push_batch(
            WorkerIntentBatch::new("field-revision-create").push(MutationIntent::Create(
                CreateIntent::Entity(EntitySpec {
                    partition_id: PartitionId::main(),
                    kind_id: KindId(1),
                    client_key: "field-revision".into(),
                    fields: string_aspect_field_patch([(
                        aspect_key("summary"),
                        field_key("title"),
                        "same",
                    )]),
                }),
            )),
            worth_execution::ExecutionAllocationPolicy::SystemAllocation,
        )
        .unwrap();
    let created = create.commit(execution, &runtime).unwrap();
    let entity = changed_entities(&created)[0];
    let title = locator("summary", "title");
    let status = locator("summary", "status");
    let read = |snapshot| {
        let view = runtime.read_truth().project_snapshot(snapshot).unwrap();
        (
            view.entity_field_revision(execution, entity, &title)
                .expect("read request remains live"),
            view.entity_field_revision(execution, entity, &status)
                .expect("read request remains live"),
        )
    };
    let before = read(&created.snapshot);
    assert_eq!(
        before.0.unwrap().presence(),
        RelationalFieldPresence::Present
    );
    assert_eq!(
        before.1.unwrap().presence(),
        RelationalFieldPresence::Absent
    );

    let mut update = begin_main_transaction(&runtime);
    update
        .push_batch(
            WorkerIntentBatch::new("field-revision-update").push(MutationIntent::Entity(
                EntityMutationIntent::UpdateFields(UpdateEntityFieldsIntent {
                    entity_id: entity,
                    fields: string_aspect_field_patch([(
                        aspect_key("summary"),
                        field_key("status"),
                        "ready",
                    )]),
                }),
            )),
            worth_execution::ExecutionAllocationPolicy::SystemAllocation,
        )
        .unwrap();
    let updated = update.commit(execution, &runtime).unwrap();
    let after = read(&updated.snapshot);
    assert_eq!(before.0, after.0);
    assert_ne!(before.1, after.1);
    assert_eq!(
        after.1.unwrap().presence(),
        RelationalFieldPresence::Present
    );
    assert_eq!(read(&created.snapshot), before);
    release_test_commit_snapshot(&runtime, &created);
    release_test_commit_snapshot(&runtime, &updated);
}

#[test]
fn absent_present_absent_aba_has_distinct_native_revision() {
    let host_request = serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let binding = summary_binding();
    let contract = binding.contract.clone();
    let runtime = AspectSchemaFixture {
        entity_aspects: vec![binding],
        ..AspectSchemaFixture::default()
    }
    .build_runtime();
    let mut create = begin_main_transaction(&runtime);
    create
        .push_batch(
            WorkerIntentBatch::new("aba-create").push(MutationIntent::Create(
                CreateIntent::Entity(EntitySpec {
                    partition_id: PartitionId::main(),
                    kind_id: KindId(1),
                    client_key: "aba".into(),
                    fields: string_aspect_field_patch([(
                        aspect_key("summary"),
                        field_key("title"),
                        "fixed",
                    )]),
                }),
            )),
            worth_execution::ExecutionAllocationPolicy::SystemAllocation,
        )
        .unwrap();
    let created = create.commit(execution, &runtime).unwrap();
    let entity = changed_entities(&created)[0];
    let status = locator("summary", "status");
    let title = locator("summary", "title");
    let revision = |snapshot| {
        runtime
            .read_truth()
            .project_snapshot(snapshot)
            .unwrap()
            .entity_field_revision(execution, entity, &status)
            .expect("read request remains live")
            .unwrap()
    };
    let initial = revision(&created.snapshot);
    assert_eq!(initial.presence(), RelationalFieldPresence::Absent);
    let title_initial = runtime
        .read_truth()
        .project_snapshot(&created.snapshot)
        .unwrap()
        .entity_field_revision(execution, entity, &title)
        .expect("read request remains live");

    let mut set = begin_main_transaction(&runtime);
    set.push_batch(
        WorkerIntentBatch::new("aba-set").push(MutationIntent::Entity(
            EntityMutationIntent::UpdateFields(UpdateEntityFieldsIntent {
                entity_id: entity,
                fields: string_aspect_field_patch([(
                    aspect_key("summary"),
                    field_key("status"),
                    "intermediate",
                )]),
            }),
        )),
        worth_execution::ExecutionAllocationPolicy::SystemAllocation,
    )
    .unwrap();
    let present = set.commit(execution, &runtime).unwrap();
    assert_eq!(
        revision(&present.snapshot).presence(),
        RelationalFieldPresence::Present
    );

    let mut clear = begin_main_transaction(&runtime);
    clear
        .push_batch(
            WorkerIntentBatch::new("aba-clear").push(MutationIntent::Entity(
                EntityMutationIntent::ApplyAspectPatch(ApplyEntityAspectPatchIntent {
                    entity_id: entity,
                    aspect_patch: PortableRecordAspectPatch::new([
                        PortableAspectPatchOperation::PatchFields {
                            basis: PortableAspectContractBasis::from_contract(&contract),
                            selected_fields: vec![field_key("status")],
                            field_sets: Vec::new(),
                            field_clears: vec![field_key("status")],
                        },
                    ]),
                }),
            )),
            worth_execution::ExecutionAllocationPolicy::SystemAllocation,
        )
        .unwrap();
    let absent_again = clear.commit(execution, &runtime).unwrap();
    let final_revision = revision(&absent_again.snapshot);
    assert_eq!(final_revision.presence(), RelationalFieldPresence::Absent);
    assert_ne!(initial, final_revision);
    assert_eq!(
        title_initial,
        runtime
            .read_truth()
            .project_snapshot(&absent_again.snapshot)
            .unwrap()
            .entity_field_revision(execution, entity, &title)
            .expect("read request remains live")
    );
    for outcome in [&created, &present, &absent_again] {
        release_test_commit_snapshot(&runtime, outcome);
    }
}

#[test]
fn forked_updates_preserve_the_original_field_revision_and_recreation_changes_identity() {
    let host_request = serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let runtime = memory_runtime();
    let created = create_entity_outcome(execution, &runtime, "first");
    let entity = changed_entities(&created)[0];
    let name = locator("name", "name");
    let original = runtime
        .read_truth()
        .project_snapshot(&created.snapshot)
        .unwrap()
        .entity_field_revision(execution, entity, &name)
        .expect("read request remains live")
        .unwrap();
    let sibling = BranchId("field-revision-fork".to_owned());
    fork(execution, &runtime, &sibling.0);
    let changed = update_entity_on_branch(execution, &runtime, entity, "second", sibling);
    let on_fork = runtime
        .read_truth()
        .project_snapshot(&changed.snapshot)
        .unwrap()
        .entity_field_revision(execution, entity, &name)
        .expect("read request remains live")
        .unwrap();
    assert_ne!(on_fork, original);
    assert_eq!(
        runtime
            .read_truth()
            .project_snapshot(&created.snapshot)
            .unwrap()
            .entity_field_revision(execution, entity, &name)
            .expect("read request remains live"),
        Some(original)
    );
    let main = snapshot_for_branch(&runtime, &BranchId("main".to_owned()));
    assert_eq!(
        runtime
            .read_truth()
            .project_snapshot(&main)
            .unwrap()
            .entity_field_revision(execution, entity, &name)
            .expect("read request remains live"),
        Some(original)
    );
    runtime.snapshots().release_snapshot(&main).unwrap();

    let deleted = delete_entity(execution, &runtime, entity);
    assert_eq!(
        runtime
            .read_truth()
            .project_snapshot(&deleted.snapshot)
            .unwrap()
            .entity_field_revision(execution, entity, &name)
            .expect("read request remains live"),
        None
    );
    let recreated = create_entity_outcome(execution, &runtime, "first");
    let new_entity = changed_entities(&recreated)[0];
    assert_ne!(new_entity, entity);
    assert_ne!(
        runtime
            .read_truth()
            .project_snapshot(&recreated.snapshot)
            .unwrap()
            .entity_field_revision(execution, new_entity, &name)
            .expect("read request remains live"),
        Some(original)
    );
    for outcome in [&created, &changed, &deleted, &recreated] {
        release_test_commit_snapshot(&runtime, outcome);
    }
}
