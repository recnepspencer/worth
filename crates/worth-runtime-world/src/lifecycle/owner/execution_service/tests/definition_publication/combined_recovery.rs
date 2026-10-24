//! A stopped combined publication must keep the original unexecuted definition.
use super::*;
use crate::facade::SignalAttemptProgressPosture;
use worth_relational::facade::{
    mvcc::RelationalTransactionIntent, transactions::WorkerIntentBatch,
};

#[path = "combined_recovery/continuation.rs"]
mod continuation;

fn combined(
    fixture: &fixture::DefinitionWorld,
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    cancellation: &RuntimeWorldCancellationSource,
) -> crate::publication::PreparedCompositePublicationWithSignal {
    let services = fixture._relational.owner_component_services();
    let mut transaction = services
        .transaction_admission_port()
        .begin_branch_transaction(
            fixture.root.basis().relational_basis(),
            RelationalTransactionIntent::ordinary(),
        )
        .unwrap();
    transaction
        .push_batch(
            WorkerIntentBatch::new("definition-retention-settlement"),
            worth_execution::ExecutionAllocationPolicy::SystemAllocation,
        )
        .unwrap();
    let candidate = services
        .preparation_port()
        .prepare_branch_transaction(execution, transaction)
        .unwrap();
    fixture
        .owner
        .publication_port()
        .prepare_with_signal(
            fixture.root.clone(),
            CompositePublicationIntent::with_signal(Some(RelationalTransactionIntent::ordinary()))
                .with_prepared_relational_candidate(candidate),
            &cancellation.token(),
            None,
        )
        .unwrap()
}

#[test]
fn cancellation_before_any_owner_effect_releases_the_prepared_definition() {
    let fixture = definition_world();
    let lifecycle = fixture.bridge.conditional_lifecycle_probe();
    let baseline = lifecycle.bridge_conditional_retention().unwrap();
    let request = worth_execution::SerialRequest::from_memory(
        worth_execution::SerialMemoryBudget::from_policy(
            &fixture.owner.root.state.execution.request_policy(),
        ),
        worth_execution::CancellationToken::new(),
        None,
    );
    let execution = worth_execution::ExecutionRequest::serial(&request);
    let cancellation = RuntimeWorldCancellationSource::new();
    let prepared_world = combined(&fixture, &execution, &cancellation);
    let predecessor = fixture
        .bridge
        .admit_conditional_signal_basis(&fixture.lowering, fixture.root.basis().signal_basis())
        .unwrap();
    let prepared_bridge = fixture
        .bridge
        .prepare_owned_conditional_definition_successor(&predecessor, source_free_request(2))
        .unwrap();
    assert_eq!(
        lifecycle
            .bridge_conditional_retention()
            .unwrap()
            .retained_definition_candidates(),
        1
    );
    cancellation.cancel();
    let outcome = fixture
        .owner
        .publication_port()
        .publish_bridge_conditional_definition(
            execution,
            prepared_world,
            prepared_bridge,
            &fixture.bridge,
            &cancellation.token(),
        );
    assert!(matches!(
        outcome,
        RuntimeWorldConditionalDefinitionPublicationOutcome::NoEffect(_)
    ));
    assert_eq!(lifecycle.bridge_conditional_retention().unwrap(), baseline);
    assert_eq!(
        fixture
            .owner
            .observation_port()
            .observe_product_branch(fixture.root.branch_identity())
            .unwrap()
            .snapshot(),
        fixture.root.snapshot(),
    );
}
