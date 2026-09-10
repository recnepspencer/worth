use super::*;

#[test]
fn activation_unwind_retains_complete_definition_custody_in_first_catalog_observation() {
    let fixture = definition_world();
    let lifecycle = fixture.bridge.conditional_lifecycle_probe();
    let baseline = lifecycle.bridge_conditional_retention().unwrap();
    let cancellation = RuntimeWorldCancellationSource::new();
    let prepared_world = fixture
        .owner
        .publication_port()
        .prepare_with_signal(
            fixture.root.clone(),
            CompositePublicationIntent::with_signal(None),
            &cancellation.token(),
            None,
        )
        .unwrap();
    let predecessor = fixture
        .bridge
        .admit_conditional_signal_basis(&fixture.lowering, fixture.root.basis().signal_basis())
        .unwrap();
    let mut prepared_bridge = fixture
        .bridge
        .prepare_owned_conditional_definition_successor(&predecessor, source_free_request(2))
        .unwrap();
    let candidate_retention = lifecycle.bridge_conditional_retention().unwrap();
    assert_eq!(candidate_retention.retained_definition_candidates(), 1);
    assert!(candidate_retention.retained_bytes() > baseline.retained_bytes());
    let publication = prepared_bridge.take_runtime_world_publication_operation();
    let unwind = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        fixture
            .owner
            .publication_port()
            .execute_conditional_definition_with_signal(
                prepared_world,
                publication,
                &mut (),
                &cancellation.token(),
                |transaction, custody| {
                    custody.retain_applied(
                        fixture
                            .bridge
                            .apply_owned_conditional_installation_extension(
                                transaction,
                                prepared_bridge,
                            )
                            .map_err(|denial| {
                                worth_signal::facade::SignalError::invalid_input(format!(
                                    "{denial:?}"
                                ))
                            })?,
                    );
                    Ok(())
                },
                |binding, custody| {
                    let mut applied = custody.lease_applied();
                    fixture
                        .bridge
                        .complete_owned_conditional_installation_activation(
                            applied.applied_mut(),
                            binding,
                        )
                        .unwrap();
                    panic!("forced unwind after definition activation preparation");
                },
            )
    }));
    assert!(unwind.is_err());
    assert_eq!(
        lifecycle.bridge_conditional_retention().unwrap(),
        candidate_retention,
        "unwind transfers the exact candidate reservation into recovery custody",
    );
    let page = fixture
        .owner
        .inspection_port()
        .recovery_page(None, std::num::NonZeroUsize::new(4).unwrap())
        .unwrap();
    assert_eq!(
        page.rows().len(),
        1,
        "the performed owner effect was not discarded as no-effect"
    );
    let handle = page.rows()[0].handle();
    let recovered = fixture
        .owner
        .recovery_port()
        .inspect_effects(handle)
        .unwrap();
    assert_eq!(
        recovered.progress().signal_posture(),
        crate::facade::SignalAttemptProgressPosture::Performed
    );
    assert_eq!(
        recovered.cause(),
        crate::facade::ProductUnpublishedCause::CallerAbandoned
    );
    assert!(
        recovered.retains_signal_definition(),
        "the first visible record contains the definition candidate"
    );
    assert_eq!(fixture.bridge.installed_conditional_definition_count(), 1);
    assert_eq!(
        fixture
            .owner
            .observation_port()
            .observe_product_branch(fixture.root.branch_identity(),)
            .unwrap()
            .snapshot(),
        fixture.root.snapshot()
    );
    drop(recovered);
    fixture
        .owner
        .recovery_port()
        .release_effects(handle, 0)
        .unwrap();
    assert_eq!(lifecycle.bridge_conditional_retention().unwrap(), baseline);
}

#[test]
fn activation_denial_retains_signal_successor_without_bridge_or_product_visibility() {
    let fixture = definition_world();
    let lifecycle = fixture.bridge.conditional_lifecycle_probe();
    let baseline = lifecycle.bridge_conditional_retention().unwrap();
    let cancellation = RuntimeWorldCancellationSource::new();
    let prepared_world = fixture
        .owner
        .publication_port()
        .prepare_with_signal(
            fixture.root.clone(),
            CompositePublicationIntent::with_signal(None),
            &cancellation.token(),
            None,
        )
        .unwrap();
    let predecessor = fixture
        .bridge
        .admit_conditional_signal_basis(&fixture.lowering, fixture.root.basis().signal_basis())
        .unwrap();
    let mut prepared_bridge = fixture
        .bridge
        .prepare_owned_conditional_definition_successor(&predecessor, source_free_request(2))
        .unwrap();
    let candidate_retention = lifecycle.bridge_conditional_retention().unwrap();
    assert_eq!(candidate_retention.retained_definition_candidates(), 1);
    assert!(candidate_retention.retained_bytes() > baseline.retained_bytes());
    let publication = prepared_bridge.take_runtime_world_publication_operation();
    let (outcome, custody) = fixture
        .owner
        .publication_port()
        .execute_conditional_definition_with_signal(
            prepared_world,
            publication,
            &mut (),
            &cancellation.token(),
            |transaction, custody| {
                custody.retain_applied(
                    fixture
                        .bridge
                        .apply_owned_conditional_installation_extension(
                            transaction,
                            prepared_bridge,
                        )
                        .map_err(|denial| {
                            worth_signal::facade::SignalError::invalid_input(format!("{denial:?}"))
                        })?,
                );
                Ok(())
            },
            |binding, custody| {
                let mut applied = custody.lease_applied();
                fixture
                    .bridge
                    .complete_owned_conditional_installation_activation(
                        applied.applied_mut(),
                        binding,
                    )
                    .unwrap();
                Err(worth_signal::facade::SignalError::invalid_input(
                    "forced activation denial",
                ))
            },
        );
    let retained_effects = match outcome {
        crate::facade::RuntimeWorldPublicationOutcome::ProductUnpublished(effects) => effects,
        other => panic!("activation denial must retain the Signal effect: {other:?}"),
    };
    drop(custody);
    let unpublished =
        crate::publication::RuntimeWorldUnpublishedConditionalDefinition::new(retained_effects);

    assert_eq!(
        lifecycle.bridge_conditional_retention().unwrap(),
        candidate_retention,
        "ProductUnpublished retains the exact candidate reservation",
    );

    assert_eq!(
        unpublished.effects().progress().signal_posture(),
        crate::facade::SignalAttemptProgressPosture::Performed
    );
    assert_eq!(fixture.bridge.installed_conditional_definition_count(), 1);
    let current = fixture
        .owner
        .observation_port()
        .observe_product_branch(fixture.root.branch_identity())
        .unwrap();
    assert_eq!(current.snapshot(), fixture.root.snapshot());
    assert!(unpublished.retains_signal_definition());
    let handle = unpublished.effects().recovery_handle();
    drop(unpublished);
    let recovered = fixture
        .owner
        .recovery_port()
        .inspect_effects(&handle)
        .expect("dropping the specialized terminal transfers both halves to the catalog");
    assert!(recovered.retains_signal_definition());
    drop(recovered);
    assert!(fixture
        .owner
        .recovery_port()
        .release_effects(&handle, 0)
        .is_ok());
    assert!(matches!(
        fixture.owner.recovery_port().inspect_effects(&handle),
        Err(crate::recovery::RuntimeWorldRecoveryDenial::MissingRecord)
    ));
    assert_eq!(fixture.bridge.installed_conditional_definition_count(), 1);
    assert_eq!(lifecycle.bridge_conditional_retention().unwrap(), baseline);
}
