use super::*;
use crate::facade::{RuntimeWorldRecoveryDenial, RuntimeWorldSettledRelationalAdoptionDenial};

fn bridge_candidate(
    fixture: &fixture::DefinitionWorld,
) -> worth_runtime_bridge::facade::BridgePreparedConditionalInstallationExtension {
    let predecessor = fixture
        .bridge
        .admit_conditional_signal_basis(&fixture.lowering, fixture.root.basis().signal_basis())
        .unwrap();
    fixture
        .bridge
        .prepare_owned_conditional_definition_successor(&predecessor, source_free_request(2))
        .unwrap()
}

#[test]
fn settled_relational_recovery_uses_original_definition_and_preserves_refused_custody() {
    let fixture = definition_world();
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
    let candidate = bridge_candidate(&fixture);
    let retained = fixture
        .bridge
        .conditional_lifecycle_probe()
        .bridge_conditional_retention()
        .unwrap();
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
        _ => panic!("real settlement stop must return unpublished effects"),
    };
    let handle = partial.effects().recovery_handle();
    let relational = partial
        .effects()
        .component_results()
        .relational_publication_identity()
        .unwrap();
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

    let inspector = fixture
        .owner
        .recovery_port()
        .inspect_effects(&handle)
        .unwrap();
    assert!(matches!(
        fixture
            .owner
            .publication_port()
            .prepare_bridge_conditional_definition_adoption(
                &handle,
                &fixture.bridge,
                &cancellation.token(),
                None
            ),
        Err(RuntimeWorldSettledRelationalAdoptionDenial::Recovery(
            RuntimeWorldRecoveryDenial::CallerCapabilityLive
        ))
    ));
    drop(inspector);
    let prepared = fixture
        .owner
        .publication_port()
        .prepare_bridge_conditional_definition_adoption(
            &handle,
            &fixture.bridge,
            &cancellation.token(),
            None,
        )
        .unwrap();
    assert!(matches!(
        fixture.owner.recovery_port().inspect_effects(&handle),
        Err(RuntimeWorldRecoveryDenial::Busy)
    ));
    drop(prepared);
    assert!(fixture
        .owner
        .recovery_port()
        .inspect_effects(&handle)
        .unwrap()
        .retains_signal_definition());
    assert_eq!(
        fixture
            .bridge
            .conditional_lifecycle_probe()
            .bridge_conditional_retention()
            .unwrap(),
        retained
    );

    let cancelled = RuntimeWorldCancellationSource::new();
    let prepared = fixture
        .owner
        .publication_port()
        .prepare_bridge_conditional_definition_adoption(
            &handle,
            &fixture.bridge,
            &cancelled.token(),
            None,
        )
        .unwrap();
    cancelled.cancel();
    assert!(matches!(
        fixture
            .owner
            .publication_port()
            .execute_bridge_conditional_definition_adoption(
                execution,
                prepared,
                &fixture.bridge,
                &cancelled.token()
            ),
        RuntimeWorldConditionalDefinitionPublicationOutcome::NoEffect(_)
    ));
    assert!(fixture
        .owner
        .recovery_port()
        .inspect_effects(&handle)
        .unwrap()
        .retains_signal_definition());
    assert_eq!(
        fixture
            .bridge
            .conditional_lifecycle_probe()
            .bridge_conditional_retention()
            .unwrap(),
        retained
    );

    let prepared = fixture
        .owner
        .publication_port()
        .prepare_bridge_conditional_definition_adoption(
            &handle,
            &fixture.bridge,
            &cancellation.token(),
            None,
        )
        .unwrap();
    let (publication, lowering) = match fixture
        .owner
        .publication_port()
        .execute_bridge_conditional_definition_adoption(
            execution,
            prepared,
            &fixture.bridge,
            &cancellation.token(),
        ) {
        RuntimeWorldConditionalDefinitionPublicationOutcome::Performed {
            publication,
            lowering,
        } => (publication, lowering),
        RuntimeWorldConditionalDefinitionPublicationOutcome::NoEffect(denial) => {
            panic!("recovery refused: {denial:?}")
        }
        RuntimeWorldConditionalDefinitionPublicationOutcome::ProductUnpublished(partial) => {
            panic!("recovery remained partial: {:?}", partial.effects())
        }
    };
    assert_eq!(
        publication
            .component_results()
            .relational_publication_identity(),
        Some(relational)
    );
    assert_eq!(publication.cost_counters().relational_owner_contacts(), 0);
    assert_eq!(publication.cost_counters().signal_owner_contacts(), 1);
    assert_eq!(lowering.signal_definition_generation(), 2);
    assert_eq!(fixture.bridge.installed_conditional_definition_count(), 2);
    let current = fixture
        .owner
        .observation_port()
        .observe_product_branch(fixture.root.branch_identity())
        .unwrap();
    let bound = fixture
        .bridge
        .admit_conditional_signal_basis(&lowering, current.basis().signal_basis())
        .unwrap();
    let session = fixture.bridge.admit_conditional_evaluation(worth_runtime_bridge::facade::BridgeConditionalEvaluationAdmissionRequest::source_free_at_signal_basis(&bound), execution).unwrap();
    super::super::execute_source_free(&fixture.bridge, &lowering, &session, 2);
    fixture
        .owner
        .recovery_port()
        .release_effects(&handle, 0)
        .unwrap();
    drop(publication.consume());
}

