mod assembly;
mod causes;
mod lineage;
mod policy;
mod rendering;

use crate::data::comparator::{
    ComparatorPolicyResolver, DefaultComparatorPolicyResolver, DefaultComparatorResolver,
    VersionComparatorPolicy,
};
use crate::data::error::SignalError;
use crate::data::graph::SignalGraph;
use crate::data::handle::NodeId;

use super::types::NodeExplanation;

pub fn explain_with_policy_resolver(
    graph: &SignalGraph,
    node: NodeId,
    comparator_resolver: &impl ComparatorPolicyResolver,
) -> Result<NodeExplanation, SignalError> {
    explain_with_policy_resolver_mode(graph, node, comparator_resolver, true)
}

pub(crate) fn explain_reconstructing_with_policy_resolver(
    graph: &SignalGraph,
    node: NodeId,
    comparator_resolver: &impl ComparatorPolicyResolver,
) -> Result<NodeExplanation, SignalError> {
    explain_with_policy_resolver_mode(graph, node, comparator_resolver, false)
}

fn explain_with_policy_resolver_mode(
    graph: &SignalGraph,
    node: NodeId,
    comparator_resolver: &impl ComparatorPolicyResolver,
    allow_retained_fast_path: bool,
) -> Result<NodeExplanation, SignalError> {
    let diagnostic_policy = if allow_retained_fast_path {
        policy::ExplanationDiagnosticPolicy::retained_or_reconstruct()
    } else {
        policy::ExplanationDiagnosticPolicy::reconstruct_only()
    };
    if diagnostic_policy.retained_fast_path_allowed()
        && let Some(explanation) = policy::retained_explanation(graph, node)?
    {
        return Ok(explanation);
    }

    let resolution = assembly::assemble(graph, node, comparator_resolver)?;
    resolution.traversal_cost.validate();
    Ok(resolution.explanation)
}

pub fn explain(graph: &SignalGraph, node: NodeId) -> Result<NodeExplanation, SignalError> {
    let resolver = DefaultComparatorPolicyResolver {
        fallback: VersionComparatorPolicy::Exact,
        custom: DefaultComparatorResolver,
    };
    explain_with_policy_resolver(graph, node, &resolver)
}
