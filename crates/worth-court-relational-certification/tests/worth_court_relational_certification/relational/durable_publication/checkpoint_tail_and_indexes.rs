use super::branch_history_recovery::current_head;
use worth_relational::facade::history::BranchId;
use worth_relational::facade::indexes::{
    DerivedIndexBuildRequest, DerivedIndexDefinition, DerivedIndexId, DerivedIndexKind,
};
use worth_testkit_relational::aspect_storage::definition::{aspect_key, field_key};
use worth_testkit_relational::durable_medium::TestDurableMedium;
use worth_testkit_relational::durable_publication::create_entity_outcome;
use worth_testkit_relational::durable_publication::durable_runtime;

use super::checkpoint_source_capture::capture_durably;
use worth_relational::facade::durability::RecoveryVerificationMode as RecoveryMode;
use worth_testkit_relational::aspect_storage::definition::aspect_field_locator;
#[test]
fn durability_contract_checkpoint_tail_recovery_preserves_post_checkpoint_commits() {
    let host_request = worth_testkit_relational::shared_relational_certification_test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let medium = TestDurableMedium::new();
    let runtime = durable_runtime(&medium);
    let main = create_entity_outcome(execution, &runtime, "main-a");
    let _checkpoint = capture_durably(execution, &runtime, &medium).unwrap();
    let later = create_entity_outcome(execution, &runtime, "main-b");
    let plan = runtime
        .durability()
        .recovery_plan_from_source(
            execution,
            medium.recovery_source(),
            RecoveryMode::NormalRecoveryVerification,
        )
        .unwrap();
    let mut recovered = durable_runtime(&medium);
    let outcome = recovered
        .durability_recovery()
        .recover(execution, plan)
        .unwrap();

    assert_eq!(outcome.recovered_commits, 2);
    assert_eq!(outcome.coverage.checkpoint_commits, 1);
    assert_eq!(outcome.coverage.replayed_tail_commits, 1);
    assert_eq!(outcome.latest_commit, Some(later.commit.clone()));
    assert_eq!(
        current_head(&recovered, &BranchId("main".to_string())),
        Some(later.commit.clone())
    );
    assert_eq!(
        recovered
            .replay()
            .canonical_commit_envelope(execution, main.commit.commit_id)
            .unwrap()
            .unwrap()
            .commit
            .commit_id,
        main.commit.commit_id
    );
}

#[test]
fn durability_contract_checkpoint_recovers_index_metadata() {
    let host_request = worth_testkit_relational::shared_relational_certification_test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let medium = TestDurableMedium::new();
    let runtime = durable_runtime(&medium);
    let commit = create_entity_outcome(execution, &runtime, "indexed");
    let index = runtime
        .index_authority()
        .register(DerivedIndexDefinition {
            index_id: DerivedIndexId(0),
            name: "entity-name".to_string(),
            kind: DerivedIndexKind::EntityField {
                field_locator: aspect_field_locator(aspect_key("name"), field_key("name")),
            },
            branch_scoped: false,
        })
        .expect("index fixture has identity capacity");
    let build = runtime.index_authority().build_for_commit(
        execution,
        DerivedIndexBuildRequest {
            source_commit_id: commit.commit.commit_id,
            branch_id: BranchId("main".to_string()),
            index_ids: vec![index.index_id],
        },
    );
    capture_durably(execution, &runtime, &medium).unwrap();
    let plan = runtime
        .durability()
        .recovery_plan_from_source(
            execution,
            medium.recovery_source(),
            RecoveryMode::NormalRecoveryVerification,
        )
        .unwrap();
    let mut recovered = durable_runtime(&medium);
    recovered
        .durability_recovery()
        .recover(execution, plan)
        .unwrap();

    let index_access = recovered.index_access();
    let generation = index_access
        .read_published_generation_for_observation(
            execution,
            index.index_id,
            &recovered
                .observe_branch(&recovered.main_branch_identity())
                .unwrap()
                .1
                .observation(),
        )
        .unwrap()
        .unwrap();
    assert_eq!(generation.generation_id, build.generations[0].generation_id);
    assert_eq!(generation.source_commit_id, commit.commit.commit_id);
}
