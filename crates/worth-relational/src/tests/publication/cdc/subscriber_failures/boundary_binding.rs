use super::fixtures::{install_schema_version, visible_bridge_transition_options};
use crate::schema::data::{
    SchemaBoundaryFingerprint, SchemaContinuationClassification, SchemaVersionId,
};
use crate::tests::support::*;

#[test]
fn subscriber_stream_rejects_checkpoint_with_mismatched_authoritative_boundary_binding() {
    let mut runtime = runtime_with_test_schema();
    let _first = create_entity_outcome(&runtime, "a");

    install_schema_version(&mut runtime, SchemaVersionId(2));
    let mut txn = {
        let transaction_validation_input =
            visible_bridge_transition_options(&runtime, SchemaVersionId(2));
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
    let _ = txn
        .commit(
            &runtime,
            worth_execution::ExecutionAllocationPolicy::SystemAllocation,
        )
        .unwrap();

    let batch = runtime
        .publication()
        .read_subscriber_stream(SubscriberResumeRequest::from_head(2))
        .unwrap();
    let forged = batch
        .next_checkpoint
        .unwrap()
        .with_authoritative_boundary_binding_for_test(
            Some(SchemaBoundaryFingerprint::ZERO),
            Some(SchemaContinuationClassification::ContinueUnchanged),
            Some(SchemaContinuationClassification::ContinueUnchanged),
            true,
        );

    let error = runtime
        .publication()
        .read_subscriber_stream(SubscriberResumeRequest::resume_after(forged, 1))
        .unwrap_err();

    assert_eq!(
        error.class,
        SubscriberStreamFailureClass::CheckpointContinuitySummaryMismatch
    );
}