#[test]
fn signal_movement_recovery_adopts_both_original_results_after_activation_refusal() {
    for cancel_after_signal in [false, true] {
        let fixture = definition_world();
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
        let candidate = bridge_candidate(&fixture);
        let (outcome, custody) = fixture
            .owner
            .publication_port()
            .execute_conditional_definition_with_signal(
                execution,
                prepared,
                candidate,
                &mut (),
                &cancellation.token(),
                |transaction, custody| {
                    custody.retain_applied(
                        fixture
                            .bridge
                            .apply_owned_conditional_installation_extension(
                                transaction,
                                custody.take_prepared(),
                            )
                            .unwrap(),
                    );
                    Ok(())
                },
                |custody| {
                    if cancel_after_signal {
                        custody.complete_activation(&fixture.bridge)?;
                        cancellation.cancel();
                        Ok(())
                    } else {
                        Err(worth_signal::facade::SignalError::invalid_input(
                            "declined activation before consuming either completion capability",
                        ))
                    }
                },
            );
        let partial = match outcome {
            crate::facade::RuntimeWorldPublicationOutcome::ProductUnpublished(partial) => partial,
            other => panic!("real Signal movement must retain original result: {other:?}"),
        };
        let handle = partial.recovery_handle();
        assert_eq!(
            partial.cause(),
            if cancel_after_signal {
                crate::facade::ProductUnpublishedCause::CancellationAfterEffect
            } else {
                crate::facade::ProductUnpublishedCause::SiblingOwnerDenied
            }
        );
        let relational = partial
            .component_results()
            .relational_publication_identity();
        let signal = partial.component_results().signal_publication_identity();
        assert_eq!(
            partial.progress().signal_posture(),
            SignalAttemptProgressPosture::Performed
        );
        assert_eq!(fixture.bridge.installed_conditional_definition_count(), 1);
        assert_eq!(
            fixture
                .owner
                .observation_port()
                .observe_product_branch(fixture.root.branch_identity())
                .unwrap()
                .snapshot(),
            fixture.root.snapshot()
        );
        drop(partial);
        drop(custody);
        let cancellation = RuntimeWorldCancellationSource::new();
        let prepared = fixture
            .owner
            .publication_port()
            .prepare_bridge_conditional_definition_adoption(
                &handle,
                &fixture.bridge,
                &cancellation.token(),
                None,
            )
            .unwrap();
        let (publication, lowering) = match fixture
            .owner
            .publication_port()
            .execute_bridge_conditional_definition_adoption(
                execution,
                prepared,
                &fixture.bridge,
                &cancellation.token(),
            ) {
            RuntimeWorldConditionalDefinitionPublicationOutcome::Performed {
                publication,
                lowering,
            } => (publication, lowering),
            RuntimeWorldConditionalDefinitionPublicationOutcome::NoEffect(denial) => {
                panic!("adoption refused: {denial:?}")
            }
            RuntimeWorldConditionalDefinitionPublicationOutcome::ProductUnpublished(partial) => {
                panic!("adoption remained partial: {:?}", partial.effects())
            }
        };
        assert_eq!(
            publication
                .component_results()
                .relational_publication_identity(),
            relational
        );
        assert_eq!(
            publication
                .component_results()
                .signal_publication_identity(),
            signal
        );
        assert_eq!(publication.cost_counters().relational_owner_contacts(), 0);
        assert_eq!(publication.cost_counters().signal_owner_contacts(), 0);
        assert_eq!(lowering.signal_definition_generation(), 2);
        assert_eq!(fixture.bridge.installed_conditional_definition_count(), 2);
        fixture
            .owner
            .recovery_port()
            .release_effects(&handle, 0)
            .unwrap();
        drop(publication.consume());
    }
}
