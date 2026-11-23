//! Fixture adaptation (C2.5): restores f9f15bf8c8 deleted-branch reader and
//! closed-branch index generation reclamation over the Memory runtime (no retired test medium).
use super::*;
use crate::durability::data::RecoveryVerificationMode as RecoveryMode;

macro_rules! scoped_name_index {
    ($runtime:expr) => {
        $runtime
            .index_authority()
            .register(DerivedIndexDefinition {
                index_id: DerivedIndexId(0),
                name: "scoped-checkpoint-index".into(),
                kind: DerivedIndexKind::EntityField {
                    field_locator: aspect_field_locator(aspect_key("name"), field_key("name")),
                },
                branch_scoped: true,
            })
            .expect("index fixture has identity capacity")
    };
}

#[test]
fn deleted_branch_reader_retains_only_its_exact_scoped_generation() {
    let host_request = crate::test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let mut runtime = runtime_with_test_schema();
    let anchor = create_entity_outcome(execution, &runtime, "pinned-deleted-fork-anchor");
    let index = scoped_name_index!(runtime);
    release_test_commit_snapshot(&runtime, &anchor);
    let branch = create_branch_from_main(&runtime, "pinned-deleted-fork");
    let identity = runtime.branch_identity(&branch).unwrap();
    let (_, basis) = runtime.observe_branch(&identity).unwrap();
    let (_, second_basis) = runtime.observe_branch(&identity).unwrap();
    let built = runtime.index_authority().build_for_basis(
        execution,
        DerivedIndexBuildRequest {
            source_commit_id: anchor.commit.commit_id,
            branch_id: branch.clone(),
            index_ids: vec![index.index_id],
        },
        &basis,
    );
    assert!(built.failed_indexes.is_empty());
    let generation = built.generations[0].generation_id;
    assert!(runtime
        .delete_branch(execution, &identity)
        .unwrap()
        .deleted()
        .is_some());
    assert_eq!(
        runtime
            .run_index_generation_reclamation_after_publication(&anchor)
            .unwrap(),
        Some(0)
    );
    assert!(runtime.indexes.generation(generation).is_some());
    runtime.durability_authority().checkpoint(execution).unwrap();
    let plan = runtime
        .durability()
        .recovery_plan(execution, RecoveryMode::NormalRecoveryVerification)
        .unwrap();
    let mut recovered = runtime_with_test_schema();
    recovered
        .durability_recovery()
        .recover(execution, plan)
        .unwrap();
    assert!(recovered.indexes.generation(generation).is_none());
    drop(basis);
    assert_eq!(
        runtime
            .run_index_generation_reclamation_after_publication(&anchor)
            .unwrap(),
        Some(0)
    );
    assert!(runtime.indexes.generation(generation).is_some());
    drop(second_basis);
    assert_eq!(
        runtime
            .run_index_generation_reclamation_after_publication(&anchor)
            .unwrap(),
        Some(1)
    );
    assert_eq!(runtime.run_index_generation_reclamation_pass(), Some(0));
    assert!(runtime.indexes.generation(generation).is_none());
}

#[test]
fn repeated_closed_branches_leave_no_latest_generation_residue() {
    let host_request = crate::test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let mut runtime = runtime_with_test_schema();
    let anchor = create_entity_outcome(execution, &runtime, "closed-index-anchor");
    let index = scoped_name_index!(runtime);
    release_test_commit_snapshot(&runtime, &anchor);
    for ordinal in 0..12 {
        let branch = create_branch_from_main(&runtime, &format!("closed-index-{ordinal}"));
        let identity = runtime.branch_identity(&branch).unwrap();
        let generation = {
            let (_, basis) = runtime.observe_branch(&identity).unwrap();
            let built = runtime.index_authority().build_for_basis(
                execution,
                DerivedIndexBuildRequest {
                    source_commit_id: anchor.commit.commit_id,
                    branch_id: branch.clone(),
                    index_ids: vec![index.index_id],
                },
                &basis,
            );
            assert!(built.failed_indexes.is_empty());
            built.generations[0].generation_id
        };
        assert!(runtime
            .delete_branch(execution, &identity)
            .unwrap()
            .deleted()
            .is_some());
        assert!(runtime.indexes.generation(generation).is_some());
        assert_eq!(runtime.run_index_generation_reclamation_pass(), Some(1));
        assert!(runtime.indexes.generation(generation).is_none());
        assert!(runtime
            .index_access()
            .read_latest_generation(execution, index.index_id, &branch)
            .unwrap()
            .is_none());
        assert_eq!(runtime.history.retired_branch_binding_count(), 0);
    }
    assert!(runtime.index_access().generations_snapshot().is_empty());
}
