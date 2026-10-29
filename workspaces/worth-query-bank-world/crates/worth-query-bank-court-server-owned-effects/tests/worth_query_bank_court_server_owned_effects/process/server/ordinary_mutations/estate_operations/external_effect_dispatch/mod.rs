pub(crate) use worth_query_bank_testkit_server::facade::estate_operations::external_effect_dispatch::*;

pub(crate) mod committed_outbox;

pub(crate) mod duplicate_acknowledgement;

pub(crate) mod preparation_denial;

#[test]
fn a_succeeding_rail_completes_and_no_fault_ever_reports_completion() {
    let expectations: [(
        &str,
        FaultScript,
        Duration,
        Option<WorthQueryPublishedExternalEffectFailure>,
    ); 6] = [
        ("succeed", FaultScript::Succeed, PATIENT, None),
        (
            "acknowledge-only",
            FaultScript::AcknowledgeWithoutCompleting,
            PATIENT,
            None,
        ),
        (
            "duplicate-ack",
            FaultScript::DuplicateAcknowledgement,
            PATIENT,
            Some(WorthQueryPublishedExternalEffectFailure::DuplicatedAcknowledgement),
        ),
        (
            "slow-completion",
            FaultScript::CompleteAfterDelay { delay_millis: 400 },
            IMPATIENT,
            Some(WorthQueryPublishedExternalEffectFailure::Timeout),
        ),
        (
            "lost-response",
            FaultScript::CommitThenLoseResponse,
            PATIENT,
            Some(WorthQueryPublishedExternalEffectFailure::LostResponse),
        ),
        (
            "disappearance",
            FaultScript::DisappearMidDispatch,
            PATIENT,
            Some(WorthQueryPublishedExternalEffectFailure::Disconnect),
        ),
    ];

    for (scenario, script, frame_timeout, expected_fault) in expectations {
        assert_dispatch_scenario(scenario, script, frame_timeout, expected_fault);
    }
}

#[test]
fn unsupported_version_survives_the_rail_adapter_and_publication_boundary() {
    let world =
        dispatch_world_with_protocol_support("predating-v1", RailProtocolSupportProfile::V2Only);
    let receipt = world.commit_notification(52);

    assert_eq!(
        receipt
            .external_dispatch_posture()
            .and_then(|posture| posture.failure()),
        Some(
            WorthQueryPublishedExternalEffectFailure::UnsupportedProtocolVersion {
                produced: 1,
                posture: WorthQueryPublishedUnsupportedProtocolVersionPosture::PredatesWindow,
            }
        )
    );
    assert_eq!(world.transport.admission_count(), 0);
    assert_eq!(
        world.transport.ledger_status(&rail_correlation(&world)),
        LedgerStatus::NoRecord
    );
}

/// Q8.25-C3: the rail learns what the notice *means*, not merely that one
/// happened.
///
/// The assertion is against the rail's own ledger, read back over a fresh TCP
/// connection to a different OS process, which holds a notice its own decoder
/// produced from the bytes the outbox co-committed. Before slice 9B the wire
/// carried a correlation family and a 32-byte token and nothing else: the
/// estate, the notice, and the subject stopped at the outbox row, and any real
/// external owner would have had to look them up elsewhere or assume them.
/// Nothing in this file could have detected that, because nothing asked the
/// rail what it understood.
#[test]
fn the_rail_decodes_the_committed_notice_rather_than_echoing_a_correlation() {
    let world = dispatch_world("decoded-notice");
    world.transport.under(FaultScript::Succeed, PATIENT);
    let receipt = world.commit_notification(47);

    assert_eq!(
        receipt
            .external_dispatch_posture()
            .map(|posture| posture.kind()),
        Some(WorthQueryPublishedExternalEffectPostureKind::Completed),
        "a well-formed notice on a healthy rail completes"
    );

    let decoded = world
        .transport
        .ledger_notice(&rail_correlation(&world))
        .expect("the rail admits an attempt only by first decoding its payload");
    assert_eq!(
        decoded.estate(),
        world.fixture.estate.get(),
        "the rail read the estate this commit actually notified"
    );
    assert_eq!(
        decoded.notice(),
        world.fixture.notice.get(),
        "the rail read the exact death notice, not some other notice on the estate"
    );
    assert_eq!(
        decoded.subject(),
        world.fixture.deceased.get(),
        "the rail read the subject the notice is about"
    );
}

/// The rail holds no meaning it did not decode.
///
/// A correlation the rail never received cannot yield a notice, so a passing
/// `ledger_notice` assertion above cannot be satisfied by a rail that answers
/// optimistically.
#[test]
fn an_unknown_correlation_yields_no_notice_from_the_rail() {
    let world = dispatch_world("unknown-correlation");
    world.transport.under(FaultScript::Succeed, PATIENT);
    let _ = world.commit_notification(49);

    let stranger = RailCorrelation::new(ESTATE_DEATH_NOTICE_RAIL, vec![0xAB; 32]);
    assert_eq!(
        world.transport.ledger_notice(&stranger),
        None,
        "the rail reports meaning only for attempts it actually decoded"
    );
}

#[test]
fn late_reconciliation_observes_rail_completion_without_dispatching_again() {
    let world = dispatch_world("late-reconciliation");
    world.transport.under(
        FaultScript::CompleteAfterDelay { delay_millis: 300 },
        IMPATIENT,
    );
    let receipt = world.commit_notification(43);

    let posture = receipt
        .external_dispatch_posture()
        .expect("the declared effect must be dispatched");
    assert_eq!(
        posture.failure(),
        Some(WorthQueryPublishedExternalEffectFailure::Timeout)
    );

    std::thread::sleep(Duration::from_millis(700));
    let correlation = rail_correlation(&world);
    assert_eq!(
        world.transport.ledger_status(&correlation),
        LedgerStatus::Completed,
        "the rail completes on its own schedule regardless of the caller's deadline"
    );
    assert_eq!(
        world.transport.attempts().len(),
        1,
        "reconciliation reads the rail's ledger; it never dispatches a second effect"
    );
    assert_eq!(
        world.notice_status(),
        DeathNoticeStatus::NotificationRequested,
        "the durable bank truth advanced exactly once"
    );
}
