use crate::evidence_identity::WorthQueryEvidenceIdentityDerivationDenial as IdentityDerivationDenial;
use std::any::TypeId;
use std::collections::{BTreeMap, HashMap};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;

use crate::authoring::{
    WorthQueryGraphReadDomainOperationDeclaration, WorthQueryGraphReadOperationKey,
};
use crate::evidence_identity::WorthQueryEvidenceIdentity;
use crate::runtime::{
    WorthQueryGraphReadOperationLookup, WorthQueryGraphReadOperationRegistration,
    WorthQueryGraphReadOperationUnsupportedShapeDeclaration,
};
use worth_query_installation::facade::{
    WorthQueryInstalledDomainOperationAuthority, WorthQueryInstalledOperationAuthority,
    WorthQueryInstalledPackageIndex,
};

use super::installation_state::WorthQueryInstalledDomainArtifact;

mod domain_operations;
mod graph_participations;
mod identity;
mod identity_inputs;
mod operation_descriptors;
mod required_domains;
mod semantic_keys;
mod semantics;
mod shape;

use domain_operations::{
    domain_operation_identity_parts, domain_operation_index, InstalledDomainOperation,
};
use graph_participations::operation_graph_participation_index;
pub(crate) use graph_participations::WorthQueryInstalledOperationGraphBinding;
use identity::execution_index_identity;
use identity_inputs::package_provenance_identity_parts;
pub(crate) use operation_descriptors::{
    WorthQueryDomainOperationExecutionDescriptor, WorthQueryWorkflowExecutionDescriptor,
};
use required_domains::operation_required_domain_index;
pub(crate) use required_domains::WorthQueryInstalledOperationRequiredDomain;
use semantic_keys::{InstalledDeclarationFamilySlot, InstalledDomainOwner, InstalledInvariantSlot};
pub(crate) use semantics::WorthQueryInstalledDomainSemantics;
pub(crate) use shape::WorthQueryInstalledDomainExecutionIndexShape;
use shape::{execution_index_shape, WorthQueryExecutionIndexShapeInputs};

pub(crate) struct WorthQueryInstalledDomainExecutionIndex {
    runtime_authority: crate::runtime::WorthQueryRuntimeAuthorityIdentity,
    graph_read_operations: BTreeMap<WorthQueryGraphReadOperationKey, InstalledGraphReadOperation>,
    domain_operations: HashMap<(TypeId, TypeId, TypeId), InstalledDomainOperation>,
    operation_graph_participations:
        HashMap<(TypeId, TypeId, TypeId), Vec<WorthQueryInstalledOperationGraphBinding>>,
    operation_required_domains:
        HashMap<(TypeId, TypeId, TypeId), Vec<WorthQueryInstalledOperationRequiredDomain>>,
    semantics_by_marker: HashMap<TypeId, Arc<WorthQueryInstalledDomainSemantics>>,
    identity: WorthQueryEvidenceIdentity,
    shape: WorthQueryInstalledDomainExecutionIndexShape,
    indexed_operation_lookups: AtomicUsize,
}

pub(crate) struct WorthQueryResolvedInstalledDomainOperation {
    pub(crate) authority: Arc<WorthQueryInstalledDomainOperationAuthority>,
    pub(crate) workflow_graph:
        Option<Arc<crate::domain_installation::WorthQueryInstalledWorkflowGraph>>,
}

pub(super) struct InstalledGraphReadOperation {
    registration: WorthQueryGraphReadOperationRegistration,
    _installation_authority: WorthQueryInstalledOperationAuthority,
}

