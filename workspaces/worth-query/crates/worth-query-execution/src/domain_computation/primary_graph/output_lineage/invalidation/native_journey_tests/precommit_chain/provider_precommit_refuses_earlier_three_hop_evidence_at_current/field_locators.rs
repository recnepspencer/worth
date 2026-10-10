use super::*;
use crate::domain_computation::primary_graph::tests::fixture::AuthorizationWorld;
use worth_foundational::facade::AspectFieldLocator;
pub(super) fn status_and_label(
    world: &AuthorizationWorld,
) -> (AspectFieldLocator, AspectFieldLocator) {
    let graph = world.application.runtime.primary_graph().unwrap();
    let status_ref = AccountStatus::reference();
    let label_ref = AccountLabel::reference();
    let status = graph
        .layout()
        .field_locator(status_ref.entity(), status_ref.aspect(), status_ref.field())
        .unwrap()
        .clone();
    let label = graph
        .layout()
        .field_locator(label_ref.entity(), label_ref.aspect(), label_ref.field())
        .unwrap()
        .clone();
    (status, label)
}
