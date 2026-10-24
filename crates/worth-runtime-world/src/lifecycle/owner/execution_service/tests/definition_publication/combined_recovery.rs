//! A stopped combined publication must keep the original unexecuted definition.
use super::*;
use crate::facade::{RelationalAttemptProgressPosture, SignalAttemptProgressPosture};
use worth_relational::facade::{
    mvcc::RelationalTransactionIntent, transactions::WorkerIntentBatch,
};

#[path = "combined_recovery/continuation.rs"]
mod continuation;
#[path = "combined_recovery/refusals.rs"]
mod refusals;

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
fn combined_settlement_stop_retains_the_original_prepared_definition() {
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
    let candidate_retention = lifecycle.bridge_conditional_retention().unwrap();
    assert_eq!(candidate_retention.retained_definition_candidates(), 1);
    assert!(candidate_retention.retained_bytes() > baseline.retained_bytes());

    fixture._relational.fail_next_durable_append_for_test();
    let unpublished = match fixture
        .owner
        .publication_port()
        .publish_bridge_conditional_definition(
            execution,
            prepared_world,
            prepared_bridge,
            &fixture.bridge,
            &cancellation.token(),
        ) {
        RuntimeWorldConditionalDefinitionPublicationOutcome::ProductUnpublished(partial) => partial,
        _ => panic!("the real Relational settlement failure must retain a combined partial"),
    };
    assert!(unpublished
        .effects()
        .progress()
        .relational_requires_settlement());
    assert_eq!(
        unpublished.effects().progress().signal_posture(),
        SignalAttemptProgressPosture::Untouched
    );
    assert_eq!(unpublished.effects().owner_effect_count(), 1);
    assert_eq!(fixture.bridge.installed_conditional_definition_count(), 1);
    assert_eq!(
        fixture
            .owner
            .observation_port()
            .observe_product_branch(fixture.root.branch_identity())
            .unwrap()
            .snapshot(),
        fixture.root.snapshot(),
    );
    let retained_after_partial = lifecycle.bridge_conditional_retention().unwrap();

    // Settle and release through the exact native owner even when the retention oracle fails.
    let handle = unpublished.effects().recovery_handle();
    drop(unpublished);
    let effects = fixture
        .owner
        .recovery_port()
        .inspect_effects(&handle)
        .unwrap();
    fixture
        .owner
        .recovery_port()
        .continue_effects(effects)
        .unwrap();
    let settled = fixture
        .owner
        .recovery_port()
        .inspect_effects(&handle)
        .unwrap();
    assert_eq!(
        settled.progress().relational_posture(),
        RelationalAttemptProgressPosture::Settled
    );
    assert_eq!(
        settled.progress().signal_posture(),
        SignalAttemptProgressPosture::Untouched
    );
    drop(settled);
    fixture
        .owner
        .recovery_port()
        .release_effects(&handle, 0)
        .unwrap();
    assert_eq!(lifecycle.bridge_conditional_retention().unwrap(), baseline);
    assert_eq!(
        retained_after_partial, candidate_retention,
        "unpublished Relational effects still require the exact prepared Signal definition and its reservation",
    );
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
