//! A discarded preparation leaves publication unchanged and later truth recoverable.
use super::branch_history_publications::transaction;
use super::branch_history_recovery::current_head;
use super::merge_execution_observation::read_entity_aspect_field_display;
use worth_relational::facade::history::BranchId;
use worth_testkit_relational::{
    aspect_storage::{
        definition::{aspect_key, field_key},
        publications::changed_entities,
    },
    durable_medium::TestDurableMedium,
    durable_publication::{batch_create, create_entity_outcome, durable_runtime},
};
#[test]
fn abandoned_client_preparation_leaves_publication_unchanged_and_later_keys_recover() {
    let host=worth_testkit_relational::shared_relational_certification_test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host);
    let medium = TestDurableMedium::new();
    let runtime = durable_runtime(&medium);
    create_entity_outcome(execution, &runtime, "published-before-abandonment");
    let capture = || {
        runtime
            .durability_authority()
            .native_checkpoint(
                execution,
                worth_execution::ExecutionAllocationPolicy::SystemAllocation,
            )
            .unwrap()
    };
    let _before = capture();
    let original_head = current_head(&runtime, &BranchId("main".into()));
    let mut candidate = transaction(&runtime, &BranchId("main".into()));
    candidate
        .push_batch(
            batch_create("unpublished-client-key"),
            worth_execution::ExecutionAllocationPolicy::SystemAllocation,
        )
        .unwrap();
    let proposal = candidate.validate(execution, &runtime).unwrap();
    let candidate = runtime
        .prepare_validated_proposal(execution, proposal)
        .unwrap();
    drop(candidate);
    let _after = capture();
    assert_eq!(
        current_head(&runtime, &BranchId("main".into())),
        original_head
    );
    let committed = create_entity_outcome(execution, &runtime, "published-after-abandonment");
    let entity = changed_entities(&committed)[0];
    let published = capture();
    let mut recovered = durable_runtime(&medium);
    let restored = recovered
        .durability_recovery()
        .restore_native_checkpoint(execution, &published)
        .unwrap();
    assert_eq!(restored.latest_commit, Some(committed.commit.clone()));
    assert_eq!(
        read_entity_aspect_field_display(
            execution,
            &recovered,
            &BranchId("main".into()),
            entity,
            aspect_key("name"),
            field_key("name")
        ),
        "published-after-abandonment"
    );
    create_entity_outcome(execution, &recovered, "editable-after-dictionary-recovery");
}
