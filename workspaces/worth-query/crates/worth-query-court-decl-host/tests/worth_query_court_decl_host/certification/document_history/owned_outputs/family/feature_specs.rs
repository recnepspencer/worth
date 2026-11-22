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
        saturated_consumer::feature(),
        capped_reader::feature(),
        capped_consumer::feature(),
        super::c3b::cycle_parent::feature(),
        super::c3b::cycle_consumer::feature(),
        super::c3b::guarded_input::feature(),
        super::c3b::guarded_member::feature(),
        super::c3b::guarded_presence::feature(),
        super::c3b::guarded_current::feature(),
        super::c3b::guarded_first::feature(),
        super::c3b::decision_write_n::feature(),
        super::c3b::self_relation::feature(),
        self_write::feature(),
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