impl WorthQueryInstalledDomainExecutionIndex {
    pub(crate) fn build(
        artifacts: &[WorthQueryInstalledDomainArtifact],
        runtime_authority: crate::runtime::WorthQueryRuntimeAuthorityIdentity,
        portable_index: &WorthQueryInstalledPackageIndex,
    ) -> Result<Self, IdentityDerivationDenial> {
        let graph_read_operations = graph_read_operation_index(artifacts, portable_index);
        let domain_operations = domain_operation_index(artifacts, portable_index);
        let operation_graph_participations = operation_graph_participation_index(artifacts);
        let operation_required_domains = operation_required_domain_index(artifacts);
        let domain_operation_identity_parts = domain_operation_identity_parts(&domain_operations);
        let semantics_by_marker = installed_domain_semantics_index(artifacts)?;
        let declaration_families = declaration_family_index(artifacts);
        let contribution_policies = contribution_policy_index(artifacts);
        let invariant_slots = invariant_slot_index(artifacts);
        let package_provenance = package_provenance_identity_parts(artifacts);
        let identity = execution_index_identity(
            &package_provenance,
            &graph_read_operations,
            &domain_operation_identity_parts,
            &declaration_families,
            &contribution_policies,
            &invariant_slots,
        )?;
        let shape = execution_index_shape(WorthQueryExecutionIndexShapeInputs {
            graph_read_operations: &graph_read_operations,
            domain_operations: &domain_operations,
            operation_graph_participations: &operation_graph_participations,
            operation_required_domains: &operation_required_domains,
            declaration_families: &declaration_families,
            contribution_policies: &contribution_policies,
            invariant_slots: &invariant_slots,
        });
        Ok(Self {
            runtime_authority,
            graph_read_operations,
            domain_operations,
            operation_graph_participations,
            operation_required_domains,
            semantics_by_marker,
            identity,
            shape,
            indexed_operation_lookups: AtomicUsize::new(0),
        })
    }

    pub(crate) fn domain_semantics(
        &self,
        marker_type: TypeId,
    ) -> Option<Arc<WorthQueryInstalledDomainSemantics>> {
        self.semantics_by_marker.get(&marker_type).map(Arc::clone)
    }

    pub(crate) fn identity(&self) -> &WorthQueryEvidenceIdentity {
        &self.identity
    }

    pub(crate) fn shape(&self) -> WorthQueryInstalledDomainExecutionIndexShape {
        self.shape
    }

    pub(crate) fn indexed_operation_lookups(&self) -> usize {
        self.indexed_operation_lookups.load(Ordering::Relaxed)
    }

    pub(crate) fn domain_operation_authority(
        &self,
        domain_marker: TypeId,
        operation_marker: TypeId,
        family_marker: TypeId,
    ) -> Option<WorthQueryResolvedInstalledDomainOperation> {
        self.indexed_operation_lookups
            .fetch_add(1, Ordering::Relaxed);
        self.domain_operations
            .get(&(domain_marker, operation_marker, family_marker))
            .map(|operation| WorthQueryResolvedInstalledDomainOperation {
                authority: Arc::clone(&operation.authority),
                workflow_graph: operation.workflow_graph.as_ref().map(Arc::clone),
            })
    }

    pub(crate) fn domain_operation_graph_bindings(
        &self,
        domain_marker: TypeId,
        operation_marker: TypeId,
        family_marker: TypeId,
    ) -> &[WorthQueryInstalledOperationGraphBinding] {
        self.operation_graph_participations
            .get(&(domain_marker, operation_marker, family_marker))
            .map(Vec::as_slice)
            .unwrap_or(&[])
    }

    pub(crate) fn domain_operation_execution_descriptors(
        &self,
    ) -> Vec<WorthQueryDomainOperationExecutionDescriptor> {
        operation_descriptors::operation_execution_descriptors(&self.domain_operations)
    }

    pub(crate) fn domain_operation_required_domains(
        &self,
        domain_marker: TypeId,
        operation_marker: TypeId,
        family_marker: TypeId,
    ) -> &[WorthQueryInstalledOperationRequiredDomain] {
        self.operation_required_domains
            .get(&(domain_marker, operation_marker, family_marker))
            .map(Vec::as_slice)
            .unwrap_or(&[])
    }

    pub(crate) fn workflow_operation_execution_descriptors(
        &self,
    ) -> Vec<WorthQueryWorkflowExecutionDescriptor> {
        operation_descriptors::workflow_execution_descriptors(&self.domain_operations)
    }
}

