//! The one finite cabinet-family inventory shared by its program recipes.
use super::*;

pub(in super::super) fn feature_specs(
    captured: ApplicationFeatureSpec,
) -> Vec<ApplicationFeatureSpec> {
    vec![
        predecessor::program::feature(),
        diamond::feature(),
        decision_input::feature(),
        parity_input::feature(),
        saturated_input::feature(),
        cross_reader::feature(),
        cross_converge::feature(),
        cross_oscillate::feature(),
        convergent::feature(),
        ordinary_input::feature(),
        decision_write::feature(),
        synchronous_edit::reconciliation::feature(),
        synchronous_edit::reconciliation::child_feature(),
        captured,
        borrowed_inputs::program::feature(),
    ]
}
