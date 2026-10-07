//! Selected semantic projection and retained fact payload growth.
use std::mem::size_of;

use crate::data::error::SignalError;
use crate::data::graph::SignalGraph;
use crate::data::handle::NodeId;
use crate::data::retained_storage::RetainedStoragePreparation as Work;
use crate::logic::explain::RewiringDependency;

pub(super) fn measure(
    graph: &SignalGraph,
    node: NodeId,
    scope_heap: u64,
    old_edges_heap: u64,
    links: usize,
    work: &mut Work,
) -> Result<u64, SignalError> {
    let rewiring_heap = (links as u64)
        .checked_mul(size_of::<RewiringDependency>() as u64)
        .and_then(|bytes| bytes.checked_add(old_edges_heap))
        .and_then(|bytes| bytes.checked_add(scope_heap))
        .ok_or_else(overflow)?;
    graph
        .epoch_semantic_diagnostic_fixed_bound(
            node,
            graph.epoch_node_clone_charge(node, work)?.bytes(),
            rewiring_heap,
            links,
            work,
        )?
        .checked_add(
            graph
                .diagnostics_state()
                .epoch_fact_replacement_capacity_bound(node, work)?,
        )
        .ok_or_else(overflow)
}

fn overflow() -> SignalError {
    SignalError::invalid_input("selected diagnostic capacity overflow")
}
