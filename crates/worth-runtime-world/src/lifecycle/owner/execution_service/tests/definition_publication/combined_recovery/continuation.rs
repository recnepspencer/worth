use super::*;

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
