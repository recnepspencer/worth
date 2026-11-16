//! No Home installs generic managed providers. These are owner DATA courts,
//! not simulated operation authority or claims of physical release.
use worth_foundational::facade::{HomeIdentity, ManagedRunObligationIdentity};
use worth_query_execution::facade::{
    domain_computation::{
        WorthQueryManagedRunCleanupDisposition as Cleanup,
        WorthQueryYieldRecoveryKind as YieldRecovery,
    },
    query_obligations::*,
};
fn append(
    ledger: &mut QueryObligationLedger,
    generations: &mut QueryObligationPublicationGenerations,
    record: QueryManagedRunRecord,
) -> Vec<u8> {
    let frame = ledger.prepare(record, generations).unwrap();
    let bytes = frame.bytes().to_vec();
    ledger
        .readmit(QueryObligationFrame::readmit(&bytes).unwrap())
        .unwrap();
    bytes
}

#[test]
fn yielded_owner_data_preserves_terminal_cause_and_refuses_unrecorded_release() {
    use worth_runtime_bridge::facade::BridgeExecutionBasisFinalizationFailureKind as Finalization;
    use worth_runtime_bridge::facade::BridgeExecutionBasisSignalTerminal as Signal;
    let terminalized = QueryManagedYieldRecoveryState::Terminalized;
    for (cause, state) in [
        (
            YieldRecovery::SignalAttemptAlreadyTerminal(Signal::Cancelled),
            terminalized,
        ),
        (
            YieldRecovery::SignalAttemptAlreadyTerminal(Signal::TimedOut),
            terminalized,
        ),
        (
            YieldRecovery::SignalAttemptAlreadyTerminal(Signal::Rejected),
            terminalized,
        ),
        (
            YieldRecovery::BridgeTerminalization(Finalization::ManagedQueueOccupied),
            QueryManagedYieldRecoveryState::Running,
        ),
    ] {
        let home = HomeIdentity::from_observed_bytes([65; 16]).unwrap();
        let mut ledger = QueryObligationLedger::empty(home);
        let mut generations = QueryObligationPublicationGenerations::from_journal(&ledger);
        let identity = ManagedRunObligationIdentity::from_recorded(home, std::num::NonZeroU64::MIN);
        let retain = append(
            &mut ledger,
            &mut generations,
            QueryManagedRunRecord::Retain(QueryManagedRunObligation {
                identity,
                logical_run: "retained-work".into(),
                original_attempt: "original-attempt".into(),
                attempt: "original-attempt".into(),
                posture: QueryManagedRunPosture::Active,
                original: None,
                recovery: None,
            }),
        );
        let cause = QueryManagedRunRecovery::Yield { cause, state };
        let recovery = append(
            &mut ledger,
            &mut generations,
            QueryManagedRunRecord::Recovery { identity, cause },
        );
        let mut frames = vec![retain, recovery];
        if state == terminalized {
            frames.push(append(
                &mut ledger,
                &mut generations,
                QueryManagedRunRecord::Handoff {
                    identity,
                    attempt: "original-attempt".into(),
                    posture: QueryManagedRunPosture::YieldedCleanup {
                        disposition: Cleanup::RecoveryRequired,
                        resources: QueryManagedRunResources {
                            native_released: false,
                            checkpoint: None,
                            provider_execution: None,
                            restored_execution: None,
                        },
                    },
                },
            ));
        }
        let mut cold = QueryObligationLedger::empty(home);
        for bytes in frames {
            cold.readmit(QueryObligationFrame::readmit(&bytes).unwrap())
                .unwrap();
        }
        let row = cold.managed_runs().next().unwrap();
        assert_eq!(row.recovery, Some(cause));
        assert_eq!(row.identity, identity);
        if state == terminalized {
            assert!(matches!(
                row.posture,
                QueryManagedRunPosture::YieldedCleanup {
                    disposition: Cleanup::RecoveryRequired,
                    ..
                }
            ));
        } else {
            assert_eq!(
                row.posture,
                QueryManagedRunPosture::Active,
                "Running still needs its live abandonment owner"
            );
        }
        let mut generations = QueryObligationPublicationGenerations::from_journal(&cold);
        assert_eq!(
            cold.prepare(
                QueryManagedRunRecord::Discharge { identity },
                &mut generations
            ),
            Err(QueryObligationDenial::ManagedRunPending { identity })
        );
    }
}
