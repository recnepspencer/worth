use worth_query_testkit_execution_primary_graph_application_attempt_2::domain_computation::managed_run::tests::provisional_attempt_fixture::ProvisionalProvider;
use worth_query_testkit_execution_primary_graph_application_attempt_2::domain_computation::managed_run::tests::provisional_attempt_fixture::ProvisionalProviderState;
#[test]
fn one_installed_anchor_retains_convergence() {
    let provider = ProvisionalProvider {
        state: std::sync::Arc::new(std::sync::Mutex::new(ProvisionalProviderState::default())),
    };
    let anchor = worth_query_execution::facade::integration::WorthQueryGraphProviderAnchor::install_convergent_invariant_capable::<
        worth_query_testkit_execution_primary_graph_application_attempt_2::domain_computation::managed_run::tests::ManagedGraph,
        _,
    >(provider);
    assert!(anchor.retains_convergence_and_phase_seven_through_ten_for_test());
}
