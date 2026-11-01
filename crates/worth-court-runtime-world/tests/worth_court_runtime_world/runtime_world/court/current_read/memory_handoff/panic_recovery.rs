use super::*;

#[test]
fn callback_panics_preserve_only_genuine_effects_and_original_recovery_can_settle_and_release() {
    for perform in [false, true] {
        let request = crate::test_execution_policy::serial_request();
        let execution = worth_execution::ExecutionRequest::serial(&request);
        let court = CompositeSupplyChainCourt::compile();
        let current = court.bootstrap();
        let target = fork(&court, &execution, &current);
        let token = RuntimeWorldCancellationSource::new().token();
        let prepared = prepare(&court, &execution, &target, &current);
        let handle = prepared.unpublished_recovery_handle();
        let panic = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            court
                .world
                .publication_port()
                .execute_without_signal_with_memory_cutover(
                    execution,
                    prepared,
                    &token,
                    |cutover| {
                        if perform {
                            cutover.perform();
                        }
                        panic!("caller panic after perform={perform}");
                    },
                )
        }));
        assert!(panic.is_err());
        assert_eq!(court.observe(&current), current);
        assert_eq!(court.observe(&target), target);
        let recovery = court.world.recovery_port();
        let actual_native = direct_basis(&court, &target);
        if !perform {
            assert_eq!(actual_native, *target.basis().relational_basis());
            assert!(matches!(
                recovery.inspect_effects(&handle),
                Err(RuntimeWorldRecoveryDenial::MissingRecord)
            ));
            let (_, outcome) = court
                .world
                .publication_port()
                .execute_without_signal_with_memory_cutover(
                    execution,
                    prepare(&court, &execution, &target, &current),
                    &token,
                    |cutover| cutover.perform(),
                );
            let RuntimeWorldPublicationOutcome::Performed(done) = outcome else {
                panic!("pre-effect panic releases original reservations")
            };
            drop(done.consume());
            continue;
        }
        assert_ne!(actual_native, *target.basis().relational_basis());
        assert!(matches!(
            recovery.release_effects(&handle, 0),
            Err(RuntimeWorldRecoveryDenial::SettlementRequired)
        ));
        let effects = recovery.inspect_effects(&handle).unwrap();
        assert_eq!(effects.owner_effect_count(), 1);
        let moved = court.publish_cargo(&current, "9");
        assert_eq!(grain(&court, &execution, &court.observe(&current)), "9");
        let continued = recovery.continue_effects(effects).unwrap();
        assert!(continued
            .actions()
            .contains(&ProductUnpublishedNextAction::ReleaseObligations));
        assert_eq!(direct_basis(&court, &target), actual_native);
        assert_eq!(
            court.records.read(&execution, &actual_native).records["grain"],
            "5"
        );
        let effects = recovery.inspect_effects(&handle).unwrap();
        // Abandonment carries exact identity repair, not a fabricated full
        // settlement result. That receipt permits cleanup, never adoption.
        assert!(matches!(
            recovery.prepare_settled_relational_adoption(&effects, &token, None),
            Err(RuntimeWorldSettledRelationalAdoptionDenial::Recovery(
                RuntimeWorldRecoveryDenial::SettlementEvidenceUnavailable
            ))
        ));
        assert_eq!(court.observe(&target), target);
        drop(effects);
        assert!(recovery
            .release_effects(&handle, 0)
            .unwrap()
            .owner_retirement_work()
            .is_empty());
        assert_eq!(direct_basis(&court, &target), actual_native);
        drop(moved);
    }
}
