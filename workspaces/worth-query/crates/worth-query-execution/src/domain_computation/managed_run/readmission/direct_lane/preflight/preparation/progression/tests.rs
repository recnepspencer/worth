use super::super::prepare_direct_provider_restore;
use super::recovery::{
    WorthQueryDirectReadmissionRecoveryKind, WorthQueryDirectReadmissionRecoveryPosture,
    WorthQueryDirectReadmissionRecoveryRequired, WorthQueryDirectReadmissionYieldReassemblyOutcome,
};
use super::restore_direct;
use crate::domain_computation::managed_run::readmission::WorthQueryDirectReadmissionOutcome;
use crate::domain_computation::managed_run::tests::readmission_direct::yielded_direct_with_provider;
use crate::domain_computation::managed_run::tests::yield_fixture::YieldProvider;

#[test]
fn bridge_cleanup_failure_returns_exact_owner_retry_authority() {
    crate::domain_computation::primary_graph::with_test_advancement(|active_phase| {
        let bootstrap = active_phase.bootstrap_for_test();
        let resource_request = bootstrap.execution_request();

        let phase = &active_phase;
        let execution = phase;

        let (yielded, bridge, runtime) = yielded_direct_with_provider(
            execution,
            YieldProvider::checkpoint_restore_failure(7),
            resource_request,
        );
        let checkpoint = yielded.inspection().checkpoint().identity().to_owned();
        let (pending, progress) = match prepare_direct_provider_restore(yielded, &runtime, &bridge)
        {
            Ok(prepared) => prepared,
            Err(_) => panic!("owner-thread Query phases must reach provider restore"),
        };
        let recovery = std::thread::spawn(move || {
            crate::domain_computation::primary_graph::with_test_advancement(|phase| {
                match restore_direct(
                    phase
                        .request_for_source(0)
                        .expect("standalone restore fixture"),
                    pending,
                    &bridge,
                    progress,
                ) {
                    WorthQueryDirectReadmissionOutcome::RecoveryRequired(recovery) => recovery,
                    _ => panic!("foreign-thread rollback must retain Bridge cleanup authority"),
                }
            })
        })
        .join()
        .expect("Bridge cleanup recovery must remain in-process");

        assert_eq!(
            recovery.kind(),
            WorthQueryDirectReadmissionRecoveryKind::BridgeCleanupFailed
        );
        assert!(recovery.detail().contains("belongs to thread"));
        assert_eq!(
            recovery.posture(),
            WorthQueryDirectReadmissionRecoveryPosture::YieldReassemblyPending
        );
        let recovery = match recovery {
            WorthQueryDirectReadmissionRecoveryRequired::YieldReassembly(recovery) => recovery,
            _ => panic!("Bridge cleanup failure must expose only yield-reassembly authority"),
        };
        let reassembled = match recovery.retry_to_yielded() {
            WorthQueryDirectReadmissionYieldReassemblyOutcome::Yielded(reassembled) => reassembled,
            _ => panic!("Signal owner thread must reconstruct exact yielded Query authority"),
        };
        let bridge_counters = reassembled
            .readmission_evidence()
            .bridge_counters()
            .expect("successful owner cleanup must carry final Bridge evidence");
        assert_eq!(bridge_counters.abort_count(), 1);
        assert_eq!(bridge_counters.commit_count(), 0);
        let yielded = reassembled.into_yielded();
        assert_eq!(yielded.inspection().checkpoint().identity(), checkpoint);

        let cleanup = match yielded.cleanup() {
            crate::domain_computation::WorthQueryDirectYieldCleanupOutcome::Complete(receipt) => {
                receipt
            }
            crate::domain_computation::WorthQueryDirectYieldCleanupOutcome::RecoveryRequired(_) => {
                panic!("reassembled yielded authority must clean up on its owner thread")
            }
        };
        assert!(cleanup.inspection().resources_released());
        assert_eq!(cleanup.inspection().released_reservation_count(), 2);
    });
}
