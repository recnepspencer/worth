//! Separate checkpoint/restored-execution facts at their durable owner.
//! Generic-provider authority is not available from Home and is never forged here.
use worth_foundational::facade::{HomeIdentity, ManagedRunObligationIdentity};
use worth_query_execution::facade::{
    domain_computation::{
        WorthQueryDirectReadmissionRecoveryKind as RestoreRecovery,
        WorthQueryManagedRunCleanupDisposition as Cleanup,
        WorthQueryProviderCheckpointReleaseDisposition as Checkpoint,
        WorthQueryProviderExecutionDestructorDisposition as Destructor,
        WorthQueryProviderExecutionDisposalDisposition as Disposal,
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
fn managed_restore_owner_data_keeps_original_failure_and_separate_release_facts() {
    let released = QueryManagedExecutionRelease {
        disposal: Disposal::Completed,
        destructor: Destructor::Completed,
    };
    for (cause, checkpoint, restored, can_discharge) in [
        (
            RestoreRecovery::ProviderRestorePanicked,
            Checkpoint::Panicked,
            None,
            false,
        ),
        (
            RestoreRecovery::ProviderRestoreRejectedAfterExecutionAdmission,
            Checkpoint::Released,
            Some(released),
            true,
        ),
        (
            RestoreRecovery::ProviderRestorePanicked,
            Checkpoint::Released,
            Some(QueryManagedExecutionRelease {
                destructor: Destructor::Panicked,
                ..released
            }),
            false,
        ),
    ] {
        let home = HomeIdentity::from_observed_bytes([66; 16]).unwrap();
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
        let cause = QueryManagedRunRecovery::Readmission(cause);
        let recovery = append(
            &mut ledger,
            &mut generations,
            QueryManagedRunRecord::Recovery { identity, cause },
        );
        let handoff = append(
            &mut ledger,
            &mut generations,
            QueryManagedRunRecord::Handoff {
                identity,
                attempt: "original-attempt".into(),
                posture: QueryManagedRunPosture::ReadmissionCleanup {
                    disposition: Cleanup::RecoveryRequired,
                    resources: QueryManagedRunResources {
                        native_released: true,
                        checkpoint: Some(checkpoint),
                        provider_execution: Some(released),
                        restored_execution: restored,
                    },
                },
            },
        );
        let mut cold = QueryObligationLedger::empty(home);
        for bytes in [retain, recovery, handoff] {
            cold.readmit(QueryObligationFrame::readmit(&bytes).unwrap())
                .unwrap();
        }
        let row = cold.managed_runs().next().unwrap();
        assert_eq!(row.identity, identity);
        assert_eq!(row.recovery, Some(cause));
        let QueryManagedRunPosture::ReadmissionCleanup {
            resources,
            disposition,
        } = row.posture
        else {
            panic!("typed terminal recovery");
        };
        assert_eq!(resources.checkpoint, Some(checkpoint));
        assert_eq!(resources.restored_execution, restored);
        assert_eq!(disposition, Cleanup::RecoveryRequired);
        let mut generations = QueryObligationPublicationGenerations::from_journal(&cold);
        let discharge = cold.prepare(
            QueryManagedRunRecord::Discharge { identity },
            &mut generations,
        );
        if can_discharge {
            cold.readmit(QueryObligationFrame::readmit(discharge.unwrap().bytes()).unwrap())
                .unwrap();
            assert!(cold.managed_runs().next().is_none());
        } else {
            assert_eq!(
                discharge,
                Err(QueryObligationDenial::ManagedRunPending { identity })
            );
        }
    }
}
