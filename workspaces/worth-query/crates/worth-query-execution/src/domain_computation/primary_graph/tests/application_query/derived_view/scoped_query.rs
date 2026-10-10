use super::*;
use crate::domain_computation::primary_graph::tests::fixture::{
    AccountSummaryResult, PublicScopedAccountSummaryQuery,
};
use crate::domain_computation::primary_graph::WorthQueryApplicationOneShotResult;

#[test]
fn declared_public_occurrence_query_reads_only_its_typed_scope() {
    let world = installed_authorization_world(true);
    let request = live_scope();
    let external = world.authenticate("alice", Duration::from_secs(60), &request);
    let selected = world
        .application
        .with_host_advancement(|phase| {
            let host_custody = phase.test_host_request();
            world.selected_product(&(phase).test_host_request())
        })
        .expect("the fixture read owns its installed host call");
    let principal = world
        .application
        .with_host_advancement(|phase| {
            let host_custody = phase.test_host_request();
            selected
                .resolve_authenticated_principal(
                    &phase,
                    &world.binding,
                    &external,
                    &request,
                    WorthQueryPrincipalResolutionMode::Ordinary,
                )
                .unwrap()
        })
        .expect("the fixture read owns its installed host call");
    let open = world
        .application
        .with_host_advancement(|phase| {
            let host_custody = phase.test_host_request();
            selected
                .resolve_entity(
                    &phase,
                    AccountStatus::reference(),
                    "open".to_string(),
                    &request,
                    WorthQueryPrincipalResolutionMode::Ordinary,
                )
                .unwrap()
        })
        .expect("the fixture read owns its installed host call");
    let query = world
        .application
        .installed_schema()
        .certification_query(PublicScopedAccountSummaryQuery::reference())
        .unwrap();
    let access = WorthQueryApplicationQueryAccessContext::new(&principal, &open);
    let result: WorthQueryApplicationOneShotResult<
        PublicScopedAccountSummaryQuery,
        AccountSummaryResult,
    > = world
        .application
        .with_host_advancement(|phase| {
            let host_custody = phase.test_host_request();
            world
                .application
                .execute_application_query_one_shot(
                    &phase,
                    selected
                        .admit_application_query(
                            &phase,
                            &query,
                            &access,
                            ApplicationQueryParameterSet::new(),
                            current_controls(&request),
                        )
                        .unwrap(),
                )
                .unwrap()
        })
        .expect("the fixture read owns its installed host call");
    assert_eq!(result.rows().len(), 1);
    assert_eq!(result.rows()[0].label(), "primary");
    assert_eq!(
        result.observed_sources()[0]
            .managed_derived_view_key()
            .root(),
        open.entity_id()
    );
}
