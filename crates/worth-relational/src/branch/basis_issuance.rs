//! Issue and describe one exact retained observation from the Branch owner.
use std::sync::Arc;

use super::{
    authority::issue_relational_branch_observation_authority,
    basis::{
        AdmittedRelationalBranchBasis, AdmittedRelationalBranchBasisInner,
        RelationalBranchBasisDescriptor, RelationalBranchBasisPosture,
    },
    basis_admission_identity::RelationalBranchBasisAdmissionIdentity,
    basis_axis_validation::require_root_matches_reference,
    basis_denial::RelationalBranchBasisDenial,
    identity::RelationalBranchIdentity,
    lifecycle::RelationalBranchLifecyclePosture,
    reference::RelationalBranchReferenceCell,
    reference_publication_cell::RelationalBranchPublicationCell,
    root::RelationalBranchRoot,
};

pub(crate) fn issue_admitted_relational_branch_basis_with_retention(
    descriptor: RelationalBranchBasisDescriptor,
    identity: RelationalBranchIdentity,
    root: Arc<RelationalBranchRoot>,
    publication_cell: RelationalBranchPublicationCell,
    retention_binding: &crate::history::retention::RelationalBranchRetentionBinding,
) -> Result<AdmittedRelationalBranchBasis, RelationalBranchBasisDenial> {
    let retention = crate::history::retention::RelationalObservationRetentionObligation::acquire(
        retention_binding,
        Arc::clone(&root),
    )
    .map_err(map_retention_denial)?;
    Ok(AdmittedRelationalBranchBasis {
        inner: Arc::new(AdmittedRelationalBranchBasisInner {
            descriptor,
            identity,
            admission_identity: RelationalBranchBasisAdmissionIdentity::issue(),
            root,
            _authority: issue_relational_branch_observation_authority(),
            retention,
            retention_binding: retention_binding.clone(),
            publication_cell,
            registry_lease: std::sync::OnceLock::new(),
        }),
    })
}

pub(super) fn map_retention_denial(
    denial: crate::history::retention::RelationalRetentionAcquisitionDenial,
) -> RelationalBranchBasisDenial {
    match denial {
        crate::history::retention::RelationalRetentionAcquisitionDenial::CapacityExhausted => {
            RelationalBranchBasisDenial::RetentionCapacityExhausted
        }
        crate::history::retention::RelationalRetentionAcquisitionDenial::OwnerUnavailable => {
            RelationalBranchBasisDenial::UnavailableRetainedTarget
        }
        crate::history::retention::RelationalRetentionAcquisitionDenial::IdentityExhausted => {
            RelationalBranchBasisDenial::RetentionIdentityExhausted
        }
        crate::history::retention::RelationalRetentionAcquisitionDenial::RootSetTooLarge
        | crate::history::retention::RelationalRetentionAcquisitionDenial::BindingMismatch => {
            RelationalBranchBasisDenial::OwnerFailure
        }
    }
}

pub(crate) fn descriptor_for_cell(
    cell: &RelationalBranchReferenceCell,
    root: &Arc<RelationalBranchRoot>,
) -> Result<RelationalBranchBasisDescriptor, RelationalBranchBasisDenial> {
    let visibility_commitment = root
        .axes()
        .map(|axes| axes.visibility.digest())
        .unwrap_or([0; 32]);
    let schema_commitment = root.schema_authority().authority_digest();
    require_root_matches_reference(&cell.observation(), root)?;
    let posture = match cell.lifecycle_posture() {
        RelationalBranchLifecyclePosture::Live => RelationalBranchBasisPosture::Live,
        RelationalBranchLifecyclePosture::Archived => RelationalBranchBasisPosture::Archived,
        RelationalBranchLifecyclePosture::Deleting => RelationalBranchBasisPosture::Deleting,
    };
    Ok(RelationalBranchBasisDescriptor::with_posture(
        super::basis::RelationalLiveBranchBasisDescriptorAxes {
            runtime_instance_id: cell.identity().runtime_instance_id(),
            branch_id: cell.identity().branch_id().clone(),
            reference: cell.observation().clone(),
            truth_version: cell.truth_version(),
            root_identity: root.id(),
            schema_commitment,
            visibility_commitment,
        },
        posture,
    ))
}
