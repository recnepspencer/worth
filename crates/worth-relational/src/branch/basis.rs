mod retirement;
pub use retirement::{
    RelationalBranchBasisRetirement, RelationalBranchBasisRetirementDenial,
    RelationalBranchBasisRetirementProgress, RetiredRelationalBranchBasis,
};
mod component_capture;
pub(crate) use component_capture::CheckpointComponentDescriptorView;
use std::sync::{Arc, OnceLock};

use crate::history::data::BranchId;

pub use super::basis_descriptor::{
    RelationalBranchBasisDescriptor, RelationalBranchBasisPosture,
    RELATIONAL_BRANCH_BASIS_DESCRIPTOR_VERSION,
};
use crate::branch::identity::RelationalBranchIdentity;
use crate::branch::reference::observation::RelationalBranchReferenceObservation;
use crate::branch::root::RelationalBranchRoot;
use crate::branch::version::RelationalBranchVersion;

/// Structurally valid but still non-operational descriptor.
///
/// Resolution proves protocol shape and cross-field consistency only. It does
/// not prove owner affinity, currentness, retained-root availability, or live
/// lifecycle posture.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResolvedRelationalBasisDescriptor {
    descriptor: RelationalBranchBasisDescriptor,
}

impl ResolvedRelationalBasisDescriptor {
    pub(crate) fn new(descriptor: RelationalBranchBasisDescriptor) -> Self {
        Self { descriptor }
    }

    pub fn descriptor(&self) -> &RelationalBranchBasisDescriptor {
        &self.descriptor
    }

    pub(crate) fn into_descriptor(self) -> RelationalBranchBasisDescriptor {
        self.descriptor
    }
}
/// Owner-admitted immutable read basis.
///
/// Clones share one owner-issued lease and one immutable root. Cloning never
/// revalidates, reacquires retention, copies truth, or contacts live branch
/// state.
#[derive(Clone, Debug)]
pub struct AdmittedRelationalBranchBasis {
    pub(crate) inner: Arc<AdmittedRelationalBranchBasisInner>,
}

impl<'basis> From<&'basis AdmittedRelationalBranchBasis>
    for crate::history::data::canonical_commit_envelope::required_output_origin::restoration_candidate::RestorationPrestate<'basis>
{
    fn from(basis: &'basis AdmittedRelationalBranchBasis) -> Self {
        Self {
            partitions: basis.inner.root.as_ref(),
            aspect_plans: basis.inner.root.schema_authority().aspect_plans(),
        }
    }
}

impl PartialEq for AdmittedRelationalBranchBasis {
    fn eq(&self, other: &Self) -> bool {
        self.inner.descriptor == other.inner.descriptor
            && self.inner.identity == other.inner.identity
            && self.inner.root.id() == other.inner.root.id()
            && self.inner.admission_identity == other.inner.admission_identity
    }
}

impl Eq for AdmittedRelationalBranchBasis {}

#[derive(Debug)]
pub(crate) struct AdmittedRelationalBranchBasisInner {
    pub(crate) descriptor: RelationalBranchBasisDescriptor,
    pub(crate) identity: RelationalBranchIdentity,
    pub(crate) admission_identity:
        crate::branch::basis_admission_identity::RelationalBranchBasisAdmissionIdentity,
    pub(crate) root: Arc<RelationalBranchRoot>,
    pub(crate) _authority: crate::branch::authority::RelationalBranchObservationAuthority,
    pub(crate) retention: crate::history::retention::RelationalObservationRetentionObligation,
    pub(crate) retention_binding: crate::history::retention::RelationalBranchRetentionBinding,
    pub(crate) publication_cell:
        crate::branch::reference_publication_cell::RelationalBranchPublicationCell,
    pub(crate) registry_lease: OnceLock<super::basis_registry::RelationalBasisRegistryLease>,
}

impl AdmittedRelationalBranchBasis {
    /// Observe only the exact original cell's lock posture. This grants no
    /// currentness or publication authority and never waits for that cell.
    #[doc(hidden)]
    pub fn publication_state_locked_for_test(&self) -> bool {
        self.inner
            .publication_cell
            .publication_state_locked_for_test()
    }
    pub fn materialization_is_complete(&self) -> bool {
        !self.inner.root.has_materialization_unavailable()
    }

    /// Identity issued by the Relational owner for this exact admission.
    ///
    /// The identity is descriptive binding for later composition; it is not
    /// a descriptor and cannot be used to mint or authorize a basis.
    pub fn admission_identity(
        &self,
    ) -> &crate::branch::basis_admission_identity::RelationalBranchBasisAdmissionIdentity {
        &self.inner.admission_identity
    }

    pub fn descriptor(&self) -> &RelationalBranchBasisDescriptor {
        &self.inner.descriptor
    }

    pub fn identity(&self) -> &RelationalBranchIdentity {
        &self.inner.identity
    }

    pub fn reference(&self) -> &RelationalBranchReferenceObservation {
        self.descriptor().reference()
    }

    pub fn truth_version(&self) -> RelationalBranchVersion {
        self.descriptor().truth_version()
    }

    pub fn retention_reason(&self) -> crate::history::retention::RelationalBasisRetentionReason {
        self.inner.retention.reason()
    }

    pub(crate) fn publication_cell(
        &self,
    ) -> &crate::branch::reference_publication_cell::RelationalBranchPublicationCell {
        &self.inner.publication_cell
    }

    pub(crate) fn is_current(&self) -> bool {
        let cell = self.publication_cell();
        let state = cell.enter_state();
        cell.identity() == self.identity()
            && state.lifecycle_posture()
                == crate::branch::lifecycle::RelationalBranchLifecyclePosture::Live
            && state.observation() == self.reference()
            && state.truth_version() == self.truth_version()
    }

    /// Commit identity carried by this exact owner-admitted immutable root.
    pub(crate) fn commit_identity(
        &self,
    ) -> Option<crate::history::commit::RelationalCommitIdentity> {
        let target = match self.reference().target() {
            worth_foundational::FoundationalBranchTarget::Empty => return None,
            worth_foundational::FoundationalBranchTarget::Basis(target) => target,
        };
        let receipt = self.inner.root.commit_receipt()?;
        (receipt.commit_id.0 == target.selected_commit_id()).then(|| {
            crate::history::commit::RelationalCommitIdentity::new(
                receipt.commit_id,
                receipt.version_id,
                receipt.branch_id.clone(),
            )
        })
    }
}
#[cfg(test)]
#[path = "basis_inline_type_budget.rs"]
pub(crate) mod inline_type_budget;
