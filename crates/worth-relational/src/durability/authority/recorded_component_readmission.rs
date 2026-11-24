//! Admit the exact source-produced image, never a rewritten runtime descriptor.
use crate::branch::basis::AdmittedRelationalBranchBasis;
use crate::durability::data::RelationalRecoveryStop;

#[derive(Debug)]
pub enum RelationalRecordedComponentAdmissionCause {
    Request(crate::execution::RelationalExecutionDenialCause),
    ForeignRecoveryRuntime,
    OwnerUnavailable,
    MissingRecoveredImage,
    RecordedTargetMismatch,
    MemoryOnly,
    CanonicalEncoding { detail: String },
    Source(RelationalRecoveryStop),
    Basis(crate::branch::basis_denial::RelationalBranchBasisDenial),
}

/// Failed admission preserves any actual pin acquired before refusal. It
/// exposes no operational basis through the refusal branch.
#[derive(Debug)]
pub struct RefusedRelationalRecordedComponent {
    pub(super) cause: RelationalRecordedComponentAdmissionCause,
    pub(super) acquired: Option<AdmittedRelationalBranchBasis>,
}

impl RefusedRelationalRecordedComponent {
    pub fn cause(&self) -> &RelationalRecordedComponentAdmissionCause {
        &self.cause
    }
    pub fn retains_acquired_custody(&self) -> bool {
        self.acquired.is_some()
    }
}

mod runtime_admission;
