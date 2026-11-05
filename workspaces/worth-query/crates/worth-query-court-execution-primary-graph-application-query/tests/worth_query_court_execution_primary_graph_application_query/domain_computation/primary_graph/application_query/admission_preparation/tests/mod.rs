use worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_query::admission_preparation::court_doors_for_test::AdmissionPreparationDoorsOps;
use std::sync::Arc;

use worth_query_installation::facade::WorthQueryApplicationQueryInstallationDenialKind;

use worth_query_execution::facade::domain_computation::WorthQueryApplicationQueryAdmissionDenialKind;
use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::capability_world_installation::installed_capability_live_world_with_label;
use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::GovernedLiveAccountActivityQuery;

#[test]
fn published_schema_validation_keeps_query_and_generation_authority() {
    let mut world = installed_capability_live_world_with_label("installed");
    let local_query = world
        .application
        .installed_schema()
        .certification_query(GovernedLiveAccountActivityQuery::reference())
        .unwrap();
    assert!(world
        .application
        .validate_installed_query_for_test(&local_query)
        .is_ok());

    let foreign = installed_capability_live_world_with_label("foreign");
    let foreign_query = foreign
        .application
        .installed_schema()
        .certification_query(GovernedLiveAccountActivityQuery::reference())
        .unwrap();
    assert_eq!(
        world
            .application
            .validate_installed_query_for_test(&foreign_query)
            .unwrap_err()
            .kind(),
        WorthQueryApplicationQueryAdmissionDenialKind::InstalledQuery(
            WorthQueryApplicationQueryInstallationDenialKind::ForeignRuntime,
        )
    );

    let rebuilt = Arc::new(
        (*world.application.runtime_for_test())
            .installed_packages_for_test()
            .rebuild(),
    );
    (*world.application.runtime_mut_for_test())
        .execution_mut_for_test()
        .replace_rebuilt_installation(rebuilt)
        .unwrap();
    assert!(world
        .application
        .validate_installed_query_for_test(&local_query)
        .is_ok());

    let successor = Arc::new(
        (*world.application.runtime_for_test())
            .installed_packages_for_test()
            .successor_generation(),
    );
    (*world.application.runtime_mut_for_test())
        .execution_mut_for_test()
        .commit_successor_installation(successor)
        .unwrap();
    assert_eq!(
        world
            .application
            .validate_installed_query_for_test(&local_query)
            .unwrap_err()
            .kind(),
        WorthQueryApplicationQueryAdmissionDenialKind::InstalledQuery(
            WorthQueryApplicationQueryInstallationDenialKind::StaleGeneration,
        )
    );
}
