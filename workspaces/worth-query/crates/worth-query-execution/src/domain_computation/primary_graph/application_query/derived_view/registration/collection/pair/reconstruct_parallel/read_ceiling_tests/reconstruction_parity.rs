//! Answers and work at real read/reconstruction doors, also run on master.
use super::*;
use crate::domain_computation::primary_graph::application_query::{
    authorized_read::WorthQueryAuthorizedApplicationReadDenial,
    one_shot::map_authorized_read_denial, read_execution::read_bounded_root_rows,
};
use crate::domain_computation::primary_graph::WorthQueryApplicationOneShotDenialKind as OneShot;

#[test]
fn unavailable_predicate_snapshot_keeps_masters_kind_and_field_subject() {
    let _observation =
        crate::facade::primary_graph::WorthQueryExecutionObservation::install_on_this_thread();
    let world = installed_authorization_world(true);
    let scope = live_scope();
    let external = world.authenticate("alice", Duration::from_secs(60), &scope);
    world
        .application
        .with_host_advancement(|phase| {
            let host = phase.test_host_request();
            let request = host.borrowed_request_for_test();
            let selected = world.selected_product(&host);
            let principal = selected
                .resolve_authenticated_principal(
                    &phase,
                    &world.binding,
                    &external,
                    &scope,
                    WorthQueryPrincipalResolutionMode::Ordinary,
                )
                .unwrap();
            let root = selected
                .resolve_entity(
                    &phase,
                    AccountIdentity::reference(),
                    "account-1".into(),
                    &scope,
                    WorthQueryPrincipalResolutionMode::Ordinary,
                )
                .unwrap();
            let access = WorthQueryApplicationQueryAccessContext::new(&principal, &root);
            let query = world
                .application
                .installed_schema()
                .certification_query(LiveAccountActivityQuery::reference())
                .unwrap();
            let plan = selected
                .retain_selection(&request)
                .unwrap()
                .admit_application_query(
                    &phase,
                    &query,
                    &access,
                    live_account_parameters("account-1"),
                    current_controls(&scope),
                )
                .unwrap();
            let runtime =
                worth_relational::facade::runtime::RelationalRuntimeApi::builder().build();

            let read = match read_bounded_root_rows(
                &runtime,
                &request,
                &world.application.primary_provider.graph.layout,
                &plan,
                None,
            ) {
                Err(denial) => denial,
                Ok(_) => panic!("foreign snapshot unexpectedly read"),
            };
            let outer = map_authorized_read_denial(
                WorthQueryAuthorizedApplicationReadDenial::Read(read),
                query.name(),
            );
            assert_eq!(outer.kind(), OneShot::PredicateIndexUnavailable);
            assert_eq!(outer.subject(), "AccountIdentity");
        })
        .unwrap();
}

#[test]
fn unavailable_root_path_snapshot_keeps_masters_kind_and_query_subject() {
    let _observation =
        crate::facade::primary_graph::WorthQueryExecutionObservation::install_on_this_thread();
    let world = installed_authorization_world(true);
    let scope = live_scope();
    let external = world.authenticate("alice", Duration::from_secs(60), &scope);
    world
        .application
        .with_host_advancement(|phase| {
            let host = phase.test_host_request();
            let request = host.borrowed_request_for_test();
            let selected = world.selected_product(&host);
            let principal = selected
                .resolve_authenticated_principal(
                    &phase,
                    &world.binding,
                    &external,
                    &scope,
                    WorthQueryPrincipalResolutionMode::Ordinary,
                )
                .unwrap();
            let root = selected
                .resolve_entity(
                    &phase,
                    AccountIdentity::reference(),
                    "account-1".into(),
                    &scope,
                    WorthQueryPrincipalResolutionMode::Ordinary,
                )
                .unwrap();
            let access = WorthQueryApplicationQueryAccessContext::new(&principal, &root);
            let query = world
                .application
                .installed_schema()
                .certification_query(CrossRootQuery::reference())
                .unwrap();
            let plan = selected
                .retain_selection(&request)
                .unwrap()
                .admit_application_query(
                    &phase,
                    &query,
                    &access,
                    ApplicationQueryParameterSet::new(),
                    current_controls(&scope),
                )
                .unwrap();
            let runtime =
                worth_relational::facade::runtime::RelationalRuntimeApi::builder().build();

            let read = match read_bounded_root_rows(
                &runtime,
                &request,
                &world.application.primary_provider.graph.layout,
                &plan,
                None,
            ) {
                Err(denial) => denial,
                Ok(_) => panic!("foreign snapshot unexpectedly read"),
            };
            let outer = map_authorized_read_denial(
                WorthQueryAuthorizedApplicationReadDenial::Read(read),
                query.name(),
            );
            assert_eq!(outer.kind(), OneShot::TraversalUnavailable);
            assert_eq!(outer.subject(), query.name());
        })
        .unwrap();
}
