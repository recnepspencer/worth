//! Production actions and receiver-scoped observation in the re-executed child.
use super::*;
pub(crate) fn continue_original(
    home: &OracleHome,
    key: u64,
    value: u64,
    pause: Option<PathBuf>,
    execution: &worth_execution::ExecutionRequest<'_, '_>,
) -> Disposition {
    let recovery = match requests::resolve_original(home.runtime(), key, value).unwrap() {
        Resolution::ProductUnpublished(effects) => effects.into_recovery(),
        Resolution::RecoveredCommitted(receipt) => {
            return Disposition::Continued(receipts::observe_original(&receipt))
        }
        other => panic!("actual original recovered effect: {other:?}"),
    };
    let _observation = pause.map(|channel| {
        home.runtime()
            .runtime()
            .world_operation_control_for_test()
            .observe_recovered_continued_for_test(move || {
                crate::killed_child::announce(
                    &channel.join("continued"),
                    b"actual Continued acknowledged",
                );
                let until = Instant::now() + Duration::from_secs(120);
                while Instant::now() < until {
                    std::thread::sleep(Duration::from_millis(10));
                }
                panic!("parent did not kill the actual disposition-paused writer");
            })
    });
    loop {
        match recovery.continue_original_in_request(execution).unwrap() {
            Continued::Pending(custody) => drop(custody),
            Continued::Published { .. } => break,
            other => panic!("original continuation retained: {other:?}"),
        }
    }
    match requests::resolve_original(home.runtime(), key, value).unwrap() {
        Resolution::RecoveredCommitted(receipt) => {
            Disposition::Continued(receipts::observe_original(&receipt))
        }
        other => panic!("original published source must resolve without re-execution: {other:?}"),
    }
}
pub(crate) fn release_original(
    home: &OracleHome,
    key: u64,
    value: u64,
    expect_continued: bool,
) -> Disposition {
    let mut recovery = match requests::resolve_original(home.runtime(), key, value).unwrap() {
        Resolution::ProductUnpublished(effects) => {
            if expect_continued {
                assert!(!effects
                    .next_actions()
                    .contains(&ProductUnpublishedNextAction::ReleaseObligations));
            }
            effects.into_recovery()
        }
        other => panic!("actual original release key: {other:?}"),
    };
    loop {
        match home
            .runtime()
            .runtime()
            .release_product_publication_recovery(recovery, 0)
        {
            Ok(_) => {
                assert!(!expect_continued);
                return Disposition::Released(1);
            }
            Err(ReleaseFailure::Recovery(failure)) => {
                if expect_continued {
                    assert_eq!(
                        failure.denial(),
                        ReleaseDenial::World(
                            RuntimeWorldRecoveryDenial::ContinuationAlreadyCommitted
                        )
                    );
                    return Disposition::ContinuedReleaseRefused;
                }
                assert_eq!(
                    failure.denial(),
                    ReleaseDenial::RecoveredPending,
                    "{:?}",
                    failure.recovered_denial()
                );
                recovery = failure.into_recovery();
            }
            Err(other) => panic!("actual recovered release: {other:?}"),
        }
    }
}
