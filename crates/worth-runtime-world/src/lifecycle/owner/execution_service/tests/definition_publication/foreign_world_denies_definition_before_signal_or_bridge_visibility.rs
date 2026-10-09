//! foreign world denies definition before signal or bridge visibility.
use super::*;
#[test]
fn foreign_world_denies_definition_before_signal_or_bridge_visibility() {
    let source = definition_world();
    let source_lifecycle = source.bridge.conditional_lifecycle_probe();
    let foreign = definition_world();
    let policy = match foreign.owner.execution_placement() {
        crate::facade::RuntimeWorldExecutionPlacement::Serial(policy)
        | crate::facade::RuntimeWorldExecutionPlacement::Leased { policy, .. } => policy,
    };
    let serial_request = worth_execution::SerialRequest::from_memory(
        worth_execution::SerialMemoryBudget::from_policy(&policy),
        worth_execution::CancellationToken::new(),
        None,
    );
    let execution = worth_execution::ExecutionRequest::serial(&serial_request);

    let cancellation = RuntimeWorldCancellationSource::new();
    let prepared_world = source
        .owner
        .publication_port()
        .prepare_with_signal(
            source.root.clone(),
            CompositePublicationIntent::with_signal(None),
            &cancellation.token(),
            None,
        )
        .unwrap();
    let predecessor = source
        .bridge
        .admit_conditional_signal_basis(&source.lowering, source.root.basis().signal_basis())
        .unwrap();
    let prepared_bridge = source
        .bridge
        .prepare_owned_conditional_definition_successor(&predecessor, source_free_request(2))
        .unwrap();

    let outcome = foreign
        .owner
        .publication_port()
        .publish_bridge_conditional_definition(
            execution,
            prepared_world,
            prepared_bridge,
            &source.bridge,
            &cancellation.token(),
        );

    assert!(matches!(
        outcome,
        RuntimeWorldConditionalDefinitionPublicationOutcome::NoEffect(ref denial)
            if denial.cause() == crate::facade::NoEffectCause::OwnerDeniedBeforeEffect
    ));
    assert_eq!(source.bridge.installed_conditional_definition_count(), 1);
    assert_eq!(foreign.bridge.installed_conditional_definition_count(), 1);
    assert_eq!(source.lowering.signal_definition_generation(), 1);
    let current = source
        .owner
        .observation_port()
        .observe_product_branch(source.root.branch_identity())
        .unwrap();
    assert_eq!(current.snapshot(), source.root.snapshot());
    let retention = source_lifecycle.bridge_conditional_retention().unwrap();
    assert_eq!(retention.retained_definition_candidates(), 0);
    assert_eq!(retention.retained_bytes(), 0);
}
