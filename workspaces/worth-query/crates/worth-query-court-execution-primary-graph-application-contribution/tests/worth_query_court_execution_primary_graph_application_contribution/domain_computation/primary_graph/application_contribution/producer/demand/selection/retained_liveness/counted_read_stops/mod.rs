use worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_contribution::producer::demand::selection::retained_liveness::denial;
use worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_contribution::producer::demand::selection::retained_liveness::read;
use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::world_installation::installed_authorization_world;
use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::Account;
use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::AccountStatus;
use worth_relational::facade::transactions::RelationalExecutionDenialCause as Cause;
#[test]
fn retained_output_liveness_door_preserves_stops_at_every_consult() {
    let world = installed_authorization_world(
        true,
        worth_query_admission::facade::authenticated_principal::WorthQueryAdmissionClock::system(),
    );
    world
        .application
        .with_host_advancement(|phase| {
            let execution = phase.borrowed_request_for_test();
            let snapshot = world.invariant.snapshot(&phase).unwrap();
            let account = snapshot
                .entities(&execution, Account::reference())
                .unwrap()
                .remove(0);
            let entity = account.entity_id();
            assert!(snapshot
                .field(&execution, &account, AccountStatus::at("AccountStatus"))
                .unwrap()
                .is_some());
            let selected = world.selected_product(&phase.test_host_request_for_test());
            (*world.application.runtime_for_test())
                .primary_graph_for_test()
                .unwrap()
                .integration_handle_for_test()
                .with_runtime(&execution, |runtime| {
                    let view = runtime
                        .read_truth()
                        .project_snapshot(selected.application_basis_for_test().snapshot_handle())
                        .unwrap();
                    worth_query_execution::test_read_stops::every_consult_under(
                        &execution,
                        |request| read(request, &view, entity).map_err(|denied| denied.kind()),
                        denial(Cause::Cancelled).kind(),
                        denial(Cause::WorkExhausted).kind(),
                    );
                });
        })
        .unwrap();
}
