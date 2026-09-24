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
    let selected = world.selected_product();
    let principal = selected
        .resolve_authenticated_principal(
            &world.binding,
            &external,
            &request,
            WorthQueryPrincipalResolutionMode::Ordinary,
        )
        .unwrap();
    let open = selected
        .resolve_entity(
            AccountStatus::reference(),
            "open".to_string(),
            &request,
            WorthQueryPrincipalResolutionMode::Ordinary,
        )
        .unwrap();
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
        .execute_application_query_one_shot(
            selected
                .admit_application_query(
                    &query,
                    &access,
                    ApplicationQueryParameterSet::new(),
                    current_controls(&request),
                )
                .unwrap(),
        )
        .unwrap();
    assert_eq!(result.rows().len(), 1);
    assert_eq!(result.rows()[0].label(), "primary");
    assert_eq!(
        result.observed_sources()[0]
            .managed_derived_view_key()
            .root(),
        open.entity_id()
    );
}
