mod cache;
pub(super) use cache::record_dependency_versions;
mod coordinates;
mod observation;
#[cfg(test)]
mod tests;

pub(crate) use observation::SignalConditionalVersionObservation;

use crate::data::aspect::{Aspect, AspectMask, MAX_ASPECTS};
use crate::data::comparator::ComparatorPolicyResolver;
use crate::data::error::SignalError;
use crate::data::graph::SignalGraph;
use crate::data::handle::NodeId;

use super::{InstalledSignalConditionalContract, SignalConditionalDecisionCounters};

#[derive(Clone)]
pub(super) struct SignalConditionalDependencyVersion {
    pub(super) node: NodeId,
    pub(super) aspect: Aspect,
    pub(super) scope: Option<crate::data::output::PartitionSubscription>,
    pub(super) version: u64,
}

// The ready recipe and delivered evidence retain the same immutable observation;
// creating the recipe must not copy every scoped dependency a second time.
pub(super) type SignalConditionalDependencyVersions =
    std::sync::Arc<Vec<SignalConditionalDependencyVersion>>;

pub(super) fn observed_dependency_versions(
    graph: &SignalGraph,
    contract: &InstalledSignalConditionalContract,
    work: &mut crate::data::retained_storage::RetainedStoragePreparation,
) -> Result<SignalConditionalDependencyVersions, SignalError> {
    let coordinates = coordinates::collect(
        contract.node(),
        contract.dependency_aspects(),
        graph.get_dep_snapshot(contract.node())?.entries(),
        work,
    )?;
    let versions = coordinates
        .into_iter()
        .map(|(node, aspect, scope)| {
            Ok(SignalConditionalDependencyVersion {
                node,
                aspect,
                version: graph.conditional_node_version_for_scope(
                    node,
                    aspect,
                    scope.as_ref(),
                    work,
                )?,
                scope,
            })
        })
        .collect::<Result<Vec<_>, SignalError>>()?;
    super::execution::work::reserve(work, Some(1))?;
    Ok(std::sync::Arc::new(versions))
}

pub(super) fn dependency_change_is_meaningful(
    graph: &SignalGraph,
    contract: &InstalledSignalConditionalContract,
    resolver: &mut impl ComparatorPolicyResolver,
    counters: &mut SignalConditionalDecisionCounters,
    work: &mut crate::data::retained_storage::RetainedStoragePreparation,
) -> Result<bool, SignalError> {
    if external_dependency_change_is_meaningful(graph, contract, resolver, counters, work)? {
        return Ok(true);
    }
    let snapshot = graph.get_dep_snapshot(contract.node())?;
    if snapshot.entries().is_empty() {
        return Ok(contract.dependency_aspects().is_empty());
    }
    for entry in snapshot.entries() {
        counters.dependency_version_checks += 1;
        let current = graph.conditional_node_version_for_scope(
            entry.source,
            entry.aspect,
            entry.scope.as_ref(),
            work,
        )?;
        super::execution::work::reserve(work, Some(1))?;
        counters.comparator_checks += 1;
        if contract.dependency_comparator().has_meaningful_change(
            entry.aspect,
            entry.cached_version,
            current,
            resolver,
        )? {
            return Ok(true);
        }
    }
    Ok(false)
}

fn external_dependency_change_is_meaningful(
    graph: &SignalGraph,
    contract: &InstalledSignalConditionalContract,
    resolver: &mut impl ComparatorPolicyResolver,
    counters: &mut SignalConditionalDecisionCounters,
    work: &mut crate::data::retained_storage::RetainedStoragePreparation,
) -> Result<bool, SignalError> {
    let mask = contract.dependency_aspects();
    if mask.is_empty() {
        return Ok(false);
    }
    let Some(cached) = cache::for_aspects(graph, contract.node(), mask, work)? else {
        counters.dependency_version_checks += dependency_aspects(mask).count();
        return Ok(true);
    };
    for aspect in dependency_aspects(mask) {
        counters.dependency_version_checks += 1;
        let current =
            graph.conditional_node_version_for_scope(contract.node(), aspect, None, work)?;
        super::execution::work::reserve(work, Some(1))?;
        counters.comparator_checks += 1;
        if contract.dependency_comparator().has_meaningful_change(
            aspect,
            cached.get(aspect),
            current,
            resolver,
        )? {
            return Ok(true);
        }
    }
    Ok(false)
}

fn dependency_aspects(mask: AspectMask) -> impl Iterator<Item = Aspect> {
    (0..MAX_ASPECTS)
        .filter_map(|index| Aspect::try_new(index as u8))
        .filter(move |aspect| mask.intersects((*aspect).into()))
}
