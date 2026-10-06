//! Per-source subscriber insertion ceilings for a selected epoch prefix.
use std::collections::BTreeMap;

use crate::data::error::SignalError;
use crate::data::graph::SignalGraph;
use crate::data::handle::NodeId;
use crate::data::request_preparation::SignalPreparationBudget;

use super::overflow;

pub(super) fn measure(
    graph: &SignalGraph,
    prior: &BTreeMap<NodeId, (usize, u64)>,
    sources: &[NodeId],
    budget: &mut SignalPreparationBudget,
) -> Result<(u64, Vec<(NodeId, usize, u64)>), SignalError> {
    budget.claim_vec::<(NodeId, usize, u64)>(sources.len())?;
    let mut growth = 0_u64;
    let mut next = Vec::with_capacity(sources.len());
    for &source in sources {
        let (prior_additions, prior_bound) = prior.get(&source).copied().unwrap_or((0, 0));
        let additions = prior_additions.checked_add(1).ok_or_else(overflow)?;
        let bound = graph.epoch_subscriber_source_capacity_bound(source, additions)?;
        growth = growth
            .checked_add(bound.checked_sub(prior_bound).ok_or_else(overflow)?)
            .ok_or_else(overflow)?;
        next.push((source, additions, bound));
    }
    Ok((growth, next))
}
