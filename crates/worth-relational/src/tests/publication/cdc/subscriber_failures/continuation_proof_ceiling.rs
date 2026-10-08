use super::fixtures::{install_schema_version, visible_bridge_transition_options};
use crate::diagnostics::data::RelationalDiagnosticValue;
use crate::publication::cdc::data::{NormalizedContinuationProof, SubscriberContinuationSummary};
use crate::schema::data::{
    DescriptorSemanticsVersion, SchemaBoundaryFingerprint, SchemaContinuationClassification,
    SchemaVersionId,
};
use crate::tests::support::*;

#[test]
fn subscriber_stream_rejects_when_normalized_continuation_proof_exceeds_complexity_ceiling() {
    let mut runtime = runtime_with_test_schema();
    let _ = create_entity_outcome(&runtime, "anchor");
    let batch = runtime
        .publication()
        .read_subscriber_stream(SubscriberResumeRequest::from_head(1))
        .unwrap();
    let checkpoint = batch
        .next_checkpoint
        .unwrap()
        .with_incoherent_continuation_for_test(
            "default.subscriber.contract".to_string(),
            NormalizedContinuationProof::from_raw_parts_for_test(
                (0_u8..64)
                    .map(|value| SchemaBoundaryFingerprint::new([value; 32]))
                    .collect(),
                DescriptorSemanticsVersion::default(),
                64,
            ),
            SubscriberContinuationSummary::new(
                "default.subscriber.contract".to_string(),
                SchemaContinuationClassification::ContinueWithVisibleBridge,
                64,
                64,
                DescriptorSemanticsVersion::default(),
                false,
            ),
            DescriptorSemanticsVersion::default(),
        );

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
    txn.commit(
        &runtime,
        worth_execution::ExecutionAllocationPolicy::SystemAllocation,
    )
    .unwrap();

    let error = runtime
        .publication()
        .read_subscriber_stream(SubscriberResumeRequest::resume_after(checkpoint, 10))
        .unwrap_err();

    assert_eq!(
        error.class,
        SubscriberStreamFailureClass::RenegotiationRequired
    );
    let rejection_entry = error
        .diagnostics
        .iter()
        .flat_map(|artifact| artifact.entries.iter())
        .find(|entry| entry.code == DiagnosticCode::SubscriberContractEvaluated)
        .unwrap();
    assert_eq!(
        diagnostic_field(rejection_entry, "normalized_boundary_count_at_failure"),
        &RelationalDiagnosticValue::Unsigned(65)
    );
    assert!(error
        .diagnostics
        .iter()
        .flat_map(|artifact| artifact.entries.iter())
        .any(|entry| entry.code == DiagnosticCode::SubscriberRenegotiationDecision));
}
