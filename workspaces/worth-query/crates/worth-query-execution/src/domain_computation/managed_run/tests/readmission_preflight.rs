use super::readmission_direct::yielded_direct;
use super::*;

#[test]
fn foreign_bridge_denies_before_fresh_query_or_provider_work_and_preserves_retry() {
    crate::domain_computation::primary_graph::with_test_advancement(|active_phase| {
        let phase = &active_phase;
        let execution = phase;
        let active_request = execution;

        let (yielded, bridge, runtime) = yielded_direct(execution);
        let checkpoint = yielded.inspection().checkpoint().identity().to_owned();
        let resource_attempt = yielded.inspection().yielded_attempt_identity().to_owned();
        let foreign_bridge = super::causal_fixture::managed_admission_context().bridge;
        let denied = match yielded.readmit_same_runtime(active_request, &runtime, &foreign_bridge) {
            crate::domain_computation::WorthQueryDirectReadmissionOutcome::Denied(denied) => denied,
            _ => panic!("foreign Bridge runtime should deny during Bridge preflight"),
        };
        assert_eq!(
        denied.kind(),
        crate::domain_computation::WorthQueryDirectReadmissionDenialKind::BridgeReadmissionDenied
    );
        let evidence = denied.readmission_evidence();
        let counters = evidence.query_counters();
        assert_eq!(counters.preflight_check_count(), 1);
        assert_eq!(counters.fresh_resource_attempt_count(), 0);
        assert_eq!(counters.bridge_readmission_attempt_count(), 0);
        assert_eq!(counters.provider_restore_attempt_count(), 0);
        let bridge_counters = evidence
            .bridge_counters()
            .expect("foreign Bridge denial must carry Bridge preflight evidence");
        assert_eq!(bridge_counters.preflight_check_count(), 1);
        assert_eq!(bridge_counters.signal_attempt_admission_count(), 0);
        assert_eq!(bridge_counters.abort_count(), 0);
        assert_eq!(bridge_counters.commit_count(), 0);
        let yielded = denied.into_yielded();
        assert_eq!(yielded.inspection().checkpoint().identity(), checkpoint);
        assert_eq!(
            yielded.inspection().yielded_attempt_identity(),
            resource_attempt
        );

        let active = match yielded.readmit_same_runtime(active_request, &runtime, &bridge) {
            crate::domain_computation::WorthQueryDirectReadmissionOutcome::Readmitted(
                readmitted,
            ) => readmitted.into_active(),
            _ => panic!("foreign-Bridge denial must preserve exact retry authority"),
        };
        let completion = match active.advance(execution) {
            WorthQueryDirectGraphStepOutcome::Completed(completion) => completion,
            _ => panic!("restored provider should complete"),
        };
        assert!(completion
            .into_running()
            .completed()
            .unwrap()
            .cleanup()
            .is_ok());
    });
}

#[test]
fn crossed_bridge_denial_preserves_both_interleaved_direct_peers() {
    crate::domain_computation::primary_graph::with_test_advancement(|active_phase| {
        let phase = &active_phase;
        let execution = phase;
        let active_request = execution;

        let (first, first_bridge, first_runtime) = yielded_direct(execution);
        let (second, second_bridge, second_runtime) = yielded_direct(execution);

        let first = match first.readmit_same_runtime(active_request, &first_runtime, &second_bridge)
        {
            crate::domain_computation::WorthQueryDirectReadmissionOutcome::Denied(denied) => {
                assert_eq!(
                denied.kind(),
                crate::domain_computation::WorthQueryDirectReadmissionDenialKind::
                    BridgeReadmissionDenied
            );
                assert_eq!(
                    denied
                        .readmission_evidence()
                        .query_counters()
                        .fresh_resource_attempt_count(),
                    0
                );
                denied.into_yielded()
            }
            _ => panic!("a peer Bridge runtime must not readmit the first direct run"),
        };

        let first = match first.readmit_same_runtime(active_request, &first_runtime, &first_bridge)
        {
            crate::domain_computation::WorthQueryDirectReadmissionOutcome::Readmitted(
                readmitted,
            ) => readmitted.into_active(),
            _ => panic!("the first owner must retain its rightful readmission"),
        };
        let second =
            match second.readmit_same_runtime(active_request, &second_runtime, &second_bridge) {
                crate::domain_computation::WorthQueryDirectReadmissionOutcome::Readmitted(
                    readmitted,
                ) => readmitted.into_active(),
                _ => panic!("the untouched peer must retain its rightful readmission"),
            };

        for active in [first, second] {
            let completion = match active.advance(execution) {
                WorthQueryDirectGraphStepOutcome::Completed(completion) => completion,
                _ => panic!("each rightful peer must complete independently"),
            };
            assert!(completion
                .into_running()
                .completed()
                .unwrap()
                .cleanup()
                .is_ok());
        }
    });
}

#[test]
fn successor_installation_denies_stale_yield_before_any_fresh_authority() {
    crate::domain_computation::primary_graph::with_test_advancement(|active_phase| {
        let phase = &active_phase;
        let execution = phase;
        let active_request = execution;

        let (yielded, _bridge, mut runtime) = yielded_direct(execution);
        let checkpoint = yielded.inspection().checkpoint().identity().to_owned();
        runtime
            .commit_successor_installation(Arc::new(
                runtime.installed_packages().successor_generation(),
            ))
            .expect("successor installation should commit");
        let bridge = super::causal_fixture::managed_admission_context().bridge;
        let denied = match yielded.readmit_same_runtime(active_request, &runtime, &bridge) {
            crate::domain_computation::WorthQueryDirectReadmissionOutcome::Denied(denied) => denied,
            _ => panic!("stale yielded installation should deny"),
        };
        assert_eq!(
        denied.kind(),
        crate::domain_computation::WorthQueryDirectReadmissionDenialKind::StaleInstallationGeneration
    );
        let evidence = denied.readmission_evidence();
        let counters = evidence.query_counters();
        assert_eq!(counters.fresh_resource_attempt_count(), 0);
        assert_eq!(counters.bridge_readmission_attempt_count(), 0);
        assert_eq!(counters.provider_restore_attempt_count(), 0);
        assert!(evidence.bridge_counters().is_none());
        let yielded = denied.into_yielded();
        assert_eq!(yielded.inspection().checkpoint().identity(), checkpoint);
        assert_eq!(
            complete_direct_yield_cleanup(yielded)
                .checkpoint()
                .unwrap()
                .identity(),
            checkpoint
        );
    });
}
