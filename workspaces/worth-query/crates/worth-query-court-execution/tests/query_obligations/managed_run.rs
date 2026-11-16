//! Value-only terminal custody rules; physical release provenance is a Host court claim.
use worth_foundational::facade::{HomeIdentity, ManagedRunObligationIdentity};
use worth_query_execution::facade::{
    domain_computation::{
        WorthQueryManagedRunCleanupDisposition as Cleanup,
        WorthQueryManagedRunTerminalKind as Terminal,
    },
    query_obligations::*,
};

#[test]
fn managed_run_history_preserves_failed_terminal_and_refuses_unrecorded_release() {
    let home = HomeIdentity::from_observed_bytes([51; 16]).unwrap();
    let identity = ManagedRunObligationIdentity::from_recorded(home, std::num::NonZeroU64::MIN);
    let mut ledger = QueryObligationLedger::empty(home);
    let mut generations = QueryObligationPublicationGenerations::from_journal(&ledger);
    let frame = ledger
        .prepare(
            QueryManagedRunRecord::Retain(QueryManagedRunObligation {
                identity,
                logical_run: "actual-original".to_owned(),
                original_attempt: "actual-attempt".to_owned(),
                attempt: "actual-attempt".to_owned(),
                posture: QueryManagedRunPosture::Active,
                original: None,
                recovery: None,
            }),
            &mut generations,
        )
        .unwrap();
    ledger
        .readmit(QueryObligationFrame::readmit(frame.bytes()).unwrap())
        .unwrap();
    assert_eq!(
        ledger.prepare(
            QueryManagedRunRecord::Discharge { identity },
            &mut generations
        ),
        Err(QueryObligationDenial::ManagedRunPending { identity })
    );
    let cause = QueryManagedRunRecovery::Cleanup(
        worth_query_execution::facade::domain_computation::WorthQueryManagedRunCleanupFailureKind::ProviderMemoryRetained);
    let frame = ledger
        .prepare(
            QueryManagedRunRecord::Recovery { identity, cause },
            &mut generations,
        )
        .unwrap();
    ledger
        .readmit(QueryObligationFrame::readmit(frame.bytes()).unwrap())
        .unwrap();
    let failed = QueryManagedRunPosture::DirectCleanup {
        terminal: Terminal::Failed,
        disposition: Cleanup::RecoveryRequired,
        resources: worth_query_execution::facade::query_obligations::QueryManagedRunResources {
            native_released: false,
            checkpoint: None,
            provider_execution: None,
            restored_execution: None,
        },
    };
    let frame = ledger
        .prepare(
            QueryManagedRunRecord::Handoff {
                identity,
                posture: failed,
                attempt: "actual-attempt".to_owned(),
            },
            &mut generations,
        )
        .unwrap();
    ledger
        .readmit(QueryObligationFrame::readmit(frame.bytes()).unwrap())
        .unwrap();
    assert_eq!(ledger.managed_runs().next().unwrap().posture, failed);
    assert_eq!(
        ledger.prepare(
            QueryManagedRunRecord::Handoff {
                identity,
                attempt: "actual-attempt".into(),
                posture: QueryManagedRunPosture::DirectCleanup {
                    terminal: Terminal::Completed,
                    disposition: Cleanup::CleanupComplete,
                    resources: QueryManagedRunResources {
                        native_released: false,
                        checkpoint: None,
                        provider_execution: None,
                        restored_execution: None
                    },
                },
            },
            &mut generations
        ),
        Err(QueryObligationDenial::ObligationSubjectMismatch),
        "failed work cannot be relabelled Completed"
    );
    assert_eq!(
        ledger.prepare(
            QueryManagedRunRecord::Discharge { identity },
            &mut generations
        ),
        Err(QueryObligationDenial::ManagedRunPending { identity })
    );
    let released = QueryManagedRunPosture::DirectCleanup {
        terminal: Terminal::Failed,
        disposition: Cleanup::RecoveryRequired,
        resources: worth_query_execution::facade::query_obligations::QueryManagedRunResources {
            native_released: true,
            checkpoint: None,
            provider_execution: None,
            restored_execution: None,
        },
    };
    let frame = ledger
        .prepare(
            QueryManagedRunRecord::Handoff {
                identity,
                posture: released,
                attempt: "actual-attempt".to_owned(),
            },
            &mut generations,
        )
        .unwrap();
    ledger
        .readmit(QueryObligationFrame::readmit(frame.bytes()).unwrap())
        .unwrap();
    let retained = ledger.managed_runs().next().unwrap();
    assert_eq!(retained.posture, released);
    assert_eq!(
        retained.recovery,
        Some(cause),
        "successful release cannot erase the original failure"
    );
    let frame = ledger
        .prepare(
            QueryManagedRunRecord::Discharge { identity },
            &mut generations,
        )
        .unwrap();
    ledger
        .readmit(QueryObligationFrame::readmit(frame.bytes()).unwrap())
        .unwrap();
    assert!(ledger.managed_runs().next().is_none());
}

