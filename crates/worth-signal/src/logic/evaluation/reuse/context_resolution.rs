use crate::data::comparator::ComparatorPolicyResolver;
use crate::data::core_profile::StableHashValue;
use crate::data::dependency::{DependencyEdge, DependencySnapshotEntry, DependencySnapshotId};
use crate::data::error::SignalError;
use crate::data::graph::SignalGraph;
use crate::data::handle::NodeId;
use crate::data::output::NodeEvaluationResult;
use crate::data::proof::PartitionScopeSet;
use crate::data::reuse::{
    stable_partition_scope_digest_from_slice, stable_persistent_correspondence_digest,
    stable_semantic_region_digest_from_parts, ArtifactFamilyId, ReuseBoundaryAuthority,
    ReuseBoundaryContext, ReuseSemanticRegionIdentity, ReuseStrategyBoundaryAuthority,
    ReuseStrategyBoundaryContext,
};
use crate::logic::evaluation::EvaluationWork;
use crate::logic::prepared::PreparedKeyedContext;

pub(crate) fn resolve_reuse_boundary_context(
    graph: &SignalGraph,
    node: NodeId,
    comparator_resolver: &impl ComparatorPolicyResolver,
    result: Option<&NodeEvaluationResult>,
    keyed: Option<&PreparedKeyedContext>,
) -> Result<ReuseBoundaryContext, SignalError> {
    resolve_reuse_boundary_context_with_policy(
        graph,
        node,
        comparator_resolver
            .policy_for_node(node, graph.node_eval_config(node)?.comparator.as_ref()),
        result,
        keyed,
    )
}

pub(crate) fn resolve_reuse_boundary_authority(
    graph: &SignalGraph,
    node: NodeId,
    comparator_resolver: &impl ComparatorPolicyResolver,
    result: Option<&NodeEvaluationResult>,
    keyed: Option<&PreparedKeyedContext>,
) -> Result<ReuseBoundaryAuthority, SignalError> {
    resolve_reuse_boundary_authority_with_policy(
        graph,
        node,
        comparator_resolver
            .policy_for_node(node, graph.node_eval_config(node)?.comparator.as_ref()),
        result,
        keyed,
    )
}

pub(crate) fn resolve_reuse_boundary_authority_with_policy(
    graph: &SignalGraph,
    node: NodeId,
    comparator_policy: crate::data::comparator::VersionComparatorPolicy,
    result: Option<&NodeEvaluationResult>,
    keyed: Option<&PreparedKeyedContext>,
) -> Result<ReuseBoundaryAuthority, SignalError> {
    let eval = graph.node_eval_config(node)?;
    let contract = &eval.contract;
    let partition_scope = contract.semantics.partition_scope.as_deref().unwrap_or(&[]);
    let strategy_detail = keyed
        .and_then(|prepared| {
            prepared
                .persistent_correspondence
                .as_ref()
                .map(
                    |persistent_correspondence| ReuseStrategyBoundaryAuthority::CrossIdentity {
                        persistent_correspondence_kind: persistent_correspondence.kind(),
                        persistent_correspondence_digest: stable_persistent_correspondence_digest(
                            persistent_correspondence,
                        ),
                        persistent_correspondence_valid: persistent_correspondence
                            .is_structurally_valid(),
                    },
                )
        })
        .or_else(|| {
            keyed.and_then(|prepared| {
                (!prepared.composition_regions.is_empty()).then_some(
                    ReuseStrategyBoundaryAuthority::PartialArtifactSplice {
                        composition_region_digest: stable_partition_scope_digest_from_slice(
                            prepared.composition_regions.as_slice(),
                        ),
                        composition_region_count: prepared.composition_regions.len() as u32,
                    },
                )
            })
        })
        .or_else(|| {
            result
                .map(|output| PartitionScopeSet::from_regions(output.changed_regions.as_slice()))
                .filter(|regions| !regions.is_empty())
                .map(
                    |regions| ReuseStrategyBoundaryAuthority::PartialArtifactSplice {
                        composition_region_digest: stable_partition_scope_digest_from_slice(
                            regions.as_slice(),
                        ),
                        composition_region_count: regions.len() as u32,
                    },
                )
        })
        .unwrap_or_default();
    let dependencies = graph.dependencies_of(node)?;
    let dependency_snapshot = graph.get_dep_snapshot(node)?;

    Ok(ReuseBoundaryAuthority {
        topology_regime: stable_topology_regime(dependencies),
        tolerance_regime: comparator_policy,
        semantic_region_digest: stable_semantic_region_digest_from_parts(
            node,
            eval.partitioned_output,
            partition_scope,
            contract.semantics.required_context,
        ),
        authority_policy: contract.authority.policy,
        artifact_family: keyed
            .and_then(|prepared| prepared.family.as_ref())
            .map(|family| ArtifactFamilyId::new(family.as_str())),
        structural_dependency_basis: stable_dependency_snapshot_basis(
            dependency_snapshot.entries(),
        ),
        partition_region_basis_digest: stable_partition_scope_digest_from_slice(partition_scope),
        partition_region_basis_count: partition_scope.len() as u32,
        strategy_detail,
    })
}

