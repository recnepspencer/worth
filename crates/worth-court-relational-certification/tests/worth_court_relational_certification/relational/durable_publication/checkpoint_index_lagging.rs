use super::checkpoint_index_generations::{
    build_on_branch, fork_branch, scoped_name_index, update_entity_on_branch,
};
use super::checkpoint_source_capture::capture_durably;
use super::merge_execution_observation::entity_field_display;
use worth_relational::facade::{
    durability::{RecoveryPlan, RecoveryVerificationMode},
    history::{BranchId, RelationalCommitReceipt},
    indexes::{DerivedIndexBuildRequest, DerivedIndexGenerationId},
    mvcc::RelationalTransactionIntent,
};
use worth_testkit_relational::{
    aspect_storage::{
        definition::{aspect_key, field_key},
        publications::{changed_entities, release_test_commit_snapshot},
    },
    durable_medium::TestDurableMedium,
    durable_publication::{create_entity_outcome, durable_runtime},
};
#[test]
fn retained_inherited_and_own_stream_generations_survive_native_maintenance_and_checkpoint_reopen()
{
    let host_request = worth_testkit_relational::shared_relational_certification_test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let medium = TestDurableMedium::new();
    let runtime = durable_runtime(&medium);
    let anchor = create_entity_outcome(execution, &runtime, "immutable-parent-name");
    let entity = changed_entities(&anchor)[0];
    let branch = fork_branch(execution, &runtime, "lagging-scoped-fork");
    let index = scoped_name_index(&runtime);
    let identity = runtime.branch_identity(&branch).unwrap();
    let (_, inherited_basis) = runtime.observe_branch(&identity).unwrap();
    let inherited = runtime.index_authority().build_for_basis(
        execution,
        DerivedIndexBuildRequest {
            source_commit_id: anchor.commit.commit_id,
            branch_id: branch.clone(),
            index_ids: vec![index.index_id],
        },
        &inherited_basis,
    );
    assert!(inherited.failed_indexes.is_empty());
    assert_eq!(inherited.generations.len(), 1);
    let inherited_generation = inherited.generations[0].generation_id;
    assert_ne!(anchor.commit.branch_id, branch);
    let _inherited_hold = inherited_basis;
    release_test_commit_snapshot(&runtime, &anchor);

    // These expectations come from actual committed native fixtures, never
    // from a manufactured generation or a query on the lagging index.
    let assert_reopened = |plan: RecoveryPlan,
                           generation: DerivedIndexGenerationId,
                           source: &RelationalCommitReceipt,
                           current: &RelationalCommitReceipt,
                           expected_name: &str| {
        let mut reopened = durable_runtime(&medium);
        reopened
            .durability_recovery()
            .recover(execution, plan)
            .unwrap();
        let retained =
            super::checkpoint_index_generations::generation(execution, &reopened, generation)
                .unwrap();
        assert_eq!(retained.index_id, index.index_id);
        assert_eq!(retained.source_commit_id, source.commit_id);
        assert_eq!(retained.source_branch_id, branch);
        assert_eq!(retained.applicability.branch_id, branch);
        assert_eq!(retained.applicability.version_id, source.version_id);
        assert_ne!(source.commit_id, current.commit_id);
        let identity = reopened.branch_identity(&branch).unwrap();
        let (_, basis) = reopened.observe_branch(&identity).unwrap();
        assert_eq!(basis.observation().commit_receipt(), Some(current));
        assert!(
            reopened
                .index_access()
                .read_published_generation_for_observation(
                    execution,
                    index.index_id,
                    &basis.observation()
                )
                .unwrap()
                .is_some_and(|selected| selected.generation_id != generation),
            "the successor selects its native generation while the old reader remains valid"
        );
        let read = reopened
            .begin_branch_transaction(&basis, RelationalTransactionIntent::ordinary())
            .unwrap()
            .read_entity(execution, entity)
            .unwrap();
        assert_eq!(
            read.base().and_then(|record| entity_field_display(
                record,
                &aspect_key("name"),
                &field_key("name")
            )),
            Some(expected_name.to_owned())
        );
    };

    let (_, inherited_source) = runtime.observe_fork_source(&branch).unwrap();
    runtime
        .fork_branch(
            execution,
            BranchId("inherited-reader-head".into()),
            inherited_source,
        )
        .unwrap();
    let first = update_entity_on_branch(
        execution,
        &runtime,
        entity,
        "fork-own-first",
        branch.clone(),
    );
    assert_eq!(first.commit.branch_id, branch);
    release_test_commit_snapshot(&runtime, &first);
    capture_durably(execution, &runtime, &medium).unwrap();
    assert_reopened(
        runtime
            .durability()
            .recovery_plan_from_source(
                execution,
                medium.recovery_source(),
                RecoveryVerificationMode::NormalRecoveryVerification,
            )
            .unwrap(),
        inherited_generation,
        &anchor.commit,
        &first.commit,
        "fork-own-first",
    );

    // The next cache is authored on the fork's own older commit. Advance its
    // real branch once more without rebuilding that cache, then capture again.
    let own_generation =
        build_on_branch(execution, &runtime, index.index_id, &first, branch.clone());
    let (_, _own_hold) = runtime.observe_branch(&identity).unwrap();
    let (_, own_source) = runtime.observe_fork_source(&branch).unwrap();
    runtime
        .fork_branch(execution, BranchId("own-reader-head".into()), own_source)
        .unwrap();
    let second = update_entity_on_branch(
        execution,
        &runtime,
        entity,
        "fork-own-second",
        branch.clone(),
    );
    assert_eq!(second.commit.branch_id, branch);
    release_test_commit_snapshot(&runtime, &second);
    capture_durably(execution, &runtime, &medium).unwrap();
    assert_reopened(
        runtime
            .durability()
            .recovery_plan_from_source(
                execution,
                medium.recovery_source(),
                RecoveryVerificationMode::NormalRecoveryVerification,
            )
            .unwrap(),
        own_generation,
        &first.commit,
        &second.commit,
        "fork-own-second",
    );
}
