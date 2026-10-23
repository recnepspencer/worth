use super::*;
use crate::facade::{
    NoEffectCause, RuntimeWorldRecoveryDenial, RuntimeWorldSettledRelationalAdoptionDenial,
};

#[test]
fn foreign_bridge_and_stale_world_preserve_original_combined_custody() {
    let fixture = definition_world();
    let foreign = definition_world();
    let request = worth_execution::SerialRequest::from_memory(
        worth_execution::SerialMemoryBudget::from_policy(
            &fixture.owner.root.state.execution.request_policy(),
        ),
        worth_execution::CancellationToken::new(),
        None,
    );
    let execution = worth_execution::ExecutionRequest::serial(&request);
    let cancellation = RuntimeWorldCancellationSource::new();
    let prepared = combined(&fixture, &execution, &cancellation);
    let predecessor = fixture
        .bridge
        .admit_conditional_signal_basis(&fixture.lowering, fixture.root.basis().signal_basis())
        .unwrap();
    let candidate = fixture
        .bridge
        .prepare_owned_conditional_definition_successor(&predecessor, source_free_request(2))
        .unwrap();
    let lifecycle = fixture.bridge.conditional_lifecycle_probe();
    let retained = lifecycle.bridge_conditional_retention().unwrap();
    fixture._relational.fail_next_durable_append_for_test();
    let partial = match fixture
        .owner
        .publication_port()
        .publish_bridge_conditional_definition(
            execution,
            prepared,
            candidate,
            &fixture.bridge,
            &cancellation.token(),
        ) {
        RuntimeWorldConditionalDefinitionPublicationOutcome::ProductUnpublished(partial) => partial,
        _ => panic!("real settlement failure retains original combined publication"),
    };
    let handle = partial.effects().recovery_handle();
    drop(partial);
    fixture
        .owner
        .recovery_port()
        .continue_effects(
            fixture
                .owner
                .recovery_port()
                .inspect_effects(&handle)
                .unwrap(),
        )
        .unwrap();

    assert!(matches!(
        fixture
            .owner
            .publication_port()
            .prepare_bridge_conditional_definition_adoption(
                &handle,
                &foreign.bridge,
                &cancellation.token(),
                None
            ),
        Err(RuntimeWorldSettledRelationalAdoptionDenial::Recovery(
            RuntimeWorldRecoveryDenial::SettlementEvidenceUnavailable
        ))
    ));
    assert!(fixture
        .owner
        .recovery_port()
        .inspect_effects(&handle)
        .unwrap()
        .retains_signal_definition());
    assert_eq!(lifecycle.bridge_conditional_retention().unwrap(), retained);
    assert_eq!(foreign.bridge.installed_conditional_definition_count(), 1);

    // Prepare before a genuine competing World publication, then refuse at
    // execution against that changed head while restoring the original owner.
    let recovery = fixture
        .owner
        .publication_port()
        .prepare_bridge_conditional_definition_adoption(
            &handle,
            &fixture.bridge,
            &cancellation.token(),
            None,
        )
        .unwrap();
    let competitor = fixture
        .owner
        .publication_port()
        .prepare_with_signal(
            fixture.root.clone(),
            CompositePublicationIntent::with_signal(None),
            &cancellation.token(),
            None,
        )
        .unwrap();
    let winner = match fixture.owner.publication_port().execute_with_signal(
        execution,
        competitor,
        &mut (),
        &cancellation.token(),
        |_| Ok(()),
    ) {
        crate::facade::RuntimeWorldPublicationOutcome::Performed(winner) => winner,
        other => panic!("independent Signal publication must perform: {other:?}"),
    };
    let current = fixture
        .owner
        .observation_port()
        .observe_product_branch(fixture.root.branch_identity())
        .unwrap();
    assert!(
        matches!(fixture.owner.publication_port().execute_bridge_conditional_definition_adoption(execution, recovery, &fixture.bridge, &cancellation.token()), RuntimeWorldConditionalDefinitionPublicationOutcome::NoEffect(ref denial) if denial.cause() == NoEffectCause::StaleExpectedProductHead)
    );
    assert!(fixture
        .owner
        .recovery_port()
        .inspect_effects(&handle)
        .unwrap()
        .retains_signal_definition());
    assert_eq!(lifecycle.bridge_conditional_retention().unwrap(), retained);
    assert!(
        matches!(fixture.owner.publication_port().prepare_bridge_conditional_definition_adoption(&handle, &fixture.bridge, &cancellation.token(), None), Err(RuntimeWorldSettledRelationalAdoptionDenial::Publication(ref denial)) if denial.cause() == NoEffectCause::StaleExpectedProductHead)
    );
    assert_eq!(
        fixture
            .owner
            .observation_port()
            .observe_product_branch(fixture.root.branch_identity())
            .unwrap()
            .snapshot(),
        current.snapshot()
    );
    assert_eq!(fixture.bridge.installed_conditional_definition_count(), 1);
    fixture
        .owner
        .recovery_port()
        .release_effects(&handle, 0)
        .unwrap();
    assert_eq!(
        lifecycle
            .bridge_conditional_retention()
            .unwrap()
            .retained_definition_candidates(),
        0
    );
    drop(winner.consume());
}
