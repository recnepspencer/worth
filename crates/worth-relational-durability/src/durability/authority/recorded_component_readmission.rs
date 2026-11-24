//! Admit the exact source-produced image, never a rewritten runtime descriptor.
use contracts::durability::data::recovery_stop::RelationalRecoveryStop;
use worth_relational_branch::internal_port::branch::basis::AdmittedRelationalBranchBasis;
use worth_relational_contracts::internal_port as contracts;

#[derive(Debug)]
pub enum RelationalRecordedComponentAdmissionCause {
    Request(contracts::execution::denial_cause::RelationalExecutionDenialCause),
    ForeignRecoveryRuntime,
    OwnerUnavailable,
    MissingRecoveredImage,
    RecordedTargetMismatch,
    MemoryOnly,
    CanonicalEncoding { detail: String },
    Source(RelationalRecoveryStop),
    Basis(contracts::branch::basis_denial::RelationalBranchBasisDenial),
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

#[doc(hidden)]
pub trait RefusedRelationalRecordedComponentOwnerPort: Sized {
    fn from_refusal(
        cause: RelationalRecordedComponentAdmissionCause,
        acquired: Option<AdmittedRelationalBranchBasis>,
    ) -> Self;
}

impl RefusedRelationalRecordedComponentOwnerPort for RefusedRelationalRecordedComponent {
    fn from_refusal(
        cause: RelationalRecordedComponentAdmissionCause,
        acquired: Option<AdmittedRelationalBranchBasis>,
    ) -> Self {
        Self { cause, acquired }
    }
}
