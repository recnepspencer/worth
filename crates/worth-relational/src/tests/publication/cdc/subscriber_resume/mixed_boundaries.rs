use super::fixtures::{install_schema_version, transaction_validation_input_for_subscriber_impact};
use crate::publication::cdc::data::{
    SubscriberContinuationClassSet, SubscriberContractDeclaration,
};
use crate::schema::data::{
    SchemaContinuationClassification, SchemaSubscriberImpact, SchemaVersionId,
};
use crate::tests::support::*;

#[test]
fn subscriber_stream_mixed_boundaries_choose_strongest_supported_outcome_and_trace_each_boundary() {
    let mut runtime = runtime_with_test_schema();
    let _first = create_entity_outcome(&runtime, "a");

    install_schema_version(&mut runtime, SchemaVersionId(2));
    let mut visible_txn = {
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
    visible_txn
        .push_batch(
            batch_create("b"),
            worth_execution::ExecutionAllocationPolicy::SystemAllocation,
        )
        .expect("test staging stays within configured resource budgets");
    visible_txn
        .commit(
            &runtime,
            worth_execution::ExecutionAllocationPolicy::SystemAllocation,
        )
        .unwrap();

    install_schema_version(&mut runtime, SchemaVersionId(3));
    let mut upgrade_txn = {
        let transaction_validation_input = transaction_validation_input_for_subscriber_impact(
            &runtime,
            SchemaVersionId(3),
            SchemaSubscriberImpact::ContractUpgradeRequired,
        );
        runtime
            .begin_branch_transaction(
                transaction_validation_input.basis(),
                transaction_validation_input.intent().clone(),
            )
            .expect("owner-admitted transaction context")
    };
    upgrade_txn
        .push_batch(
            batch_create("c"),
            worth_execution::ExecutionAllocationPolicy::SystemAllocation,
        )
        .expect("test staging stays within configured resource budgets");
    upgrade_txn
        .commit(
            &runtime,
            worth_execution::ExecutionAllocationPolicy::SystemAllocation,
        )
        .unwrap();

    let contract = SubscriberContractDeclaration {
        contract_id: "subscriber.contract.geometry.v3".to_string(),
        accepted_upgrade_classes: SubscriberContinuationClassSet::new([
            SchemaContinuationClassification::ContinueWithContractUpgrade,
        ]),
        ..SubscriberContractDeclaration::default()
    };
    let batch = runtime
        .publication()
        .read_subscriber_stream(
            SubscriberResumeRequest::from_head(10).with_subscriber_contract(contract),
        )
        .unwrap();

    assert_eq!(
        batch.continuation.continuation_outcome(),
        SchemaContinuationClassification::ContinueWithContractUpgrade
    );
    assert_eq!(batch.continuation.crossed_boundaries().len(), 2);
    let boundary_entries = batch
        .diagnostics
        .iter()
        .flat_map(|artifact| artifact.entries.iter())
        .filter(|entry| entry.code == DiagnosticCode::SubscriberBoundaryEvaluated)
        .count();
    assert_eq!(boundary_entries, 2);
}

#[test]
fn resumed_subscriber_stream_mixed_boundaries_choose_strongest_supported_outcome() {
    let mut runtime = runtime_with_test_schema();
    let _first = create_entity_outcome(&runtime, "a");
    let contract = SubscriberContractDeclaration {
        contract_id: "subscriber.contract.geometry.v3".to_string(),
        accepted_upgrade_classes: SubscriberContinuationClassSet::new([
            SchemaContinuationClassification::ContinueWithContractUpgrade,
        ]),
        ..SubscriberContractDeclaration::default()
    };

    let first_batch = runtime
        .publication()
        .read_subscriber_stream(
            SubscriberResumeRequest::from_head(1).with_subscriber_contract(contract.clone()),
        )
        .unwrap();
    let checkpoint = first_batch.next_checkpoint.unwrap();

    install_schema_version(&mut runtime, SchemaVersionId(2));
    let mut visible_txn = {
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
    visible_txn
        .push_batch(
            batch_create("b"),
            worth_execution::ExecutionAllocationPolicy::SystemAllocation,
        )
        .expect("test staging stays within configured resource budgets");
    visible_txn
        .commit(
            &runtime,
            worth_execution::ExecutionAllocationPolicy::SystemAllocation,
        )
        .unwrap();

    install_schema_version(&mut runtime, SchemaVersionId(3));
    let mut upgrade_txn = {
        let transaction_validation_input = transaction_validation_input_for_subscriber_impact(
            &runtime,
            SchemaVersionId(3),
            SchemaSubscriberImpact::ContractUpgradeRequired,
        );
        runtime
            .begin_branch_transaction(
                transaction_validation_input.basis(),
                transaction_validation_input.intent().clone(),
            )
            .expect("owner-admitted transaction context")
    };
    upgrade_txn
        .push_batch(
            batch_create("c"),
            worth_execution::ExecutionAllocationPolicy::SystemAllocation,
        )
        .expect("test staging stays within configured resource budgets");
    upgrade_txn
        .commit(
            &runtime,
            worth_execution::ExecutionAllocationPolicy::SystemAllocation,
        )
        .unwrap();

    let resumed = runtime
        .publication()
        .read_subscriber_stream(
            SubscriberResumeRequest::resume_after(checkpoint, 10)
                .with_subscriber_contract(contract),
        )
        .unwrap();

    assert_eq!(
        resumed.continuation.continuation_outcome(),
        SchemaContinuationClassification::ContinueWithContractUpgrade
    );
    assert_eq!(
        resumed.recovery_decision.disposition,
        crate::publication::cdc::data::SubscriberRecoveryDisposition::ContinueWithContractUpgrade
    );
    assert_eq!(resumed.continuation.crossed_boundaries().len(), 2);
    let next_checkpoint = resumed.next_checkpoint.unwrap();
    assert_eq!(
        next_checkpoint
            .normalized_continuation_proof()
            .normalized_boundary_count(),
        2
    );
    assert_eq!(
        next_checkpoint.continuation_summary().continuation_outcome,
        SchemaContinuationClassification::ContinueWithContractUpgrade
    );
    let boundary_entries = resumed
        .diagnostics
        .iter()
        .flat_map(|artifact| artifact.entries.iter())
        .filter(|entry| entry.code == DiagnosticCode::SubscriberBoundaryEvaluated)
        .count();
    assert_eq!(boundary_entries, 2);
}
