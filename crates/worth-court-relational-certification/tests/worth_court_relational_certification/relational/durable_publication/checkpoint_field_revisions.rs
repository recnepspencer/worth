//! Exact field revision identity and presence survive public checkpoint recovery.
use super::branch_history_publications::{
    changed_relations, create_relation_outcome, create_with_fields, fork,
};
use super::checkpoint_recovery_observation::snapshot_for_branch;
use worth_foundational::facade::{
    AspectFieldLocator, CanonicalFieldPath, LocatorAuthority, PortableAspectContractBasis,
    PortableAspectPatchOperation, PortableRecordAspectPatch,
};
use worth_relational::facade::{
    durability::RecoveryVerificationMode,
    history::BranchId,
    identity::{EntityId, KindId, PartitionId},
    runtime::{RelationalFieldPresence, RelationalFieldRevision, RelationalRuntime},
    snapshots::SnapshotHandle,
    transactions::{
        ApplyEntityAspectPatchIntent, CreateIntent, EntityMutationIntent, EntitySpec,
        MutationIntent, UpdateEntityFieldsIntent, WorkerIntentBatch,
    },
};
use worth_testkit_relational::{
    aspect_storage::{
        aspect_key, entity_summary_struct_aspect, field_key,
        publications::{
            begin_main_transaction, changed_entities, release_test_commit_snapshot,
            string_aspect_field_patch,
        },
        relation_field_aspect, AspectSchemaFixture,
    },
    durable_publication::{create_entity_outcome, memory_runtime},
    shared_relational_certification_test_execution_policy::serial_request,
};

fn locator(aspect: &str, field: &str) -> AspectFieldLocator {
    AspectFieldLocator::new(
        LocatorAuthority::Authoritative,
        aspect_key(aspect),
        CanonicalFieldPath::single(field_key(field)),
    )
}

#[test]
fn checkpoint_restore_preserves_exact_field_revision() {
    let host_request = serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let runtime = memory_runtime();
    let created = create_entity_outcome(execution, &runtime, "same");
    let entity = changed_entities(&created)[0];
    let name = locator("name", "name");
    let before = runtime
        .read_truth()
        .project_snapshot(&created.snapshot)
        .unwrap()
        .entity_field_revision(execution, entity, &name)
        .expect("read request remains live")
        .expect("created field has a native owner revision");
    runtime
        .durability_authority()
        .checkpoint(execution)
        .unwrap();
    let recovery = runtime
        .durability()
        .recovery_plan(
            execution,
            RecoveryVerificationMode::NormalRecoveryVerification,
        )
        .unwrap();
    let mut recovered = memory_runtime();
    let outcome = recovered
        .durability_recovery()
        .recover(execution, recovery)
        .unwrap();
    assert_eq!(outcome.coverage.checkpoint_commits, 1);
    let snapshot = snapshot_for_branch(&recovered, &BranchId("main".to_owned()));
    assert_eq!(
        recovered
            .read_truth()
            .project_snapshot(&snapshot)
            .unwrap()
            .entity_field_revision(execution, entity, &name)
            .expect("read request remains live"),
        Some(before)
    );
    recovered.snapshots().release_snapshot(&snapshot).unwrap();
    recovered
        .durability_authority()
        .checkpoint(execution)
        .unwrap();
    let second_recovery = recovered
        .durability()
        .recovery_plan(
            execution,
            RecoveryVerificationMode::NormalRecoveryVerification,
        )
        .unwrap();
    let mut reopened = memory_runtime();
    reopened
        .durability_recovery()
        .recover(execution, second_recovery)
        .unwrap();
    let reopened_snapshot = snapshot_for_branch(&reopened, &BranchId("main".to_owned()));
    assert_eq!(
        reopened
            .read_truth()
            .project_snapshot(&reopened_snapshot)
            .unwrap()
            .entity_field_revision(execution, entity, &name)
            .expect("read request remains live"),
        Some(before)
    );
    reopened
        .snapshots()
        .release_snapshot(&reopened_snapshot)
        .unwrap();
    release_test_commit_snapshot(&runtime, &created);
}

#[test]
fn checkpoint_restore_preserves_relation_field_revision() {
    let host_request = serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let fixture = || {
        AspectSchemaFixture {
            relation_aspects: vec![relation_field_aspect(
                aspect_key("label"),
                field_key("label"),
            )],
            ..AspectSchemaFixture::default()
        }
        .build_runtime()
    };
    let runtime = fixture();
    let main = BranchId("main".to_owned());
    let source = create_with_fields(execution, &runtime, "source", &main, Default::default());
    let target = create_with_fields(execution, &runtime, "target", &main, Default::default());
    let created = create_relation_outcome(
        execution,
        &runtime,
        changed_entities(&source)[0],
        changed_entities(&target)[0],
        "relation-label",
    );
    let relation = changed_relations(&created)[0];
    let label = locator("label", "label");
    let before = runtime
        .read_truth()
        .project_snapshot(&created.snapshot)
        .unwrap()
        .relation_field_revision(execution, relation, &label)
        .expect("read request remains live")
        .unwrap();
    assert_eq!(before.presence(), RelationalFieldPresence::Present);

    runtime
        .durability_authority()
        .checkpoint(execution)
        .unwrap();
    let recovery = runtime
        .durability()
        .recovery_plan(
            execution,
            RecoveryVerificationMode::NormalRecoveryVerification,
        )
        .unwrap();
    let mut recovered = fixture();
    recovered
        .durability_recovery()
        .recover(execution, recovery)
        .unwrap();
    let snapshot = snapshot_for_branch(&recovered, &BranchId("main".to_owned()));
    assert_eq!(
        recovered
            .read_truth()
            .project_snapshot(&snapshot)
            .unwrap()
            .relation_field_revision(execution, relation, &label)
            .expect("read request remains live"),
        Some(before)
    );
    recovered.snapshots().release_snapshot(&snapshot).unwrap();
    for outcome in [&source, &target, &created] {
        release_test_commit_snapshot(&runtime, outcome);
    }
}