pub(crate) fn resolve_reuse_boundary_context_with_policy(
    graph: &SignalGraph,
    node: NodeId,
    comparator_policy: crate::data::comparator::VersionComparatorPolicy,
    result: Option<&NodeEvaluationResult>,
    keyed: Option<&PreparedKeyedContext>,
) -> Result<ReuseBoundaryContext, SignalError> {
    let eval = graph.node_eval_config(node)?;
    let contract = &eval.contract;
    let partition_region_basis =
        PartitionScopeSet::from(contract.semantics.partition_scope.as_deref().unwrap_or(&[]));
    let composition_regions = keyed
        .and_then(|prepared| {
            (!prepared.composition_regions.is_empty())
                .then_some(prepared.composition_regions.clone())
        })
        .or_else(|| {
            result
                .map(|output| PartitionScopeSet::from_regions(output.changed_regions.as_slice()))
                .filter(|regions| !regions.is_empty())
        });
    let strategy_detail = keyed
        .and_then(|prepared| {
            prepared
                .persistent_correspondence
                .clone()
                .map(
                    |persistent_correspondence| ReuseStrategyBoundaryContext::CrossIdentity {
                        persistent_correspondence,
                    },
                )
        })
        .or_else(|| {
            composition_regions.clone().map(|composition_regions| {
                ReuseStrategyBoundaryContext::PartialArtifactSplice {
                    composition_regions,
                }
            })
        })
        .unwrap_or_default();
    let dependencies = graph.dependencies_of(node)?;
    let dependency_snapshot = graph.get_dep_snapshot(node)?;

    Ok(ReuseBoundaryContext {
        topology_regime: stable_topology_regime(dependencies),
        tolerance_regime: comparator_policy,
        semantic_region: ReuseSemanticRegionIdentity::new(
            node,
            eval.partitioned_output,
            contract
                .semantics
                .partition_scope
                .clone()
                .unwrap_or_default(),
            contract.semantics.required_context,
        ),
        authority_policy: contract.authority.policy,
        artifact_family: keyed
            .and_then(|prepared| prepared.family.as_ref())
            .map(|family| ArtifactFamilyId::new(family.as_str())),
        structural_dependency_basis: stable_dependency_snapshot_basis(
            dependency_snapshot.entries(),
        ),
        partition_region_basis: partition_region_basis.clone(),
        strategy_detail,
    })
}

pub(crate) fn resolve_reuse_boundary_context_with_policy_observed(
    graph: &SignalGraph,
    node: NodeId,
    comparator_policy: crate::data::comparator::VersionComparatorPolicy,
    result: Option<&NodeEvaluationResult>,
    keyed: Option<&PreparedKeyedContext>,
    work: &mut EvaluationWork<'_, '_>,
) -> Result<ReuseBoundaryContext, SignalError> {
    let partition_scope = graph
        .node_eval_config(node)?
        .contract
        .semantics
        .partition_scope
        .as_deref()
        .unwrap_or(&[]);
    // The partition basis is built, copied into the context and copied into
    // semantic-region identity. Composition is built and may be copied into
    // the selected strategy detail.
    PartitionScopeSet::admit_scopes_copy_work(partition_scope, 3, 2, work)?;
    let detail_clones_composition = keyed
        .and_then(|prepared| prepared.persistent_correspondence.as_ref())
        .is_none();
    if let Some(scopes) = keyed
        .map(|prepared| prepared.composition_regions.as_slice())
        .filter(|scopes| !scopes.is_empty())
    {
        PartitionScopeSet::admit_scopes_copy_work(
            scopes,
            usize::from(detail_clones_composition) + 1,
            0,
            work,
        )?;
    } else if let Some(result) = result {
        PartitionScopeSet::admit_regions_copy_work(
            result.changed_regions.as_slice(),
            usize::from(detail_clones_composition) + 1,
            1,
            work,
        )?;
    }
    resolve_reuse_boundary_context_with_policy(graph, node, comparator_policy, result, keyed)
}

fn stable_topology_regime(edges: &[DependencyEdge]) -> u32 {
    if edges.is_empty() {
        return 0;
    }
    stable_hash_to_u32(edges.iter().fold(topology_seed(), |hash, edge| {
        let hash = hash_u64(hash, edge.aspect().index() as u64);
        match edge.scope_ref() {
            Some(scope) => hash_string(hash_u64(hash, 1), &format!("{scope:?}")),
            None => hash_u64(hash, 0),
        }
    }))
}

fn stable_dependency_snapshot_basis(entries: &[DependencySnapshotEntry]) -> DependencySnapshotId {
    if entries.is_empty() {
        return DependencySnapshotId::EMPTY;
    }
    let fingerprint = stable_hash_to_u32(entries.iter().fold(snapshot_seed(), |hash, entry| {
        let hash = hash_node_id(hash, entry.source);
        let hash = hash_u64(hash, entry.aspect.index() as u64);
        match &entry.scope {
            Some(scope) => hash_string(hash_u64(hash, 1), &format!("{scope:?}")),
            None => hash_u64(hash, 0),
        }
    }));
    DependencySnapshotId::from_semantic_fingerprint(fingerprint)
}

fn topology_seed() -> StableHashValue {
    0x8422_2325_cbf2_9ce4_u64 as StableHashValue
}

fn snapshot_seed() -> StableHashValue {
    0x6c62_272e_07bb_0142_u64 as StableHashValue
}

fn hash_node_id(hash: StableHashValue, node: NodeId) -> StableHashValue {
    hash_u64(
        hash_u64(hash, node.index() as u64),
        node.generation() as u64,
    )
}

fn hash_string(mut hash: StableHashValue, value: &str) -> StableHashValue {
    for byte in value.as_bytes() {
        hash ^= *byte as StableHashValue;
        hash = hash.wrapping_mul(0x100000001b3_u64 as StableHashValue);
    }
    hash
}

fn hash_u64(mut hash: StableHashValue, value: u64) -> StableHashValue {
    for byte in value.to_le_bytes() {
        hash ^= byte as StableHashValue;
        hash = hash.wrapping_mul(0x100000001b3_u64 as StableHashValue);
    }
    hash
}

fn stable_hash_to_u32(hash: StableHashValue) -> u32 {
    let folded = (hash ^ (hash >> 32)) as u32;
    if folded == 0 {
        1
    } else {
        folded
    }
}
