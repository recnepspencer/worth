//! Public Native publication keeps a separate original readonly source current.
mod critical_section;
use super::retained_entity_selection::{create, fixture, perform, update, values};
use worth_execution::{ExecutionAllocationPolicy, ExecutionRequest};
use worth_relational::facade::{
    branch::{AdmittedRelationalBranchBasis, RelationalBranchIdentity},
    history::BranchId,
    identity::{KindId, PartitionId},
    mvcc::{
        PreparedRelationalCommitCandidate, RelationalPublicationDenial,
        RelationalPublicationOutcome,
        RelationalPublicationReadDependencyDenial as DependencyDenial, RelationalTransactionIntent,
    },
    runtime::RelationalRuntime,
    symbols::ClientKey,
    transactions::{CreateIntent, EntitySpec, MutationIntent, WorkerIntentBatch},
};
use worth_testkit_relational::{
    aspect_storage::{
        aspect_key, field_key,
        publications::{
            changed_entities, release_test_commit_snapshot, string_aspect_field_patch,
            string_aspect_value,
        },
    },
    shared_relational_certification_test_execution_policy::serial_request,
};

#[test]
fn changed_read_source_refuses_output_without_moving_either_target_lock_order() {
    for branch in ["aaa", "zzz"] {
        changed_source_refuses(branch, false);
    }
}

#[test]
fn equal_value_aba_cannot_restore_the_original_read_dependency() {
    changed_source_refuses("aaa", true);
}

fn changed_source_refuses(branch: &str, aba: bool) {
    let host = serial_request();
    let execution = &ExecutionRequest::serial(&host);
    let runtime = fixture().build_runtime();
    let original = create(execution, &runtime, "control", "allowed");
    let model = changed_entities(&original)[0];
    let source = runtime
        .admit_branch_basis(&runtime.main_branch_identity())
        .unwrap();
    let target = fork(execution, &runtime, branch);
    let before = runtime.admit_branch_basis(&target).unwrap();
    let mut candidate = prepare(execution, &runtime, &before);
    candidate.require_current_read(&source).unwrap();

    let changed = update(execution, &runtime, [(model, "revoked")]);
    release_test_commit_snapshot(&runtime, &changed);
    if aba {
        let returned = update(execution, &runtime, [(model, "allowed")]);
        assert_eq!(
            values(execution, &runtime, &returned, [model, model]),
            values(execution, &runtime, &original, [model, model])
        );
        release_test_commit_snapshot(&runtime, &returned);
    }
    let current = runtime
        .admit_branch_basis(&runtime.main_branch_identity())
        .unwrap();
    assert_ne!(current.descriptor(), source.descriptor());
    let committed_before = runtime.history().immutable_commit_count();
    assert!(matches!(
        runtime
            .publication_port()
            .compare_and_publish(execution, candidate),
        RelationalPublicationOutcome::Denied(RelationalPublicationDenial::CurrentReadDependency(
            DependencyDenial::NotCurrent
        ))
    ));
    assert_eq!(runtime.history().immutable_commit_count(), committed_before);
    assert_eq!(
        runtime.admit_branch_basis(&target).unwrap().descriptor(),
        before.descriptor()
    );
    assert_eq!(
        runtime
            .admit_branch_basis(&runtime.main_branch_identity())
            .unwrap()
            .descriptor(),
        current.descriptor()
    );

    // Same target and actual write, freshly bound to the real current C.
    let mut lawful = prepare(execution, &runtime, &before);
    lawful.require_current_read(&current).unwrap();
    let result = perform(execution, &runtime, lawful);
    let output = changed_entities(&result)[0];
    assert_eq!(
        values(execution, &runtime, &result, [output, output]),
        ["produced", "produced"].map(string_aspect_value)
    );
    assert_ne!(
        runtime.admit_branch_basis(&target).unwrap().descriptor(),
        before.descriptor()
    );
    assert_eq!(
        runtime
            .admit_branch_basis(&runtime.main_branch_identity())
            .unwrap()
            .descriptor(),
        current.descriptor()
    );
    release_test_commit_snapshot(&runtime, &result);
    release_test_commit_snapshot(&runtime, &original);
}

