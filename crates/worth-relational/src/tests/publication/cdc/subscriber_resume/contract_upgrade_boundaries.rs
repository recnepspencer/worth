use super::fixtures::{install_schema_version, transaction_validation_input_for_subscriber_impact};
use crate::diagnostics::data::RelationalDiagnosticValue;
use crate::publication::cdc::data::{
    SubscriberContinuationClassSet, SubscriberContractDeclaration,
};
use crate::schema::data::{
    SchemaContinuationClassification, SchemaSubscriberImpact, SchemaVersionId,
};
use crate::tests::support::*;

#[test]
fn subscriber_stream_rejects_unsupported_contract_upgrade_boundary() {
    let mut runtime = runtime_with_test_schema();
    let _first = create_entity_outcome(&runtime, "a");

    install_schema_version(&mut runtime, SchemaVersionId(2));
    let mut txn = {
        let transaction_validation_input = transaction_validation_input_for_subscriber_impact(
            &runtime,
            SchemaVersionId(2),
            SchemaSubscriberImpact::ContractUpgradeRequired,
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
        contract_id: "subscriber.contract.geometry.v1".to_string(),
        accepted_upgrade_classes: SubscriberContinuationClassSet::new([]),
        ..SubscriberContractDeclaration::default()
    };
    let error = runtime
        .publication()
        .read_subscriber_stream(
            SubscriberResumeRequest::from_head(10).with_subscriber_contract(contract),
        )
        .unwrap_err();

    assert_eq!(
        error.class,
        SubscriberStreamFailureClass::ContractUpgradeUnsupported
    );
    let rejection_entry = error
        .diagnostics
        .iter()
        .flat_map(|artifact| artifact.entries.iter())
        .find(|entry| entry.code == DiagnosticCode::SubscriberContractEvaluated)
        .unwrap();
    assert_eq!(
        diagnostic_field(rejection_entry, "subscriber_contract_id"),
        &RelationalDiagnosticValue::string("subscriber.contract.geometry.v1")
    );
    assert_eq!(
        diagnostic_field(rejection_entry, "failure_class"),
        &RelationalDiagnosticValue::string("ContractUpgradeUnsupported")
    );
    assert_eq!(
        diagnostic_field(rejection_entry, "normalized_boundary_count_at_failure"),
        &RelationalDiagnosticValue::Unsigned(1)
    );
}

#[test]
fn subscriber_stream_applies_contract_upgrade_when_declared_supported() {
    let mut runtime = runtime_with_test_schema();
    let _first = create_entity_outcome(&runtime, "a");

    install_schema_version(&mut runtime, SchemaVersionId(2));
    let mut txn = {
        let transaction_validation_input = transaction_validation_input_for_subscriber_impact(
            &runtime,
            SchemaVersionId(2),
            SchemaSubscriberImpact::ContractUpgradeRequired,
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
        contract_id: "subscriber.contract.geometry.v2".to_string(),
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
    assert!(batch.continuation.contract_upgrade_applied());
    assert_eq!(
        batch
            .continuation
            .continuation_summary()
            .continuation_outcome,
        SchemaContinuationClassification::ContinueWithContractUpgrade
    );
    assert_eq!(
        batch.recovery_decision.disposition,
        crate::publication::cdc::data::SubscriberRecoveryDisposition::ContinueWithContractUpgrade
    );
    assert!(batch.diagnostics.iter().any(|artifact| artifact
        .entries
        .iter()
        .any(|entry| { entry.code == DiagnosticCode::SubscriberContractEvaluated })));
    assert!(batch.diagnostics.iter().any(|artifact| artifact
        .entries
        .iter()
        .any(|entry| { entry.code == DiagnosticCode::SubscriberContractUpgradeDecision })));
}

#[test]
fn subscriber_stream_rejects_renegotiation_required_boundary() {
    let mut runtime = runtime_with_test_schema();
    let _first = create_entity_outcome(&runtime, "a");

    install_schema_version(&mut runtime, SchemaVersionId(2));
    let mut txn = {
        let transaction_validation_input = transaction_validation_input_for_subscriber_impact(
            &runtime,
            SchemaVersionId(2),
            SchemaSubscriberImpact::RenegotiationRequired,
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

    let error = runtime
        .publication()
        .read_subscriber_stream(SubscriberResumeRequest::from_head(10))
        .unwrap_err();

    assert_eq!(
        error.class,
        SubscriberStreamFailureClass::RenegotiationRequired
    );
    assert!(error
        .diagnostics
        .iter()
        .flat_map(|artifact| artifact.entries.iter())
        .any(|entry| entry.code == DiagnosticCode::SubscriberBoundaryEvaluated));
    assert!(error
        .diagnostics
        .iter()
        .flat_map(|artifact| artifact.entries.iter())
        .any(|entry| entry.code == DiagnosticCode::SubscriberRenegotiationDecision));
    let rejection_entry = error
        .diagnostics
        .iter()
        .flat_map(|artifact| artifact.entries.iter())
        .find(|entry| entry.code == DiagnosticCode::SubscriberContractEvaluated)
        .unwrap();
    assert_eq!(
        diagnostic_field(rejection_entry, "normalized_boundary_count_at_failure"),
        &RelationalDiagnosticValue::Unsigned(1)
    );
}
