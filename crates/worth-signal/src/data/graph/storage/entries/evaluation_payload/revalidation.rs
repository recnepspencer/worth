//! Installation of the waiter owner's already-resolved node projection.
use crate::data::graph::PendingRevalidationNodeProjection;
use crate::data::node::{NodeHotData, NodeWarmData};

pub(in crate::data::graph) fn install_revalidation_resolution(
    hot: &mut NodeHotData,
    warm: &mut NodeWarmData,
    projected: PendingRevalidationNodeProjection,
) {
    warm.pending_dependency_revalidation = projected.pending;
    hot.state = projected.state;
}