// These are protocol inputs, not physical-release evidence. Each case records a
// successful native/basis release while one distinct provider resource failed.
#[test]
fn managed_run_discharge_requires_each_recorded_provider_release() {
    use worth_query_execution::facade::domain_computation::{
        WorthQueryProviderCheckpointReleaseDisposition as Checkpoint,
        WorthQueryProviderExecutionDestructorDisposition as Destructor,
        WorthQueryProviderExecutionDisposalDisposition as Disposal,
    };
    let home = HomeIdentity::from_observed_bytes([52; 16]).unwrap();
    let complete = QueryManagedExecutionRelease {
        disposal: Disposal::Completed,
        destructor: Destructor::Completed,
    };
    let baseline = QueryManagedRunResources {
        native_released: true,
        checkpoint: Some(Checkpoint::Released),
        provider_execution: Some(complete),
        restored_execution: Some(complete),
    };
    for resources in [
        QueryManagedRunResources {
            checkpoint: Some(Checkpoint::Panicked),
            ..baseline
        },
        QueryManagedRunResources {
            provider_execution: Some(QueryManagedExecutionRelease {
                disposal: Disposal::Rejected,
                ..complete
            }),
            ..baseline
        },
        QueryManagedRunResources {
            restored_execution: Some(QueryManagedExecutionRelease {
                destructor: Destructor::Panicked,
                ..complete
            }),
            ..baseline
        },
    ] {
        let mut ledger = QueryObligationLedger::empty(home);
        let mut generations = QueryObligationPublicationGenerations::from_journal(&ledger);
        let identity = ManagedRunObligationIdentity::from_recorded(home, std::num::NonZeroU64::MIN);
        let frame = ledger
            .prepare(
                QueryManagedRunRecord::Retain(QueryManagedRunObligation {
                    identity,
                    logical_run: "work".into(),
                    original_attempt: "attempt".into(),
                    attempt: "attempt".into(),
                    posture: QueryManagedRunPosture::Active,
                    original: None,
                    recovery: None,
                }),
                &mut generations,
            )
            .unwrap();
        ledger
            .readmit(QueryObligationFrame::readmit(frame.bytes()).unwrap())
            .unwrap();
        let frame = ledger
            .prepare(
                QueryManagedRunRecord::Handoff {
                    identity,
                    attempt: "attempt".into(),
                    posture: QueryManagedRunPosture::ReadmissionCleanup {
                        disposition: Cleanup::RecoveryRequired,
                        resources,
                    },
                },
                &mut generations,
            )
            .unwrap();
        ledger
            .readmit(QueryObligationFrame::readmit(frame.bytes()).unwrap())
            .unwrap();
        let retained = ledger.managed_runs().next().unwrap();
        assert_eq!(
            retained.posture,
            QueryManagedRunPosture::ReadmissionCleanup {
                disposition: Cleanup::RecoveryRequired,
                resources
            }
        );
        assert_eq!(
            ledger.prepare(
                QueryManagedRunRecord::Discharge { identity },
                &mut generations
            ),
            Err(QueryObligationDenial::ManagedRunPending { identity })
        );
    }
}

#[test]
fn workflow_owner_data_retains_pending_artifacts_after_native_resource_release() {
    for route in [
        QueryManagedWorkflowCleanupRoute::Terminal(Terminal::Failed),
        QueryManagedWorkflowCleanupRoute::Yielded,
        QueryManagedWorkflowCleanupRoute::Readmission,
    ] {
        let home = HomeIdentity::from_observed_bytes([67; 16]).unwrap();
        let identity = ManagedRunObligationIdentity::from_recorded(home, std::num::NonZeroU64::MIN);
        let mut ledger = QueryObligationLedger::empty(home);
        let mut generations = QueryObligationPublicationGenerations::from_journal(&ledger);
        let retained = ledger
            .prepare(
                QueryManagedRunRecord::Retain(QueryManagedRunObligation {
                    identity,
                    logical_run: "workflow-computation".into(),
                    original_attempt: "original-attempt".into(),
                    attempt: "original-attempt".into(),
                    posture: QueryManagedRunPosture::Active,
                    original: None,
                    recovery: None,
                }),
                &mut generations,
            )
            .unwrap();
        ledger
            .readmit(QueryObligationFrame::readmit(retained.bytes()).unwrap())
            .unwrap();
        let resources = QueryManagedRunResources {
            native_released: true,
            checkpoint: None,
            provider_execution: None,
            restored_execution: None,
        };
        let pending = ledger
            .prepare(
                QueryManagedRunRecord::Handoff {
                    identity,
                    attempt: "original-attempt".into(),
                    posture: QueryManagedRunPosture::WorkflowCleanup {
                        route,
                        disposition: Cleanup::CleanupPending,
                        resources,
                        artifacts_released: false,
                    },
                },
                &mut generations,
            )
            .unwrap();
        let mut cold = QueryObligationLedger::empty(home);
        for frame in [retained, pending] {
            cold.readmit(QueryObligationFrame::readmit(frame.bytes()).unwrap())
                .unwrap();
        }
        let mut generations = QueryObligationPublicationGenerations::from_journal(&cold);
        assert_eq!(
            cold.prepare(
                QueryManagedRunRecord::Discharge { identity },
                &mut generations
            ),
            Err(QueryObligationDenial::ManagedRunPending { identity }),
            "empty fresh resources cannot release the original artifact"
        );
        assert!(
            matches!(cold.managed_runs().next().unwrap().posture, QueryManagedRunPosture::WorkflowCleanup { route: retained, artifacts_released: false, .. } if retained == route)
        );
        let released = cold
            .prepare(
                QueryManagedRunRecord::Handoff {
                    identity,
                    attempt: "original-attempt".into(),
                    posture: QueryManagedRunPosture::WorkflowCleanup {
                        route,
                        disposition: Cleanup::CleanupComplete,
                        resources,
                        artifacts_released: true,
                    },
                },
                &mut generations,
            )
            .unwrap();
        cold.readmit(QueryObligationFrame::readmit(released.bytes()).unwrap())
            .unwrap();
        let discharge = cold
            .prepare(
                QueryManagedRunRecord::Discharge { identity },
                &mut generations,
            )
            .unwrap();
        cold.readmit(QueryObligationFrame::readmit(discharge.bytes()).unwrap())
            .unwrap();
        assert!(cold.managed_runs().next().is_none());
    }
}