#[test]
fn foreign_or_duplicate_condition_cannot_replace_the_original_owner_binding() {
    let host = serial_request();
    let execution = &ExecutionRequest::serial(&host);
    let runtime = fixture().build_runtime();
    let original = create(execution, &runtime, "control", "allowed");
    let source = runtime
        .admit_branch_basis(&runtime.main_branch_identity())
        .unwrap();
    let other = fixture().build_runtime();
    let foreign_value = create(execution, &other, "control", "allowed");
    let foreign = other
        .admit_branch_basis(&other.main_branch_identity())
        .unwrap();
    let target = fork(execution, &runtime, "derived");
    let before = runtime.admit_branch_basis(&target).unwrap();
    let mut candidate = prepare(execution, &runtime, &before);
    assert_eq!(
        candidate.require_current_read(&foreign),
        Err(DependencyDenial::ForeignOwner)
    );
    candidate.require_current_read(&source).unwrap();
    assert_eq!(
        candidate.require_current_read(&source),
        Err(DependencyDenial::AlreadyRequired)
    );
    let result = perform(execution, &runtime, candidate);
    assert_eq!(changed_entities(&result).len(), 1);
    release_test_commit_snapshot(&runtime, &result);
    release_test_commit_snapshot(&runtime, &original);
    release_test_commit_snapshot(&other, &foreign_value);
}

#[test]
fn same_cell_requires_matching_axes_and_uses_one_live_publication_guard() {
    let host = serial_request();
    let execution = &ExecutionRequest::serial(&host);
    let runtime = fixture().build_runtime();
    let first = create(execution, &runtime, "control", "allowed");
    let model = changed_entities(&first)[0];
    let previous = runtime
        .admit_branch_basis(&runtime.main_branch_identity())
        .unwrap();
    let moved = update(execution, &runtime, [(model, "changed")]);
    let current = runtime
        .admit_branch_basis(&runtime.main_branch_identity())
        .unwrap();
    let mut candidate = prepare(execution, &runtime, &current);
    assert_eq!(
        candidate.require_current_read(&previous),
        Err(DependencyDenial::ConflictingSource)
    );
    candidate.require_current_read(&current).unwrap();
    let result = perform(execution, &runtime, candidate);
    assert_eq!(result.commit.parents, vec![moved.commit.commit_id]);
    assert_eq!(changed_entities(&result).len(), 1);
    for commit in [&first, &moved, &result] {
        release_test_commit_snapshot(&runtime, commit);
    }
}

#[test]
fn archived_read_source_refuses_before_the_live_target_moves() {
    let host = serial_request();
    let execution = &ExecutionRequest::serial(&host);
    let runtime = fixture().build_runtime();
    let original = create(execution, &runtime, "control", "allowed");
    let source = runtime
        .admit_branch_basis(&runtime.main_branch_identity())
        .unwrap();
    let target = fork(execution, &runtime, "derived");
    let before = runtime.admit_branch_basis(&target).unwrap();
    let mut candidate = prepare(execution, &runtime, &before);
    candidate.require_current_read(&source).unwrap();
    let _archived = runtime
        .owner_component_services()
        .lifecycle_port()
        .archive_branch(execution, source.identity())
        .unwrap();
    let committed_before = runtime.history().immutable_commit_count();
    assert!(matches!(
        runtime
            .publication_port()
            .compare_and_publish(execution, candidate),
        RelationalPublicationOutcome::Denied(RelationalPublicationDenial::CurrentReadDependency(
            DependencyDenial::NotCurrent
        ))
    ));
    assert_eq!(runtime.history().immutable_commit_count(), committed_before);
    assert_eq!(
        runtime.admit_branch_basis(&target).unwrap().descriptor(),
        before.descriptor()
    );
    release_test_commit_snapshot(&runtime, &original);
}

