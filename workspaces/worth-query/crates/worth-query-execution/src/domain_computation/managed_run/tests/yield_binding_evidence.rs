use super::yield_fixture::YieldProvider;
use super::*;

#[test]
fn direct_yield_exposes_one_cross_owner_authority_chain() {
    crate::domain_computation::primary_graph::with_test_advancement(|active_phase| {
        let bootstrap = active_phase.bootstrap_for_test();
        let resource_request = bootstrap.execution_request();

        let phase = &active_phase;
        let execution = phase;

        let (running, graph) = managed_graph_run_with_provider(
            WorthQueryOperationGraphAccess::Observe,
            YieldProvider::installed(5),
            resource_request,
        );
        let active = running
            .begin_graph_execution(
                execution,
                &graph,
                WorthQueryManagedGraphCallRequest::new(
                    WorthQueryGraphProviderCallKind::Observe,
                    "direct-yield-binding-evidence",
                ),
            )
            .expect("yield evidence provider should begin");
        let paused = match active.advance(execution) {
            WorthQueryDirectGraphStepOutcome::Continue(paused) => paused,
            _ => panic!("yield evidence provider did not pause"),
        };
        let yielded = match paused.yield_run() {
            crate::domain_computation::WorthQueryDirectYieldOutcome::Yielded(yielded) => yielded,
            _ => panic!("eligible evidence run did not yield"),
        };

        assert!(!yielded.inspection().operation_binding_identity().is_empty());
        assert!(!yielded
            .inspection()
            .installed_operation_identity()
            .is_empty());
        assert!(!yielded.inspection().semantic_basis_identity().is_empty());
        assert_eq!(
            yielded.inspection().installation_generation(),
            worth_query_installation::facade::WorthQueryInstallationGeneration::initial()
        );
        assert_eq!(
            yielded.inspection().provider_session_identity(),
            yielded
                .inspection()
                .provider_work()
                .provider_session_identity()
        );
        super::cost_bound::assert_exact_admission_work(yielded.inspection().run_counters());

        let cleanup = complete_direct_yield_cleanup(yielded);
        super::cost_bound::assert_exact_admission_work(cleanup.run_counters());
        assert_eq!(
            cleanup
                .checkpoint()
                .expect("yielded cleanup carries checkpoint release")
                .release_disposition(),
            crate::domain_computation::WorthQueryProviderCheckpointReleaseDisposition::Released
        );
    });
}
