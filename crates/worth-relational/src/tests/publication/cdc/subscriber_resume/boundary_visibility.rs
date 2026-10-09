use super::fixtures::{install_schema_version, transaction_validation_input_for_subscriber_impact};
use crate::publication::cdc::data::{
    SubscriberContinuationClassSet, SubscriberContractDeclaration, SubscriberStrataSet,
};
use crate::schema::data::{
    SchemaContinuationClassification, SchemaStratum, SchemaSubscriberImpact, SchemaVersionId,
};
use crate::tests::support::*;

#[test]
fn subscriber_stream_reports_crossed_schema_boundary_from_in_memory_history() {
    let mut runtime = runtime_with_test_schema();
    let _first = create_entity_outcome(&runtime, "a");

    install_schema_version(&mut runtime, SchemaVersionId(2));
    let mut txn = {
        let transaction_validation_input = transaction_validation_input_for_subscriber_impact(
            &runtime,
            SchemaVersionId(2),
            SchemaSubscriberImpact::ConsumableSurfaceChanged,
        );
        runtime
            .begin_branch_transaction(
                transaction_validation_input.basis(),
                transaction_validation_input.intent().clone(),
            )
            .expect("owner-admitted transaction context")
    };
    txn.push_batch(
        batch_create("b"),
        worth_execution::ExecutionAllocationPolicy::SystemAllocation,
    )
    .expect("test staging stays within configured resource budgets");
    txn.commit(
        &runtime,
        worth_execution::ExecutionAllocationPolicy::SystemAllocation,
    )
    .unwrap();

    let batch = runtime
        .publication()
        .read_subscriber_stream(SubscriberResumeRequest::from_head(10))
        .unwrap();

    assert_eq!(
        batch.continuation.continuation_outcome(),
        SchemaContinuationClassification::ContinueWithVisibleBridge
    );
    assert_eq!(batch.continuation.crossed_boundaries().len(), 1);
    assert_eq!(
        batch
            .continuation
            .continuation_summary()
            .continuation_outcome,
        SchemaContinuationClassification::ContinueWithVisibleBridge
    );
    assert_eq!(
        batch
            .continuation
            .continuation_summary()
            .crossed_boundary_count,
        1
    );
    assert_eq!(
        batch
            .next_checkpoint
            .unwrap()
            .normalized_continuation_proof()
            .normalized_boundary_count(),
        1
    );
}

#[test]
fn subscriber_stream_treats_unconsumed_boundary_as_unchanged() {
    let mut runtime = runtime_with_test_schema();
    let _first = create_entity_outcome(&runtime, "a");

    install_schema_version(&mut runtime, SchemaVersionId(2));
    let mut txn = {
        let transaction_validation_input = transaction_validation_input_for_subscriber_impact(
            &runtime,
            SchemaVersionId(2),
            SchemaSubscriberImpact::ConsumableSurfaceChanged,
        );
        runtime
            .begin_branch_transaction(
                transaction_validation_input.basis(),
                transaction_validation_input.intent().clone(),
            )
            .expect("owner-admitted transaction context")
    };
    txn.push_batch(
        batch_create("b"),
        worth_execution::ExecutionAllocationPolicy::SystemAllocation,
    )
    .expect("test staging stays within configured resource budgets");
    txn.commit(
        &runtime,
        worth_execution::ExecutionAllocationPolicy::SystemAllocation,
    )
    .unwrap();

    let contract = SubscriberContractDeclaration {
        contract_id: "subscriber.contract.identity-only.v1".to_string(),
        consumable_strata: SubscriberStrataSet::new([SchemaStratum::EntityIdentitySemantics]),
        accepted_continuation_classes: SubscriberContinuationClassSet::new([
            SchemaContinuationClassification::ContinueUnchanged,
        ]),
        accepted_upgrade_classes: SubscriberContinuationClassSet::new([]),
    };

    let batch = runtime
        .publication()
        .read_subscriber_stream(
            SubscriberResumeRequest::from_head(10).with_subscriber_contract(contract),
        )
        .unwrap();

    assert_eq!(
        batch.continuation.continuation_outcome(),
        SchemaContinuationClassification::ContinueUnchanged
    );
    assert_eq!(batch.continuation.crossed_boundaries().len(), 1);
}
