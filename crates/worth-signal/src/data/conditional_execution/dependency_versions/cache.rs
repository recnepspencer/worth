//! Work-admitted access to conditional dependency version observations.
use super::{
    dependency_aspects, InstalledSignalConditionalContract, SignalConditionalVersionObservation,
};
use crate::data::aspect::{AspectMask, AspectVersion, MAX_ASPECTS};
use crate::data::conditional_execution::execution::work::reserve;
use crate::data::error::SignalError;
use crate::data::graph::SignalGraph;
use crate::data::handle::NodeId;
use crate::data::retained_storage::RetainedStoragePreparation;

pub(super) fn for_aspects(
    graph: &SignalGraph,
    node: NodeId,
    aspects: AspectMask,
    work: &mut RetainedStoragePreparation,
) -> Result<Option<AspectVersion>, SignalError> {
    reserve(
        work,
        graph
            .conditional_dependency_versions
            .lookup_steps()
            .checked_add(MAX_ASPECTS),
    )?;
    Ok(graph
        .conditional_dependency_versions
        .get(&node)
        .and_then(|observation| observation.for_aspects(aspects)))
}

pub(in crate::data::conditional_execution) fn record_dependency_versions(
    graph: &mut SignalGraph,
    contract: &InstalledSignalConditionalContract,
    work: &mut RetainedStoragePreparation,
) -> Result<(), SignalError> {
    reserve(work, Some(MAX_ASPECTS))?;
    let aspects = contract.dependency_aspects();
    let mut versions = AspectVersion::zero();
    for aspect in dependency_aspects(aspects) {
        versions = versions.with(
            aspect,
            graph.conditional_node_version_for_scope(contract.node(), aspect, None, work)?,
        );
    }
    // Concrete NodeId keys and inline AspectVersion observations have no
    // variable payload clone or comparator. Include query/base paths, interval
    // readmission (remove plus up to two inserts), and overlay path copying.
    reserve(
        work,
        graph
            .conditional_dependency_versions
            .lookup_steps()
            .checked_mul(16)
            .and_then(|n| n.checked_add(MAX_ASPECTS)),
    )?;
    graph.record_conditional_dependency_versions(
        contract.node(),
        SignalConditionalVersionObservation::new(aspects, versions),
        work,
    )
}
