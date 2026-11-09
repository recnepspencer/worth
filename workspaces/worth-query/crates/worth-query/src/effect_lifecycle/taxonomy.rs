#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub enum EffectFamily {
    Mutation,
    Merge,
    Writeback,
}

impl EffectFamily {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Mutation => "mutation",
            Self::Merge => "merge",
            Self::Writeback => "writeback",
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum EffectAuthorityLane {
    Relational,
    RuntimeBridge,
}

impl EffectAuthorityLane {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Relational => "relational",
            Self::RuntimeBridge => "runtime_bridge",
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum EffectIntentDenialKind {
    EvidenceIdentityDerivationDenied(
        crate::evidence_identity::WorthQueryEvidenceIdentityDerivationDenial,
    ),
    WorkflowDeclarationFamilyMismatch,
    WorkflowAuthorityTargetMismatch,
    BasisWorkflowBindingMismatch,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DeniedEffectEligibilityKind {
    UnsupportedForBasisFamily,
    BranchAuthorityRequired,
    PreviewRebindRequired,
    PreviewReadOnlyExecutionForbidden,
    AuthorityTargetMismatch,
    WorkflowBroadeningForbidden,
    StoreBackedExecutionDeferred,
    DurableReplayDeferred,
    WorkflowAdmissionDenied,
}

impl DeniedEffectEligibilityKind {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::UnsupportedForBasisFamily => "unsupported_for_basis_family",
            Self::BranchAuthorityRequired => "branch_authority_required",
            Self::PreviewRebindRequired => "preview_rebind_required",
            Self::PreviewReadOnlyExecutionForbidden => "preview_read_only_execution_forbidden",
            Self::AuthorityTargetMismatch => "authority_target_mismatch",
            Self::WorkflowBroadeningForbidden => "workflow_broadening_forbidden",
            Self::StoreBackedExecutionDeferred => "store_backed_execution_deferred",
            Self::DurableReplayDeferred => "durable_replay_deferred",
            Self::WorkflowAdmissionDenied => "workflow_admission_denied",
        }
    }
}
