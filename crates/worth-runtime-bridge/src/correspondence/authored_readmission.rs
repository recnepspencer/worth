//! Re-admit recorded performed targets through fresh mapping and graph proofs.
use super::{
    BridgeCorrespondenceDenialKind as D, BridgeInstalledSemanticCorrespondence,
    BridgeSemanticCorrespondenceRegistration, InstalledCorrespondenceTarget,
    ProvenCorrespondenceTargets,
};
use crate::facade::RuntimeBridge;
use worth_proof::TransitionOutcome;
use worth_signal::facade::SignalGraph;

pub(crate) fn readmit_performed_correspondence(
    bridge: &RuntimeBridge,
    registration: &BridgeSemanticCorrespondenceRegistration,
    graph: &SignalGraph,
    mut expected: Vec<InstalledCorrespondenceTarget>,
    allocations: &super::CorrespondenceAllocationRegistry,
) -> Result<BridgeInstalledSemanticCorrespondence, D> {
    let resolved = super::resolution::resolve_registration(bridge, registration, graph)
        .map_err(|_| D::InvalidPortableDependency)?;
    let mapped = super::target_mapping::map_targets(bridge, resolved)
        .map_err(|_| D::MappingSemanticMismatch)?;
    if mapped.targets.len() != expected.len() {
        return Err(D::InvalidPortableDependency);
    }
    for actual in &mapped.targets {
        let target = expected
            .iter_mut()
            .find(|target| {
                target.node == actual.declaration.node
                    && target.partition == actual.declaration.partition
                    && target.mapping_identity == actual.mapping_identity
            })
            .ok_or(D::MappingSemanticMismatch)?;
        let super::BridgeSignalSlotRequest::Exact(aspect) = &actual.declaration.slot else {
            return Err(D::InvalidPortableDependency);
        };
        if target.aspect != aspect.aspect()
            || !super::slot_allocation::signal_node_admits(&actual.declaration, target.aspect)
            || target.precision != actual.precision
            || target.admitted_source_widening != actual.admitted_source_widening
            || target.signal_graph_instance_id != mapped.resolved.signal_graph.graph_instance_id()
            || target.allocation_sources.is_empty()
        {
            return Err(D::InvalidPortableDependency);
        }
        target.allocation_sources.sort();
        if target
            .allocation_sources
            .windows(2)
            .any(|pair| pair[0] == pair[1])
        {
            return Err(D::InvalidPortableDependency);
        }
        for source in &target.allocation_sources {
            if !allocations.authored_allocations().any(|record| {
                record.node == target.node
                    && record.partition == &target.partition
                    && record.aspect == target.aspect
                    && record.owner == source
                    && record.mapping == &actual.declaration.aspect_registration_id
            }) {
                return Err(D::InvalidPortableDependency);
            }
        }
    }
    let targets = ProvenCorrespondenceTargets::admit(expected)?;
    let TransitionOutcome::Success(capability) = graph.admit_installed_aspects(
        targets
            .as_slice()
            .iter()
            .map(|target| (target.node, target.aspect)),
    ) else {
        return Err(D::MissingOrStaleSignalNode);
    };
    Ok(BridgeInstalledSemanticCorrespondence::admit_ready(
        mapped.resolved.recipe,
        targets,
        mapped.resolved.counters,
        &mapped.resolved.signal_graph,
        &capability,
    ))
}
