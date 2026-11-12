//! Rebind semantic declarations and restore exact authored allocation records.
use super::super::{
    dependency::BridgeAuthoredDependencyDescription,
    fields::Reader,
    layout::Layout,
    registration::RegistrationView,
    rows::Rows,
    targets::{AllocationView, DeclarationView},
};
use super::{BridgeAuthoredInstallationBindings, BridgeAuthoredReadmissionDenial as D};
use crate::correspondence::{
    AllocationSlotIdentity, BridgeSemanticCorrespondenceRegistration,
    BridgeSignalAspectTargetDeclaration,
};
use std::collections::BTreeMap;
use worth_execution::ExecutionRequest;
use worth_foundational::facade::{BridgeSemanticClaimReference, DurableSignalNodeIdentity};
use worth_proof::TransitionOutcome;
use worth_signal::facade::durability::SignalAuthoredBranchInstallation;
use worth_signal::facade::{NodeId, PartitionToken, SignalGraph};

pub(super) fn registrations(
    layout: &Layout<'_>,
    branch: &SignalAuthoredBranchInstallation<'_>,
    bindings: &mut dyn BridgeAuthoredInstallationBindings,
    backing: &super::backing::Backing,
    execution: &ExecutionRequest<'_, '_>,
) -> Result<BTreeMap<BridgeSemanticClaimReference, BridgeSemanticCorrespondenceRegistration>, D> {
    let mut rows = Rows::new(layout.registrations)?;
    let bytes = crate::correspondence::authored_registration_storage_bytes(
        rows.len(),
        layout.registrations.len() / 8,
        layout.registrations.len(),
    )
    .ok_or(D::section(
        super::super::BridgeAuthoredSectionDenial::SizeLimit,
    ))?;
    backing.grow(execution, bytes)?;
    let mut keys = BTreeMap::new();
    let mut registrations = BTreeMap::new();
    while let Some(bytes) = rows.next()? {
        let mut input = Reader::new(bytes);
        let row = RegistrationView::read(&mut input)?;
        input.finish()?;
        let description = BridgeAuthoredDependencyDescription::decode(execution, row.meaning)?;
        let candidate =
            bindings.dependency(layout.branch, row.reference, &description, execution)?;
        backing.grow(
            execution,
            crate::correspondence::authored_binding_key_storage_bytes(
                &candidate,
                row.meaning.len(),
            )
            .ok_or(D::section(
                super::super::BridgeAuthoredSectionDenial::SizeLimit,
            ))?,
        )?;
        if !description.admits_fresh_candidate(execution, &candidate)? {
            return Err(D::SemanticBindingMismatch(row.reference));
        }
        let key = candidate.canonical_registration_key();
        if let Some(first) = keys.insert(key, row.reference) {
            return Err(D::AliasedBindings {
                first,
                second: row.reference,
            });
        }
        let mut targets = Rows::new(row.targets)?;
        let mut declarations = Vec::new();
        while let Some(bytes) = targets.next()? {
            let mut input = Reader::new(bytes);
            let target = DeclarationView::read(&mut input)?;
            input.finish()?;
            // Retired names remain in the authored section, without live capabilities.
            if let Some(declaration) = declaration(branch.graph(), &target, backing, execution)? {
                declarations.push(declaration);
            }
        }
        targets.finish()?;
        if !declarations.is_empty() {
            let mut registration =
                BridgeSemanticCorrespondenceRegistration::new(candidate, declarations)
                    .map_err(|_| D::ConditionalDefinitionMismatch)?;
            registration.recorded_claim = Some(row.reference);
            registrations.insert(row.reference, registration);
        }
    }
    rows.finish()?;
    Ok(registrations)
}
pub(super) fn node(identity: DurableSignalNodeIdentity) -> NodeId {
    NodeId::new(identity.index(), identity.generation())
}
pub(super) fn declaration(
    graph: &SignalGraph,
    target: &DeclarationView<'_>,
    backing: &super::backing::Backing,
    execution: &ExecutionRequest<'_, '_>,
) -> Result<Option<BridgeSignalAspectTargetDeclaration>, D> {
    if !graph.is_alive(node(target.node)) {
        return Ok(None);
    }
    let bytes = graph
        .installed_node_capability_backing_bytes(node(target.node), execution)
        .map_err(|cause| D::SignalNode(Box::new(cause)))?;
    backing.grow(execution, bytes)?;
    let TransitionOutcome::Success(capability) = graph.admit_installed_node(node(target.node))
    else {
        return Ok(None);
    };
    let mapping = crate::mapping::BridgeAspectRegistrationId::from_stable_name(target.mapping);
    let partition = PartitionToken(target.partition.to_owned());
    match target.slot {
        AllocationSlotIdentity::Allocate => Ok(Some(
            BridgeSignalAspectTargetDeclaration::allocate(mapping, partition, capability),
        )),
        AllocationSlotIdentity::Exact(aspect) => {
            let TransitionOutcome::Success(aspect) =
                graph.admit_installed_aspect(node(target.node), aspect)
            else {
                return Err(D::ConditionalDefinitionMismatch);
            };
            BridgeSignalAspectTargetDeclaration::exact(mapping, partition, capability, aspect)
                .map(Some)
                .map_err(|_| D::ConditionalDefinitionMismatch)
        }
    }
}
pub(super) fn allocations(
    registry: &mut crate::correspondence::CorrespondenceAllocationRegistry,
    layout: &Layout<'_>,
    graph: &SignalGraph,
    backing: &super::backing::Backing,
    execution: &ExecutionRequest<'_, '_>,
) -> Result<(), D> {
    let mut rows = Rows::new(layout.allocations)?;
    backing.grow(execution,crate::correspondence::CorrespondenceAllocationRegistry::authored_readmission_storage_bytes(
        rows.len(),layout.allocations.len()).ok_or(D::section(super::super::BridgeAuthoredSectionDenial::SizeLimit))?)?;
    while let Some(bytes) = rows.next()? {
        let mut input = Reader::new(bytes);
        let row = AllocationView::read(&mut input)?;
        input.finish()?;
        registry
            .readmit_authored_allocation(
                graph.installed_graph_capability().graph_instance_id(),
                PartitionToken(row.declaration.partition.to_owned()),
                node(row.declaration.node),
                row.selected,
                row.owner,
                crate::mapping::BridgeAspectRegistrationId::from_stable_name(
                    row.declaration.mapping,
                ),
                row.declaration.slot,
            )
            .map_err(D::Correspondence)?;
    }
    rows.finish()?;
    Ok(())
}
