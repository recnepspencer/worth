//! Installed collection fields whose revisions govern membership and labels.
use crate::domain_computation::primary_graph::tests::application_query::derived_view::membership_reconciliation::*;
use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::authorization_world_installation::AuthorizationWorld;

pub fn membership_and_label(
    world: &AuthorizationWorld,
) -> (AspectFieldLocator, AspectFieldLocator) {
    let graph = (*world.application.runtime_for_test()).primary_graph().unwrap();
    let tag_field = AccountMembershipTag::reference();
    let tag_locator = (*graph
        .layout_for_test())
        .field_locator_for_test(tag_field.entity(), tag_field.aspect(), tag_field.field())
        .unwrap()
        .clone();
    let label_field = AccountLabel::reference();
    let label_locator = (*graph
        .layout_for_test())
        .field_locator_for_test(
            label_field.entity(),
            label_field.aspect(),
            label_field.field(),
        )
        .unwrap()
        .clone();
    (tag_locator, label_locator)
}
