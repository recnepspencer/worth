//! Runtime declaration records use semantic keys, never interner IDs or memo values.
use super::{contracts, policies, runtime_tiers, Denial, Read, Write};
use crate::data::comparator::VersionComparatorPolicy;
use crate::data::handle::NodeId;
use crate::data::node::NodeContract;
use crate::data::temporal::TemporalDuration;
use crate::data::tier::{DependencyMode, DirtyPropagation, EvaluationTrigger, TierPolicy};

pub(crate) struct RuntimeDefinitionView<'a, T: Copy + Ord> {
    pub(crate) fallback: &'a VersionComparatorPolicy,
    pub(crate) resource_deadline: Option<TemporalDuration>,
    pub(crate) policies: Vec<(&'a str, &'a TierPolicy<T>)>,
    pub(crate) computations: Vec<ComputationDefinitionView<'a>>,
    pub(crate) node_tiers: Vec<(NodeId, &'a str)>,
    pub(crate) keyed_nodes: Vec<KeyedNodeDefinitionView<'a>>,
}
pub(crate) struct ComputationDefinitionView<'a> {
    pub(crate) family: &'a str,
    pub(crate) contract: &'a NodeContract,
    pub(crate) tier: &'a str,
    pub(crate) comparator: &'a VersionComparatorPolicy,
}
pub(crate) struct KeyedNodeDefinitionView<'a> {
    pub(crate) family: &'a str,
    pub(crate) key: &'a str,
    pub(crate) node: NodeId,
}
pub(crate) struct RuntimeDefinitionRows {
    pub(crate) branch: worth_foundational::facade::DurableSignalBranchIdentity,
    pub(crate) fallback: VersionComparatorPolicy,
    pub(crate) resource_deadline: Option<TemporalDuration>,
    pub(crate) policies: Vec<RecordedTierPolicy>,
    pub(crate) computations: Vec<RecordedComputation>,
    pub(crate) node_tiers: Vec<(NodeId, String)>,
    pub(crate) keyed_nodes: Vec<RecordedKeyedNode>,
}
pub(crate) struct RecordedTierPolicy {
    pub(crate) tier: String,
    pub(crate) dependency: DependencyMode,
    pub(crate) propagation: DirtyPropagation,
    pub(crate) trigger: EvaluationTrigger,
    pub(crate) comparator: VersionComparatorPolicy,
}
pub(crate) struct RecordedComputation {
    pub(crate) family: String,
    pub(crate) contract: NodeContract,
    pub(crate) tier: String,
    pub(crate) comparator: VersionComparatorPolicy,
}
pub(crate) struct RecordedKeyedNode {
    pub(crate) family: String,
    pub(crate) key: String,
    pub(crate) node: NodeId,
}
pub(crate) fn write<T: Copy + Ord>(
    out: &mut Write<'_>,
    view: &RuntimeDefinitionView<'_, T>,
) -> Result<(), Denial> {
    policies::write_comparator(out, view.fallback)?;
    out.boolean(view.resource_deadline.is_some())?;
    if let Some(deadline) = view.resource_deadline {
        out.number(deadline.get())?;
    }
    out.count(view.policies.len())?;
    for (name, policy) in &view.policies {
        out.text(name)?;
        runtime_tiers::write(
            out,
            policy.dependency_mode,
            policy.dirty_propagation,
            policy.evaluation_trigger,
        )?;
        policies::write_comparator(out, &policy.default_comparator)?;
    }
    out.count(view.computations.len())?;
    for row in &view.computations {
        out.text(row.family)?;
        contracts::write(out, row.contract)?;
        out.text(row.tier)?;
        policies::write_comparator(out, row.comparator)?;
    }
    out.count(view.node_tiers.len())?;
    for (node, tier) in &view.node_tiers {
        out.node(out.scope_node(*node))?;
        out.text(tier)?;
    }
    out.count(view.keyed_nodes.len())?;
    for row in &view.keyed_nodes {
        out.text(row.family)?;
        out.text(row.key)?;
        out.node(out.scope_node(row.node))?;
    }
    Ok(())
}
pub(crate) fn read(
    payload: &[u8],
    materialize: bool,
    graph_instance: u64,
) -> Result<RuntimeDefinitionRows, Denial> {
    let mut input = Read::image(payload)?;
    input.graph_instance = graph_instance;
    let branch = input.branch;
    input.materialize = materialize;
    let fallback = policies::read_comparator(&mut input)?;
    let resource_deadline = if input.boolean()? {
        Some(
            TemporalDuration::new("resource runtime deadline", input.number()?)
                .map_err(|_| Denial::Malformed)?,
        )
    } else {
        None
    };
    let policies = input.list(|input| {
        let tier = input.text()?;
        let (dependency, propagation, trigger) = runtime_tiers::read(input)?;
        let comparator = policies::read_comparator(input)?;
        Ok(RecordedTierPolicy {
            tier,
            dependency,
            propagation,
            trigger,
            comparator,
        })
    })?;
    let computations = input.list(|input| {
        Ok(RecordedComputation {
            family: input.text()?,
            contract: contracts::read(input)?,
            tier: input.text()?,
            comparator: policies::read_comparator(input)?,
        })
    })?;
    let node_tiers = input.list(|input| Ok((input.node_id()?, input.text()?)))?;
    let keyed_nodes = input.list(|input| {
        Ok(RecordedKeyedNode {
            family: input.text()?,
            key: input.text()?,
            node: input.node_id()?,
        })
    })?;
    input.finish()?;
    Ok(RuntimeDefinitionRows {
        branch,
        fallback,
        resource_deadline,
        policies,
        computations,
        node_tiers,
        keyed_nodes,
    })
}
