pub(crate) use worth_query_bank_testkit_server::facade::estate_operations::phase8_safe_retry_completed::*;

#[test]
fn safe_retry_of_already_completed_effect_adds_no_contact_and_no_physical_consequence() {
    let world = world::cross_gate_world("safe-retry-completed");
    let route = world
        .fixture
        .world
        .runtime
        .install_estate_rail_completion_verifier(Arc::new(EstateRailSource))
        .expect("the estate rail source installs once");
    world.transport.under(FaultScript::Succeed, world::PATIENT);
    let (receipt, source) = world.commit_with_source(
        bank_domain::proposals::BankIdempotencyKey::new("completed-notice-correction").unwrap(),
    );
    assert_eq!(
        receipt
            .external_dispatch_posture()
            .map(|posture| posture.kind()),
        Some(WorthQueryPublishedExternalEffectPostureKind::Completed)
    );
    let correlation = world.transport.attempts()[0].clone();
    let token: [u8; 32] = correlation.token().try_into().expect("32-byte token");
    let terminal = world
        .fixture
        .world
        .runtime
        .observe_estate_rail_completion(&route, token)
        .expect("synchronous completion entered the one terminal owner");

    let mut handle = world.open_recovery(&receipt);
    let specialist = world.fixture.authenticate_specialist();
    let action = world.specialist_action();
    let scope = request_scope();
    let coordinates = source.coordinates();
    assert_eq!(
        coordinates.world_commit(),
        terminal.original_world_commit().ordinal()
    );
    assert_eq!(
        coordinates.world_owner(),
        terminal.original_world_commit().owner_identity().get()
    );
    // This is the generic original-source entrance, before selecting any inverse family.
    // A legacy external aftermath name does not install a typed correction association.
    let request = world.fixture.world.runtime.request(&specialist, &scope);
    let corrections = request.corrections();
    let selection = corrections.select_coordinates(
        &coordinates,
        worth_query_host::facade::application_entry::WorthQueryCorrectionHistoryLimits::new(
            None,
            std::num::NonZeroUsize::MIN,
        ),
    );
    assert!(matches!(selection,
        Err(worth_query_host::facade::application_entry::WorthQueryApplicationCorrectionDenial::Source(
            worth_query_host::facade::primary_graph::WorthQueryCorrectionSelectionDenial::SourceNotCorrectable
        ))
    ), "completed external notice has no installed typed correction");
    // Any contact would run this fault path; completion must win before
    // transport admission, so the rail is never asked (9.17.7 race court).
    world
        .transport
        .under(FaultScript::DuplicateAcknowledgement, world::PATIENT);
    for retry in 0..2 {
        let denied = world
            .fixture
            .world
            .runtime
            .safe_retry_commit_recovery(handle, &specialist, action, &scope)
            .expect_err("an already-completed effect admits no new physical attempt");
        let (denied, retained) = denied.into_parts();
        match denied {
            BankEstateProgressionDenial::Recovery(denied) => assert_eq!(
                denied.kind(),
                BankRecoveryDenialKind::AlreadyCompleted,
                "retry {retry} must name the terminal completion"
            ),
            other => panic!("expected AlreadyCompleted, got {other:?}"),
        }
        handle = retained.expect("an already-completed refusal returns the live handle");
        // (rail contacts, adapter crossings, rail admissions, consequences)
        let observed = (
            world.transport.attempts().len(),
            world.transport.production_dispatches().len(),
            world.transport.admission_count(),
            world.transport.completed_effect_count(),
        );
        assert_eq!(observed, (1, 1, 1, 1), "retry {retry} contacted the rail");
        let status = world.transport.ledger_status(&correlation);
        assert_eq!(status, LedgerStatus::Completed);
        let still = world
            .fixture
            .world
            .runtime
            .observe_estate_rail_completion(&route, token)
            .expect("the terminal owner keeps its completion");
        assert_eq!(
            (
                still.original_world_commit(),
                still.completion_world_commit()
            ),
            (
                terminal.original_world_commit(),
                terminal.completion_world_commit()
            ),
            "retry {retry} must keep the original terminal provenance"
        );
    }
}
