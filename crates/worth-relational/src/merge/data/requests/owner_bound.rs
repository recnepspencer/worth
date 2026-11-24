use crate::branch::basis::AdmittedRelationalBranchBasis;
use crate::history::data::coordinates::BranchId;

use super::{MergeExecutionRequest, MergeIntent, MergePlanningRequest};

pub use super::binding_denial::RelationalMergeRequestBindingDenial;

/// Owner-issued planning request.  The raw request remains available as
/// descriptive workflow data, while this value carries exact, proof-backed
/// branch bindings and is the only production planning entry.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OwnerBoundMergePlanningRequest {
    pub(crate) request: MergePlanningRequest,
    pub(crate) target_binding: AdmittedRelationalBranchBasis,
    pub(crate) source_binding: AdmittedRelationalBranchBasis,
}

impl OwnerBoundMergePlanningRequest {
    pub(crate) fn new(
        request: MergePlanningRequest,
        target_binding: AdmittedRelationalBranchBasis,
        source_binding: AdmittedRelationalBranchBasis,
    ) -> Self {
        Self {
            request,
            target_binding,
            source_binding,
        }
    }

    pub fn request(&self) -> &MergePlanningRequest {
        &self.request
    }

    pub fn target_branch(&self) -> &BranchId {
        self.request.target_branch()
    }

    pub fn source_branch(&self) -> &BranchId {
        self.request.source_branch()
    }

    pub fn merge_intent(&self) -> MergeIntent {
        self.request.merge_intent()
    }

    pub(crate) fn into_parts(
        self,
    ) -> (
        MergePlanningRequest,
        AdmittedRelationalBranchBasis,
        AdmittedRelationalBranchBasis,
    ) {
        (self.request, self.target_binding, self.source_binding)
    }
}

/// Owner-issued execution request.  It is intentionally not serializable or
/// constructible from a public raw selector; the runtime creates it only
/// after binding both branch cells and their concrete proof.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OwnerBoundMergeExecutionRequest {
    pub(crate) request: MergeExecutionRequest,
    pub(crate) target_binding: AdmittedRelationalBranchBasis,
    pub(crate) source_binding: AdmittedRelationalBranchBasis,
}

impl OwnerBoundMergeExecutionRequest {
    pub(crate) fn new(
        request: MergeExecutionRequest,
        target_binding: AdmittedRelationalBranchBasis,
        source_binding: AdmittedRelationalBranchBasis,
    ) -> Self {
        Self {
            request,
            target_binding,
            source_binding,
        }
    }

    pub fn request(&self) -> &MergeExecutionRequest {
        &self.request
    }

    pub fn target_branch(&self) -> &BranchId {
        self.request.target_branch()
    }

    pub fn source_branch(&self) -> &BranchId {
        self.request.source_branch()
    }

    pub fn merge_intent(&self) -> MergeIntent {
        self.request.merge_intent()
    }

    pub(crate) fn into_parts(
        self,
    ) -> (
        MergeExecutionRequest,
        AdmittedRelationalBranchBasis,
        AdmittedRelationalBranchBasis,
    ) {
        (self.request, self.target_binding, self.source_binding)
    }
}

impl From<RelationalMergeRequestBindingDenial>
    for crate::branch::basis_denial::RelationalBranchBasisDenial
{
    fn from(value: RelationalMergeRequestBindingDenial) -> Self {
        match value {
            RelationalMergeRequestBindingDenial::ForeignRuntime {
                expected_runtime_instance_id,
                actual_runtime_instance_id,
            } => Self::ForeignRuntime {
                expected_runtime_instance_id,
                actual_runtime_instance_id,
            },
            RelationalMergeRequestBindingDenial::UnknownBranch(branch) => {
                Self::UnknownBranch(branch)
            }
            RelationalMergeRequestBindingDenial::ArchivedBranch(branch) => {
                Self::ArchivedBranch(branch)
            }
            RelationalMergeRequestBindingDenial::DeletingBranch(branch) => {
                Self::DeletingBranch(branch)
            }
            RelationalMergeRequestBindingDenial::RetentionCapacityExhausted => {
                Self::RetentionCapacityExhausted
            }
            RelationalMergeRequestBindingDenial::RetentionIdentityExhausted => {
                Self::RetentionIdentityExhausted
            }
            RelationalMergeRequestBindingDenial::SnapshotIdentityExhausted => {
                Self::SnapshotIdentityExhausted
            }
            RelationalMergeRequestBindingDenial::IdentityMismatch => Self::MixedAxis(
                crate::branch::basis_denial::RelationalBranchBasisMismatchAxis::Branch,
            ),
        }
    }
}
