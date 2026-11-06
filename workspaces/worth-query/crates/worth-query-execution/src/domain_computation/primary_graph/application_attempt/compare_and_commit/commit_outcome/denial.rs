//! Pre-publication application denial categories and owner evidence.
use crate::domain_computation::primary_graph::vocabulary::application_attempt::denial::WorthQueryApplicationCommitDenialKind;
use crate::domain_computation::primary_graph::vocabulary::application_attempt::denial::WorthQueryApplicationCommitDenialStage;

mod advancement;
mod authored_retirement;
mod capacity;
mod decision_read_set;
mod denial_cause;
mod domain_value;
mod effect_lowering;
mod execution;
mod invariant_execution;
mod invariant_refusal;
mod lane;
mod program_binding;
mod recorded_idempotency;
mod relation_law;
mod request_authority;
mod signed_creation;
mod signing_lifecycle;
mod unobserved_write;
mod workflow;

/// Which fail-closed integrity path refused to attribute one occurrence's
/// branch program activation.
///
/// These are distinct causes for the same refusal: a host that never seeded
/// activation, a snapshot that cannot produce the record, and a record naming
/// meaning this host never admitted are three different operator situations.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(in crate::domain_computation::primary_graph) enum WorthQueryProgramActivationUnresolved {
    /// The branch never published an activation record at all.
    NeverPublished,
    /// The activation record exists but this attempt's snapshot cannot read it.
    Unreadable,
    /// The activation record renders a program this host never rostered.
    NamesNoRosteredProgram,
}

impl WorthQueryProgramActivationUnresolved {
    const fn detail(self) -> &'static str {
        match self {
            Self::NeverPublished => "branch program activation was never published",
            Self::Unreadable => "branch program activation is unreadable on this occurrence",
            Self::NamesNoRosteredProgram => "branch program activation names no rostered program",
        }
    }
}


/// A typed refusal of a commit before publication, carried by the `Denied`
/// commit outcome.
///
/// Nothing was committed. Match on [`kind`](Self::kind); the stage and detail
/// locate the refusal, and the custom-invariant accessors explain a
/// `CustomInvariantDenied` kind.
#[derive(Debug)]
pub struct WorthQueryApplicationCommitDenial {
    kind: WorthQueryApplicationCommitDenialKind,
    stage: WorthQueryApplicationCommitDenialStage,
    detail: Option<std::sync::Arc<str>>,
    cause: Option<Box<denial_cause::DenialCause>>,
}

impl WorthQueryApplicationCommitDenial {
    pub(crate) fn correction_identity_mismatch(detail: &'static str) -> Self {
        Self {
            kind: WorthQueryApplicationCommitDenialKind::CorrectionIdentityMismatch,
            stage: WorthQueryApplicationCommitDenialStage::ProposalBinding,
            detail: Some(detail.into()),
            cause: None,
        }
    }
    pub(crate) const fn correction_context_mismatch() -> Self {
        Self {
            kind: WorthQueryApplicationCommitDenialKind::CorrectionContextMismatch,
            stage: WorthQueryApplicationCommitDenialStage::ProposalBinding,
            detail: None,
            cause: None,
        }
    }
    pub const fn kind(&self) -> WorthQueryApplicationCommitDenialKind {
        self.kind
    }

    pub const fn stage(&self) -> WorthQueryApplicationCommitDenialStage {
        self.stage
    }

    /// Returns provider-owned diagnostic detail when the rejection boundary
    /// supplies a concrete cause.
    pub fn detail(&self) -> Option<&str> {
        self.detail.as_deref()
    }

    pub fn custom_invariant_denial(
        &self,
    ) -> Option<&crate::domain_computation::vocabulary::provider_session::invariant_execution_denial::WorthQueryCustomInvariantDenial> {
        self.custom_invariant()
    }

    pub fn custom_invariant_violation_identity(
        &self,
    ) -> Option<&worth_relational::facade::transactions::CustomInvariantSemanticIdentity> {
        match self.custom_invariant()? {
            crate::domain_computation::vocabulary::provider_session::invariant_execution_denial::WorthQueryCustomInvariantDenial::Violation {
                identity,
                ..
            } => Some(identity),
            crate::domain_computation::vocabulary::provider_session::invariant_execution_denial::WorthQueryCustomInvariantDenial::Failure { .. } => None,
        }
    }

    pub fn custom_invariant_failure(
        &self,
    ) -> Option<(
        &worth_relational::facade::transactions::CustomInvariantFailureIdentity,
        worth_relational::facade::transactions::CustomInvariantFailurePhase,
        worth_relational::facade::transactions::ResultCustomInvariantFailureKind,
    )> {
        match self.custom_invariant()? {
            crate::domain_computation::vocabulary::provider_session::invariant_execution_denial::WorthQueryCustomInvariantDenial::Failure {
                identity,
                phase,
                failure,
            } => Some((identity, *phase, *failure)),
            crate::domain_computation::vocabulary::provider_session::invariant_execution_denial::WorthQueryCustomInvariantDenial::Violation { .. } => None,
        }
    }

    pub(in crate::domain_computation::primary_graph::application_attempt) const fn provider_rejected(
        stage: WorthQueryApplicationCommitDenialStage,
    ) -> Self {
        Self {
            kind: WorthQueryApplicationCommitDenialKind::ProviderRejected,
            stage,
            detail: None,
            cause: None,
        }
    }

    pub(in crate::domain_computation::primary_graph::application_attempt) fn provider_rejected_with_detail(
        stage: WorthQueryApplicationCommitDenialStage,
        detail: impl Into<std::sync::Arc<str>>,
    ) -> Self {
        Self {
            kind: WorthQueryApplicationCommitDenialKind::ProviderRejected,
            stage,
            detail: Some(detail.into()),
            cause: None,
        }
    }

    pub(in crate::domain_computation::primary_graph::application_attempt) const fn idempotency_intent_drift(
    ) -> Self {
        Self {
            kind: WorthQueryApplicationCommitDenialKind::IdempotencyIntentDrift,
            stage: WorthQueryApplicationCommitDenialStage::Idempotency,
            detail: None,
            cause: None,
        }
    }

    pub(in crate::domain_computation::primary_graph::application_attempt) const fn mutation_binding_mismatch(
    ) -> Self {
        Self {
            kind: WorthQueryApplicationCommitDenialKind::MutationBindingMismatch,
            stage: WorthQueryApplicationCommitDenialStage::Idempotency,
            detail: None,
            cause: None,
        }
    }

    pub(in crate::domain_computation::primary_graph::application_attempt) const fn mutation_input_mismatch(
    ) -> Self {
        Self {
            kind: WorthQueryApplicationCommitDenialKind::MutationInputMismatch,
            stage: WorthQueryApplicationCommitDenialStage::Idempotency,
            detail: None,
            cause: None,
        }
    }

    pub(in crate::domain_computation::primary_graph::application_attempt) const fn operation_body_execution_required(
    ) -> Self {
        Self {
            kind: WorthQueryApplicationCommitDenialKind::OperationBodyExecutionRequired,
            stage: WorthQueryApplicationCommitDenialStage::ProposalBinding,
            detail: None,
            cause: None,
        }
    }

    pub(in crate::domain_computation) const fn application_program_required() -> Self {
        Self {
            kind: WorthQueryApplicationCommitDenialKind::ApplicationProgramRequired,
            stage: WorthQueryApplicationCommitDenialStage::ProposalBinding,
            detail: None,
            cause: None,
        }
    }
}

mod court_doors_for_test;
