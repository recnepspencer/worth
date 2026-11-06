use std::time::Duration;

use worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_contribution::producer::demand::selection::selected_basis::source_basis_is_admitted;
use worth_query_execution::facade::domain_computation::WorthQueryProductBranchReadIdentity;
use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::world_installation::installed_authorization_world;
use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::schema_types::live_scope;
use worth_query_testkit_execution_primary_graph_application_attempt_2::domain_computation::primary_graph::tests::live_delivery_support::commit_live_activity;
use worth_query_execution::facade::domain_computation::WorthQueryPrincipalResolutionMode;

#[test]
fn owner_retained_program_basis_survives_a_new_head_without_admitting_foreign_or_direct_reads() {
    let world = installed_authorization_world(
        true,
        worth_query_admission::facade::authenticated_principal::WorthQueryAdmissionClock::system(),
    );
    let selected = world
        .application
        .with_host_advancement(|host_phase| {
            world.selected_product(&host_phase.test_host_request_for_test())
        })
        .expect("installed host call opens");
    let source = WorthQueryProductBranchReadIdentity::from_observation_for_test(
        selected.product().observation_for_test(),
    );
    let retained = selected.retain_application_read();
    let request = live_scope();
    let external = world.authenticate("alice", Duration::from_secs(60), &request);
    let principal = world
        .application
        .with_host_advancement(|host_phase| {
            world
                .selected_product(&host_phase.test_host_request_for_test())
                .resolve_authenticated_principal(
                    &host_phase,
                    &world.binding,
                    &external,
                    &request,
                    WorthQueryPrincipalResolutionMode::Ordinary,
                )
                .expect("the source owner authenticates")
        })
        .expect("installed host call opens");
    commit_live_activity(&world, &principal, &request);
    let selected_current = world
        .application
        .with_host_advancement(|host_phase| {
            world.selected_product(&host_phase.test_host_request_for_test())
        })
        .expect("installed host call opens");
    let current = selected_current.product().observation_for_test();
    assert!(!source.matches_observation_for_test(current));
    assert!(world
        .application
        .with_host_advancement(|host_phase| world
            .application
            .select_application_read_observation(&host_phase, &retained)
            .is_ok())
        .expect("installed host call opens"));
    assert!(source_basis_is_admitted(&source, current, true));
    assert!(!source_basis_is_admitted(&source, current, false));

    let foreign = installed_authorization_world(
        true,
        worth_query_admission::facade::authenticated_principal::WorthQueryAdmissionClock::system(),
    );
    let selected_foreign = foreign
        .application
        .with_host_advancement(|host_phase| {
            foreign.selected_product(&host_phase.test_host_request_for_test())
        })
        .expect("installed host call opens");
    let foreign_current = selected_foreign.product().observation_for_test();
    assert!(!source_basis_is_admitted(&source, foreign_current, true));
    assert!(foreign
        .application
        .with_host_advancement(|host_phase| foreign
            .application
            .select_application_read_observation(&host_phase, &retained)
            .is_err())
        .expect("installed host call opens"));
}