fn graph_read_operation_index(
    artifacts: &[WorthQueryInstalledDomainArtifact],
    portable_index: &WorthQueryInstalledPackageIndex,
) -> BTreeMap<WorthQueryGraphReadOperationKey, InstalledGraphReadOperation> {
    artifacts
        .iter()
        .flat_map(|artifact| {
            artifact
                .graph_read_operations
                .iter()
                .map(move |operation| (artifact, operation))
        })
        .map(|(artifact, operation)| {
            let key = operation
                .operation_key()
                .expect("admitted package operations have canonical keys");
            let operation_slot = format!("{}:{}", key.name().as_str(), key.version().value());
            let installation_authority = portable_index
                .operation(&artifact.domain_owner, &operation_slot)
                .expect("every lowered operation must have portable installation authority");
            portable_index
                .validate_operation(&installation_authority)
                .expect("newly minted operation authority must validate in its installed index");
            (
                key,
                InstalledGraphReadOperation {
                    registration: operation.clone(),
                    _installation_authority: installation_authority,
                },
            )
        })
        .collect()
}

fn installed_domain_semantics_index(
    artifacts: &[WorthQueryInstalledDomainArtifact],
) -> Result<HashMap<TypeId, Arc<WorthQueryInstalledDomainSemantics>>, IdentityDerivationDenial> {
    artifacts
        .iter()
        .map(|artifact| {
            Ok((
                artifact.marker_type,
                Arc::new(WorthQueryInstalledDomainSemantics::from_artifact(artifact)?),
            ))
        })
        .collect()
}

fn declaration_family_index(
    artifacts: &[WorthQueryInstalledDomainArtifact],
) -> BTreeMap<InstalledDeclarationFamilySlot, String> {
    artifacts
        .iter()
        .flat_map(|artifact| {
            artifact.declaration_families.iter().map(|family| {
                (
                    InstalledDeclarationFamilySlot::new(
                        &artifact.domain_owner,
                        family.family_key(),
                    ),
                    family.canonical_part(),
                )
            })
        })
        .collect()
}

fn contribution_policy_index(
    artifacts: &[WorthQueryInstalledDomainArtifact],
) -> BTreeMap<InstalledDomainOwner, Vec<String>> {
    artifacts
        .iter()
        .map(|artifact| {
            (
                InstalledDomainOwner::new(&artifact.domain_owner),
                artifact
                    .contribution_policy
                    .iter()
                    .map(|category| category.as_str().to_string())
                    .collect(),
            )
        })
        .collect()
}

fn invariant_slot_index(
    artifacts: &[WorthQueryInstalledDomainArtifact],
) -> BTreeMap<InstalledInvariantSlot, String> {
    artifacts
        .iter()
        .flat_map(|artifact| {
            artifact.invariant_definitions.iter().map(|invariant| {
                (
                    InstalledInvariantSlot::new(&artifact.domain_owner, invariant.slot_key()),
                    invariant.canonical_part(),
                )
            })
        })
        .collect()
}

impl WorthQueryGraphReadOperationLookup for WorthQueryInstalledDomainExecutionIndex {
    fn matching_declared_operation(
        &self,
        declaration: &WorthQueryGraphReadDomainOperationDeclaration,
        installed_authority: Option<&super::WorthQueryInstalledDomainAuthorityWitness>,
    ) -> Option<&WorthQueryGraphReadOperationRegistration> {
        self.indexed_operation_lookups
            .fetch_add(1, Ordering::Relaxed);
        let installed_authority = installed_authority?;
        self.graph_read_operations
            .get(declaration.key())
            .filter(|operation| {
                let authority = installed_authority.authority();
                let provenance = operation.registration.installed_provenance();
                authority.runtime_authority() == self.runtime_authority
                    && authority.is_current_installation_generation()
                    && provenance.is_some_and(|provenance| {
                        provenance.domain_owner() == authority.domain_owner()
                            && provenance.package_identity()
                                == authority.package_identity().as_str()
                    })
            })
            .map(|operation| &operation.registration)
            .filter(|registration| registration.matches_declared_operation(declaration))
    }

    fn matching_unsupported_declared_operation(
        &self,
        _declaration: &WorthQueryGraphReadDomainOperationDeclaration,
        _installed_authority: Option<&super::WorthQueryInstalledDomainAuthorityWitness>,
    ) -> Option<&WorthQueryGraphReadOperationUnsupportedShapeDeclaration> {
        None
    }
}