#[test]
fn an_uncommitted_empty_source_refuses_at_attachment_and_keeps_the_candidate_usable() {
    let host = serial_request();
    let execution = &ExecutionRequest::serial(&host);
    let runtime = fixture().build_runtime();
    let empty = runtime
        .admit_branch_basis(&runtime.main_branch_identity())
        .unwrap();
    // Empty references cannot fork. Use the actual empty main as D so the
    // genuine current empty C reaches the new attachment boundary itself.
    let target = runtime.main_branch_identity();
    let before = runtime.admit_branch_basis(&target).unwrap();
    let mut candidate = prepare(execution, &runtime, &before);
    assert_eq!(
        candidate.require_current_read(&empty),
        Err(DependencyDenial::EmptySourceUnsupported)
    );
    assert_eq!(
        runtime.admit_branch_basis(&target).unwrap().descriptor(),
        before.descriptor()
    );
    let ordinary = perform(execution, &runtime, candidate);
    assert_eq!(changed_entities(&ordinary).len(), 1);
    let current = runtime
        .admit_branch_basis(&runtime.main_branch_identity())
        .unwrap();
    let mut lawful = prepare_named_with_control(
        execution,
        &runtime,
        &current,
        "lawful-output",
        worth_relational::facade::mvcc::RelationalOperationControl::uninterrupted(),
    );
    lawful.require_current_read(&current).unwrap();
    let result = perform(execution, &runtime, lawful);
    assert_eq!(changed_entities(&result).len(), 1);
    release_test_commit_snapshot(&runtime, &result);
    release_test_commit_snapshot(&runtime, &ordinary);
}

fn fork(
    execution: &ExecutionRequest<'_, '_>,
    runtime: &RelationalRuntime,
    name: &str,
) -> RelationalBranchIdentity {
    let services = runtime.owner_component_services();
    let (_, source) = services
        .fork_port()
        .observe_fork_source(&BranchId("main".into()))
        .unwrap();
    services
        .fork_port()
        .fork_branch(execution, BranchId(name.into()), source)
        .unwrap()
        .target_identity()
        .clone()
}

fn prepare(
    execution: &ExecutionRequest<'_, '_>,
    runtime: &RelationalRuntime,
    target: &AdmittedRelationalBranchBasis,
) -> PreparedRelationalCommitCandidate {
    prepare_with_control(
        execution,
        runtime,
        target,
        worth_relational::facade::mvcc::RelationalOperationControl::uninterrupted(),
    )
}

fn prepare_with_control(
    execution: &ExecutionRequest<'_, '_>,
    runtime: &RelationalRuntime,
    target: &AdmittedRelationalBranchBasis,
    control: worth_relational::facade::mvcc::RelationalOperationControl,
) -> PreparedRelationalCommitCandidate {
    prepare_named_with_control(execution, runtime, target, "derived", control)
}

fn prepare_named_with_control(
    execution: &ExecutionRequest<'_, '_>,
    runtime: &RelationalRuntime,
    target: &AdmittedRelationalBranchBasis,
    key: &str,
    control: worth_relational::facade::mvcc::RelationalOperationControl,
) -> PreparedRelationalCommitCandidate {
    let mut transaction = runtime
        .begin_branch_transaction_with_control(
            target,
            RelationalTransactionIntent::ordinary(),
            control,
        )
        .unwrap();
    let batch = WorkerIntentBatch::new("conditional-output").push(MutationIntent::Create(
        CreateIntent::Entity(EntitySpec {
            partition_id: PartitionId::main(),
            kind_id: KindId(1),
            client_key: ClientKey::raw(key),
            fields: string_aspect_field_patch([(
                aspect_key("name"),
                field_key("label"),
                "produced",
            )]),
        }),
    ));
    transaction
        .push_batch(batch, ExecutionAllocationPolicy::SystemAllocation)
        .unwrap();
    runtime
        .prepare_branch_transaction(execution, transaction)
        .unwrap()
}
