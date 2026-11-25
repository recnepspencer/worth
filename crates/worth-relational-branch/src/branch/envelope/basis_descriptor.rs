//! Transportable exact branch provenance carried by canonical envelopes.
use serde::{Deserialize, Serialize};

use crate::branch::reference::observation::RelationalBranchReferenceObservation;
use crate::branch::version::RelationalBranchVersion;
use crate::history::data::BranchId;

pub const RELATIONAL_BRANCH_BASIS_DESCRIPTOR_VERSION: u16 = 2;

/// Lifecycle posture carried by a descriptive branch basis.
///
/// Phase 6 admits only `Live`. Archive/delete transitions remain owned by the
/// later lifecycle phase, but a transported descriptor cannot erase posture.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum RelationalBranchBasisPosture {
    Live,
    Archived,
    Deleting,
}

/// Serializable description of one exact Relational branch basis.
///
/// This value is deliberately non-operational. Serialization, copying, or
/// restoration weakens freshness; only the owning runtime may resolve and
/// readmit it into an operational basis.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RelationalBranchBasisDescriptor {
    descriptor_version: u16,
    runtime_instance_id: u64,
    branch_id: BranchId,
    reference: RelationalBranchReferenceObservation,
    truth_version: RelationalBranchVersion,
    root_identity: u64,
    #[serde(default)]
    schema_commitment: worth_foundational::CanonicalDigestId,
    visibility_commitment: worth_foundational::CanonicalDigestId,
    posture: RelationalBranchBasisPosture,
}

pub struct RelationalLiveBranchBasisDescriptorAxes {
    pub runtime_instance_id: u64,
    pub branch_id: BranchId,
    pub reference: RelationalBranchReferenceObservation,
    pub truth_version: RelationalBranchVersion,
    pub root_identity: u64,
    pub schema_commitment: [u8; 32],
    pub visibility_commitment: [u8; 32],
}

impl RelationalBranchBasisDescriptor {
    pub(crate) fn with_posture(
        axes: RelationalLiveBranchBasisDescriptorAxes,
        posture: RelationalBranchBasisPosture,
    ) -> Self {
        Self {
            descriptor_version: RELATIONAL_BRANCH_BASIS_DESCRIPTOR_VERSION,
            runtime_instance_id: axes.runtime_instance_id,
            branch_id: axes.branch_id,
            reference: axes.reference,
            truth_version: axes.truth_version,
            root_identity: axes.root_identity,
            schema_commitment: worth_foundational::CanonicalDigestId::new(axes.schema_commitment),
            visibility_commitment: worth_foundational::CanonicalDigestId::new(
                axes.visibility_commitment,
            ),
            posture,
        }
    }

    pub const fn descriptor_version(&self) -> u16 {
        self.descriptor_version
    }

    pub const fn runtime_instance_id(&self) -> u64 {
        self.runtime_instance_id
    }

    pub fn branch_id(&self) -> &BranchId {
        &self.branch_id
    }

    pub fn reference(&self) -> &RelationalBranchReferenceObservation {
        &self.reference
    }

    pub const fn truth_version(&self) -> RelationalBranchVersion {
        self.truth_version
    }

    pub const fn root_identity(&self) -> u64 {
        self.root_identity
    }

    pub const fn schema_commitment(&self) -> [u8; 32] {
        *self.schema_commitment.bytes()
    }

    pub const fn visibility_commitment(&self) -> [u8; 32] {
        *self.visibility_commitment.bytes()
    }

    pub const fn posture(&self) -> RelationalBranchBasisPosture {
        self.posture
    }
}

/// Operations used by the Relational owners.
pub trait RelationalBranchBasisDescriptorOwnerPort: Sized {
    fn with_reference(&self, reference: RelationalBranchReferenceObservation) -> Self;
    fn with_recorded_posture(
        axes: RelationalLiveBranchBasisDescriptorAxes,
        posture: RelationalBranchBasisPosture,
    ) -> Self;
}

impl RelationalBranchBasisDescriptorOwnerPort for RelationalBranchBasisDescriptor {
    fn with_recorded_posture(
        axes: RelationalLiveBranchBasisDescriptorAxes,
        posture: RelationalBranchBasisPosture,
    ) -> Self {
        Self::with_posture(axes, posture)
    }
    fn with_reference(&self, reference: RelationalBranchReferenceObservation) -> Self {
        Self::with_posture(
            RelationalLiveBranchBasisDescriptorAxes {
                runtime_instance_id: self.runtime_instance_id,
                branch_id: self.branch_id.clone(),
                reference,
                truth_version: self.truth_version,
                root_identity: self.root_identity,
                schema_commitment: *self.schema_commitment.bytes(),
                visibility_commitment: *self.visibility_commitment.bytes(),
            },
            self.posture,
        )
    }
}
