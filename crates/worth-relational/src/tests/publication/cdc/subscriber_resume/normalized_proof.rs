use super::fixtures::{install_schema_version, transaction_validation_input_for_subscriber_impact};
use crate::schema::data::{SchemaSubscriberImpact, SchemaVersionId};
use crate::tests::support::*;

#[test]
fn subscriber_stream_composes_prior_and_new_boundaries_into_normalized_proof() {
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

    let first_batch = runtime
        .publication()
        .read_subscriber_stream(SubscriberResumeRequest::from_head(10))
        .unwrap();
    let checkpoint = first_batch.next_checkpoint.clone().unwrap();
    assert_eq!(
        checkpoint
            .normalized_continuation_proof()
            .normalized_boundary_count(),
        1
    );

    install_schema_version(&mut runtime, SchemaVersionId(3));
    let mut second_txn = {
        let transaction_validation_input = transaction_validation_input_for_subscriber_impact(
            &runtime,
            SchemaVersionId(3),
            SchemaSubscriberImpact::ConsumableSurfaceChanged,
        );
        runtime
            .begin_branch_transaction(
                transaction_validation_input.basis(),
                transaction_validation_input.intent().clone(),
            )
            .expect("owner-admitted transaction context")
    };
    second_txn
        .push_batch(
            batch_create("c"),
            worth_execution::ExecutionAllocationPolicy::SystemAllocation,
        )
        .expect("test staging stays within configured resource budgets");
    second_txn
        .commit(
            &runtime,
            worth_execution::ExecutionAllocationPolicy::SystemAllocation,
        )
        .unwrap();

    let resumed = runtime
        .publication()
        .read_subscriber_stream(SubscriberResumeRequest::resume_after(checkpoint, 10))
        .unwrap();

    let next_checkpoint = resumed.next_checkpoint.unwrap();
    assert_eq!(
        next_checkpoint
            .normalized_continuation_proof()
            .normalized_boundary_count(),
        2
    );
    assert_eq!(
        next_checkpoint
            .continuation_summary()
            .normalized_boundary_count,
        2
    );
    assert_eq!(
        next_checkpoint
            .normalized_continuation_proof()
            .boundary_fingerprints()
            .len(),
        2
    );
}

#[test]
fn resumed_subscriber_stream_preserves_prior_boundary_and_adds_new_boundary_trace() {
    let mut runtime = runtime_with_test_schema();
    let _first = create_entity_outcome(&runtime, "a");

    install_schema_version(&mut runtime, SchemaVersionId(2));
    let mut first_transition = {
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
    first_transition
        .push_batch(
            batch_create("b"),
            worth_execution::ExecutionAllocationPolicy::SystemAllocation,
        )
        .expect("test staging stays within configured resource budgets");
    first_transition
        .commit(
            &runtime,
            worth_execution::ExecutionAllocationPolicy::SystemAllocation,
        )
        .unwrap();

    let first_batch = runtime
        .publication()
        .read_subscriber_stream(SubscriberResumeRequest::from_head(10))
        .unwrap();
    let checkpoint = first_batch.next_checkpoint.unwrap();
    assert_eq!(
        checkpoint
            .normalized_continuation_proof()
            .normalized_boundary_count(),
        1
    );

    install_schema_version(&mut runtime, SchemaVersionId(3));
    let mut second_transition = {
        let transaction_validation_input = transaction_validation_input_for_subscriber_impact(
            &runtime,
            SchemaVersionId(3),
            SchemaSubscriberImpact::ConsumableSurfaceChanged,
        );
        runtime
            .begin_branch_transaction(
                transaction_validation_input.basis(),
                transaction_validation_input.intent().clone(),
            )
            .expect("owner-admitted transaction context")
    };
    second_transition
        .push_batch(
            batch_create("c"),
            worth_execution::ExecutionAllocationPolicy::SystemAllocation,
        )
        .expect("test staging stays within configured resource budgets");
    second_transition
        .commit(
            &runtime,
            worth_execution::ExecutionAllocationPolicy::SystemAllocation,
        )
        .unwrap();

    let resumed = runtime
        .publication()
        .read_subscriber_stream(SubscriberResumeRequest::resume_after(checkpoint, 10))
        .unwrap();

    let next_checkpoint = resumed.next_checkpoint.unwrap();
    assert_eq!(
        next_checkpoint
            .normalized_continuation_proof()
            .normalized_boundary_count(),
        2
    );
    assert_eq!(
        next_checkpoint
            .normalized_continuation_proof()
            .boundary_fingerprints()
            .len(),
        2
    );
    assert_eq!(resumed.continuation.crossed_boundaries().len(), 1);
    let boundary_entries = resumed
        .diagnostics
        .iter()
        .flat_map(|artifact| artifact.entries.iter())
        .filter(|entry| entry.code == DiagnosticCode::SubscriberBoundaryEvaluated)
        .count();
    assert_eq!(boundary_entries, 1);
}
