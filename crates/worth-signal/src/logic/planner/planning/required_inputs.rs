use crate::data::error::SignalError;
use crate::data::graph::SignalGraph;
use crate::data::handle::NodeId;
use crate::data::request_preparation::{self as preparation_budget, SignalPreparationBudget};

/// The current causal edges plus the maximum observation envelope needed to
/// execute this node. Declared inputs are planning prerequisites only.
pub(super) fn required_input_sources(
    graph: &mut SignalGraph,
    node: NodeId,
    mut work: Option<&mut worth_execution::MapKernelContext<'_, '_>>,
    preparation: Option<&mut SignalPreparationBudget>,
) -> Result<Vec<NodeId>, SignalError> {
    let current = graph.current_runtime_dependencies_of(node)?.len();
    let declared = graph
        .get_contract(node)?
        .execution
        .bounded_inputs
        .as_ref()
        .map_or(0, |inputs| inputs.as_slice().len());
    super::super::precompute::work::checkpoint(
        work.as_deref_mut(),
        current.saturating_add(declared).saturating_add(1),
    )?;
    // Checked planning observes existing topology. Dead-edge maintenance belongs
    // to reconciliation, after the request has admitted its work and memory.
    let dependencies = if work.is_some() {
        graph.current_runtime_dependencies_of(node)?
    } else {
        graph.runtime_dependencies_of(node)?
    };
    let capacity = current
        .checked_add(declared)
        .ok_or_else(|| SignalError::invalid_input("Signal input capacity overflow"))?;
    preparation_budget::claim_vec::<NodeId>(preparation, capacity)?;
    let mut sources = Vec::with_capacity(capacity);
    sources.extend(dependencies.iter().map(|edge| edge.source()));
    if let Some(inputs) = &graph.get_contract(node)?.execution.bounded_inputs {
        sources.extend(inputs.as_slice().iter().map(|input| input.source));
    }
    sources.sort_unstable();
    sources.dedup();
    Ok(sources)
}
