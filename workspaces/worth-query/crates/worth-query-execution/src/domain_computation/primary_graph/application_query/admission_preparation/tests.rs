use std::sync::Arc;

use worth_query_installation::facade::WorthQueryApplicationQueryInstallationDenialKind;

use super::WorthQueryApplicationQueryAdmissionDenialKind;
use crate::domain_computation::primary_graph::tests::fixture::{
    installed_capability_live_world_with_label, GovernedLiveAccountActivityQuery,
};

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
        .validate_installed_query(&local_query)
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
            .validate_installed_query(&foreign_query)
            .unwrap_err()
            .kind(),
        WorthQueryApplicationQueryAdmissionDenialKind::InstalledQuery(
            WorthQueryApplicationQueryInstallationDenialKind::ForeignRuntime,
        )
    );

    let rebuilt = Arc::new(world.application.runtime.installed_packages().rebuild());
    world
        .application
        .runtime
        .replace_rebuilt_installation(rebuilt)
        .unwrap();
    assert!(world
        .application
        .validate_installed_query(&local_query)
        .is_ok());

    let successor = Arc::new(
        world
            .application
            .runtime
            .installed_packages()
            .successor_generation(),
    );
    world
        .application
        .runtime
        .commit_successor_installation(successor)
        .unwrap();
    assert_eq!(
        world
            .application
            .validate_installed_query(&local_query)
            .unwrap_err()
            .kind(),
        WorthQueryApplicationQueryAdmissionDenialKind::InstalledQuery(
            WorthQueryApplicationQueryInstallationDenialKind::StaleGeneration,
        )
    );
}
