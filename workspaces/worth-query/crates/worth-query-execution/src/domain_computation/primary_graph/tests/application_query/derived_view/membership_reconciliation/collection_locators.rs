//! Installed collection fields whose revisions govern membership and labels.
use super::*;
use crate::domain_computation::primary_graph::tests::fixture::AuthorizationWorld;

pub(super) fn membership_and_label(
    world: &AuthorizationWorld,
) -> (AspectFieldLocator, AspectFieldLocator) {
    let graph = world.application.runtime.primary_graph().unwrap();
    let tag_field = AccountMembershipTag::reference();
    let tag_locator = graph
        .layout
        .field_locator(tag_field.entity(), tag_field.aspect(), tag_field.field())
        .unwrap()
        .clone();
    let label_field = AccountLabel::reference();
    let label_locator = graph
        .layout
        .field_locator(
            label_field.entity(),
            label_field.aspect(),
            label_field.field(),
        )
        .unwrap()
        .clone();
    (tag_locator, label_locator)
}