#[test]
fn checkpoint_restore_preserves_forked_field_revisions_and_absent_aba() {
    let host_request = serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let binding = entity_summary_struct_aspect(aspect_key("summary"), field_key("summary"));
    let contract = binding.contract.clone();
    let fixture = || {
        AspectSchemaFixture {
            entity_aspects: vec![binding.clone()],
            ..AspectSchemaFixture::default()
        }
        .build_runtime()
    };
    let runtime = fixture();
    let mut create = begin_main_transaction(&runtime);
    create
        .push_batch(
            WorkerIntentBatch::new("checkpoint-field-create").push(MutationIntent::Create(
                CreateIntent::Entity(EntitySpec {
                    partition_id: PartitionId::main(),
                    kind_id: KindId(1),
                    client_key: "field-checkpoint".into(),
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
    fork(execution, &runtime, "field-prior");
    let prior_branch = BranchId("field-prior".into());
    let title = locator("summary", "title");
    let status = locator("summary", "status");
    let initial_title = field_at(execution, &runtime, &created.snapshot, entity, &title).unwrap();
    let initial_absent = field_at(execution, &runtime, &created.snapshot, entity, &status).unwrap();
    assert_eq!(initial_absent.presence(), RelationalFieldPresence::Absent);

    let mut set = begin_main_transaction(&runtime);
    set.push_batch(
        WorkerIntentBatch::new("checkpoint-field-set").push(MutationIntent::Entity(
            EntityMutationIntent::UpdateFields(UpdateEntityFieldsIntent {
                entity_id: entity,
                fields: string_aspect_field_patch([(
                    aspect_key("summary"),
                    field_key("status"),
                    "temporary",
                )]),
            }),
        )),
        worth_execution::ExecutionAllocationPolicy::SystemAllocation,
    )
    .unwrap();
    let present = set.commit(execution, &runtime).unwrap();
    let present_revision =
        field_at(execution, &runtime, &present.snapshot, entity, &status).unwrap();
    assert_eq!(
        present_revision.presence(),
        RelationalFieldPresence::Present
    );
    fork(execution, &runtime, "field-present");
    let present_branch = BranchId("field-present".into());

    let mut clear = begin_main_transaction(&runtime);
    clear
        .push_batch(
            WorkerIntentBatch::new("checkpoint-field-clear").push(MutationIntent::Entity(
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
    let final_absent =
        field_at(execution, &runtime, &absent_again.snapshot, entity, &status).unwrap();
    assert_eq!(final_absent.presence(), RelationalFieldPresence::Absent);
    assert_ne!(initial_absent, final_absent);
    assert_eq!(
        field_at(execution, &runtime, &absent_again.snapshot, entity, &title),
        Some(initial_title)
    );

    runtime
        .durability_authority()
        .checkpoint(execution)
        .unwrap();
    let recovery = runtime
        .durability()
        .recovery_plan(
            execution,
            RecoveryVerificationMode::NormalRecoveryVerification,
        )
        .unwrap();
    let mut recovered = fixture();
    recovered
        .durability_recovery()
        .recover(execution, recovery)
        .unwrap();
    for (branch, expected) in [
        (prior_branch, initial_absent),
        (present_branch, present_revision),
        (BranchId("main".to_owned()), final_absent),
    ] {
        let snapshot = snapshot_for_branch(&recovered, &branch);
        assert_eq!(
            field_at(execution, &recovered, &snapshot, entity, &status),
            Some(expected)
        );
        assert_eq!(
            field_at(execution, &recovered, &snapshot, entity, &title),
            Some(initial_title)
        );
        recovered.snapshots().release_snapshot(&snapshot).unwrap();
    }
    for outcome in [&created, &present, &absent_again] {
        release_test_commit_snapshot(&runtime, outcome);
    }
}

fn field_at(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    runtime: &RelationalRuntime,
    snapshot: &SnapshotHandle,
    entity: EntityId,
    locator: &AspectFieldLocator,
) -> Option<RelationalFieldRevision> {
    runtime
        .read_truth()
        .project_snapshot(snapshot)?
        .entity_field_revision(execution, entity, locator)
        .expect("read request remains live")
}
