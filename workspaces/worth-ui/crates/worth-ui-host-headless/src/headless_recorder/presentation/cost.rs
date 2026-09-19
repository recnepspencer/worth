use super::*;

pub(in crate::headless_recorder) fn production_cost(
    work: UiMountedPresentationWorkView<'_>,
) -> worth_ui_host_contract::UiMountedPresentationProductionCost {
    match work {
        UiMountedPresentationWorkView::Initial(work) => work.production_cost(),
        UiMountedPresentationWorkView::Delta(work) => work.production_cost(),
        UiMountedPresentationWorkView::Reconstruction(work) => work.production_cost(),
        UiMountedPresentationWorkView::Sample(work) => work.production_cost(),
        UiMountedPresentationWorkView::Unchanged(work) => work.production_cost(),
    }
}

pub(in crate::headless_recorder) fn add_order_cost(
    base: UiHostPresentationCostReport,
    order: worth_ui_retained_order::UiRetainedOrderCost,
) -> UiHostPresentationCostReport {
    base.checked_add(UiHostPresentationCostReport::from_adapter(
        UiHostPresentationCostInput {
            order_index_lookups: order.identity_lookups(),
            order_index_node_touches: order.node_touches(),
            order_index_rotations: order.rotations(),
            order_index_high_water: order.high_water_entries(),
            ..Default::default()
        },
    ))
    .expect("profile-bounded retained-order evidence cannot overflow")
}
