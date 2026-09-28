//! Undo denial causes (R8.39).

/// Typed undo denial. Each variant is a distinct cause — no shared fallback.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WorthQueryUndoDenialKind {
    /// Legal hold / irreversible legal classification.
    IrreversibleLegal,
    /// Audit retention forbids reversal.
    IrreversibleAudit,
    /// Approval / elevation outcome cannot be undone.
    IrreversibleApproval,
    /// Released estate — also type-level absence of undo on the outcome (R8.21).
    ReleasedEstate,
    /// Escaped effect without a compensatable/reconcilable path.
    EscapedEffect,
    /// Binding or receipt is stale against current truth.
    Stale,
    /// Conflicted against current graph/policy.
    Conflicted,
    /// Recovery handle or undo intent already consumed.
    AlreadyConsumed,
    /// Installed aftermath does not admit undo for this axis pair.
    CorrectionNotAdmitted,
    /// Mutation work is missing commit-derived touched records (C2).
    TouchedRecordsRequired,
    /// Fresh capability/policy admission denied — receipt is not current authority.
    CurrentPolicyDenied,
    /// Foundational material alone cannot authorize undo (R8.41).
    FoundationalNotAuthority,
    /// Foreign runtime / principal / handle.
    ForeignHandle,
    /// Recorded inverse requires the receipt's retained pre-image (R8.2).
    RetainedPreImageRequired,
    /// Correction derivation requires the exact canonically bound original input.
    OriginalGovernedInputRequired,
    /// Installed inverse correspondence is missing from the admitted request.
    LoweringCorrespondenceRequired,
}

/// Refusal to admit or progress an undo of a committed change.
///
/// Returned by undo admission, progression, and continuation closing. A denial
/// from admission or progression means the undo never reached the mutation
/// lane. Read [`Self::kind`] for the exact cause. Kinds such as
/// `IrreversibleLegal` or `EscapedEffect` are reached only by the runtime,
/// from the installed contract.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorthQueryUndoDenial {
    kind: WorthQueryUndoDenialKind,
}

impl WorthQueryUndoDenial {
    pub(crate) const fn new(kind: WorthQueryUndoDenialKind) -> Self {
        Self { kind }
    }

    pub const fn kind(&self) -> WorthQueryUndoDenialKind {
        self.kind
    }

    // Only the kinds a downstream owner actually reports for its own checks are
    // public. `IrreversibleLegal`, `IrreversibleAudit`, `IrreversibleApproval`,
    // `ReleasedEstate`, `EscapedEffect`, and `OriginalGovernedInputRequired` are
    // classified here from the installed contract or derived inside admission,
    // so a public constructor for them would only let a caller author a verdict
    // the runtime is supposed to reach. They are reachable through `kind()` on a
    // denial the runtime returned; they are not mintable.

    pub const fn stale() -> Self {
        Self::new(WorthQueryUndoDenialKind::Stale)
    }

    pub const fn conflicted() -> Self {
        Self::new(WorthQueryUndoDenialKind::Conflicted)
    }

    pub const fn already_consumed() -> Self {
        Self::new(WorthQueryUndoDenialKind::AlreadyConsumed)
    }

    pub const fn correction_not_admitted() -> Self {
        Self::new(WorthQueryUndoDenialKind::CorrectionNotAdmitted)
    }

    pub const fn current_policy_denied() -> Self {
        Self::new(WorthQueryUndoDenialKind::CurrentPolicyDenied)
    }

    pub const fn touched_records_required() -> Self {
        Self::new(WorthQueryUndoDenialKind::TouchedRecordsRequired)
    }

    pub const fn retained_preimage_required() -> Self {
        Self::new(WorthQueryUndoDenialKind::RetainedPreImageRequired)
    }
}
