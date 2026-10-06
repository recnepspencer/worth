use crate::data::comparator::ComparatorPolicyResolver;
use crate::data::comparator::VersionComparatorPolicy;
use crate::data::dependency::DependencySnapshotId;
use crate::data::error::SignalError;
use crate::data::graph::SignalGraph;
use crate::data::handle::NodeId;
use crate::data::output::NodeEvaluationResult;
use crate::data::reuse::{ReuseBoundaryAuthority, ReuseBoundaryContext, ReuseBoundaryEvidence};
use crate::logic::evaluation::EvaluationWork;

pub(super) fn resolve_effect_reuse_boundary(
    graph: &SignalGraph,
    node: NodeId,
    comparator_resolver: &impl ComparatorPolicyResolver,
    result: Option<&NodeEvaluationResult>,
    keyed: Option<&crate::logic::prepared::PreparedKeyedContext>,
    strategy: Option<crate::data::reuse::ReuseStrategy>,
    previous: Option<&ReuseBoundaryAuthority>,
) -> Result<(ReuseBoundaryAuthority, Option<ReuseBoundaryContext>), SignalError> {
    if retains_reuse_boundary_detail(graph, strategy) {
        let detail = hydrate_reuse_topology_boundary_from_previous(
            graph,
            node,
            crate::logic::evaluation::resolve_reuse_boundary_context(
                graph,
                node,
                comparator_resolver,
                result,
                keyed,
            )?,
            previous,
        )?;
        let authority = detail.authority();
        return Ok((authority, Some(detail)));
    }

    let authority = hydrate_reuse_topology_boundary_authority_from_previous(
        graph,
        node,
        crate::logic::evaluation::resolve_reuse_boundary_authority(
            graph,
            node,
            comparator_resolver,
            result,
            keyed,
        )?,
        previous,
    )?;
    Ok((authority, None))
}

pub(super) fn resolve_effect_reuse_boundary_with_policy(
    graph: &SignalGraph,
    node: NodeId,
    comparator_policy: VersionComparatorPolicy,
    result: Option<&NodeEvaluationResult>,
    keyed: Option<&crate::logic::prepared::PreparedKeyedContext>,
    strategy: Option<crate::data::reuse::ReuseStrategy>,
    previous: Option<&ReuseBoundaryAuthority>,
    work: &mut EvaluationWork<'_, '_>,
) -> Result<(ReuseBoundaryAuthority, Option<ReuseBoundaryContext>), SignalError> {
    if retains_reuse_boundary_detail(graph, strategy) {
        let detail = hydrate_reuse_topology_boundary_from_previous(
            graph,
            node,
            crate::logic::evaluation::resolve_reuse_boundary_context_with_policy_observed(
                graph,
                node,
                comparator_policy,
                result,
                keyed,
                work,
            )?,
            previous,
        )?;
        let authority = detail.authority();
        return Ok((authority, Some(detail)));
    }

    if keyed
        .and_then(|prepared| prepared.persistent_correspondence.as_ref())
        .is_none()
        && keyed.is_none_or(|prepared| prepared.composition_regions.is_empty())
    {
        if let Some(output) = result {
            crate::data::proof::PartitionScopeSet::admit_regions_copy_work(
                output.changed_regions.as_slice(),
                1,
                1,
                work,
            )?;
        }
    }
    let authority = hydrate_reuse_topology_boundary_authority_from_previous(
        graph,
        node,
        crate::logic::evaluation::resolve_reuse_boundary_authority_with_policy(
            graph,
            node,
            comparator_policy,
            result,
            keyed,
        )?,
        previous,
    )?;
    Ok((authority, None))
}

pub(super) fn hydrate_reuse_boundary_evidence(
    mut evidence: ReuseBoundaryEvidence,
) -> ReuseBoundaryEvidence {
    if let Some(previous) = evidence.previous.as_mut() {
        if previous.structural_dependency_basis == DependencySnapshotId::EMPTY
            && evidence.current.structural_dependency_basis != DependencySnapshotId::EMPTY
        {
            previous.structural_dependency_basis = evidence.current.structural_dependency_basis;
        }
        if evidence.current.structural_dependency_basis == DependencySnapshotId::EMPTY
            && previous.structural_dependency_basis != DependencySnapshotId::EMPTY
        {
            evidence.current.structural_dependency_basis = previous.structural_dependency_basis;
        }
    }
    evidence
}

fn hydrate_reuse_topology_boundary_from_previous(
    graph: &SignalGraph,
    node: NodeId,
    mut current: ReuseBoundaryContext,
    previous: Option<&ReuseBoundaryAuthority>,
) -> Result<ReuseBoundaryContext, SignalError> {
    let Some(previous) = previous else {
        return Ok(current);
    };
    if current.topology_regime != 0
        || !graph.dependencies_of(node)?.is_empty()
        || previous.topology_regime == 0
    {
        return Ok(current);
    }
    current.topology_regime = previous.topology_regime;
    Ok(current)
}

fn hydrate_reuse_topology_boundary_authority_from_previous(
    graph: &SignalGraph,
    node: NodeId,
    mut current: ReuseBoundaryAuthority,
    previous: Option<&ReuseBoundaryAuthority>,
) -> Result<ReuseBoundaryAuthority, SignalError> {
    let Some(previous) = previous else {
        return Ok(current);
    };
    if current.topology_regime != 0
        || !graph.dependencies_of(node)?.is_empty()
        || previous.topology_regime == 0
    {
        return Ok(current);
    }
    current.topology_regime = previous.topology_regime;
    Ok(current)
}

pub(super) fn retains_reuse_boundary_detail(
    graph: &SignalGraph,
    strategy: Option<crate::data::reuse::ReuseStrategy>,
) -> bool {
    let retention = graph.installed_runtime_policy().retention_budget();
    let cold_retention_active = matches!(
        retention.explanation_retention,
        crate::diagnostics::policy::ArtifactRetentionPolicy::Retain
    ) || matches!(
        retention.provenance_retention,
        crate::diagnostics::policy::ArtifactRetentionPolicy::Retain
    );
    cold_retention_active
        && matches!(
            strategy,
            Some(crate::data::reuse::ReuseStrategy::CrossIdentityPersistentMatch)
                | Some(crate::data::reuse::ReuseStrategy::PartialArtifactSplicing)
        )
}

pub(super) fn format_reuse_boundary_evidence(evidence: &ReuseBoundaryEvidence) -> String {
    let previous = evidence
        .previous
        .as_ref()
        .map(|context| {
            format!(
                "prev[topology={}, structural={:?}, family={:?}, partition_regions={}]",
                context.topology_regime,
                context.structural_dependency_basis,
                context.artifact_family,
                context.partition_region_basis_count
            )
        })
        .unwrap_or_else(|| "prev[none]".to_string());
    let current = format!(
        "curr[topology={}, structural={:?}, family={:?}, partition_regions={}]",
        evidence.current.topology_regime,
        evidence.current.structural_dependency_basis,
        evidence.current.artifact_family,
        evidence.current.partition_region_basis_count
    );
    format!("{previous}; {current}")
}
