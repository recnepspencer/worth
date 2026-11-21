use super::super::WorthQueryApplicationQueryAdmissionDenialKind;
use crate::domain_computation::primary_graph::{
    WorthQueryOperationAuthorizationDenial, WorthQueryOperationAuthorizationDenialKind,
};

/// The specific reason a live read could not be opened.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WorthQueryApplicationLiveOpenDenialKind {
    Content(worth_relational::facade::content::RelationalContentDenial),
    /// The installed query declares no live contract.
    LiveContractMissing,
    /// The live cause binding does not match the installed live contract.
    BindingMismatch,
    /// The initial query admission was refused.
    Admission(WorthQueryApplicationQueryAdmissionDenialKind),
    /// Authorization refused the read; the full denial is attached.
    AuthorizationDenied(WorthQueryOperationAuthorizationDenialKind),
    /// The scope's identity could not be encoded for cause matching.
    ScopeIdentityUnavailable,
    /// The opening basis could not be released cleanly.
    BasisReleaseFailed,
    /// The provider's current version could not be read.
    ProviderVersionUnavailable,
    /// The phase belongs to another installed application; no read was admitted.
    ForeignAdvancementPhase,
    /// The Runtime Bridge rejected the live execution basis.
    BridgeBasisRejected,
}

/// Refusal to open a live read. No subscription was opened.
///
/// [`Self::kind`] says why; for authorization causes,
/// [`Self::authorization_denial`] carries the full denial.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorthQueryApplicationLiveOpenDenial {
    kind: WorthQueryApplicationLiveOpenDenialKind,
    authorization_denial: Option<Box<WorthQueryOperationAuthorizationDenial>>,
    subject: String,
}

impl WorthQueryApplicationLiveOpenDenial {
    pub fn new(kind: WorthQueryApplicationLiveOpenDenialKind, subject: impl Into<String>) -> Self {
        Self {
            kind,
            authorization_denial: None,
            subject: subject.into(),
        }
    }

    pub fn with_authorization(
        kind: WorthQueryApplicationLiveOpenDenialKind,
        denial: WorthQueryOperationAuthorizationDenial,
    ) -> Self {
        Self {
            kind,
            subject: denial.subject().to_string(),
            authorization_denial: Some(Box::new(denial)),
        }
    }

    pub const fn kind(&self) -> WorthQueryApplicationLiveOpenDenialKind {
        self.kind
    }

    pub fn subject(&self) -> &str {
        &self.subject
    }

    pub fn authorization_denial(&self) -> Option<&WorthQueryOperationAuthorizationDenial> {
        self.authorization_denial.as_deref()
    }
}

/// Why a buffered live cause could not be turned into a delivery.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WorthQueryApplicationLiveCauseDenialKind {
    Content(worth_relational::facade::content::RelationalContentDenial),
    /// The cause's target identity could not be resolved.
    TargetIdentityUnavailable,
    /// The cause's target lies outside the subscription's scope.
    TargetOutsideScope,
    /// The read produced no result in the shape the query declares.
    ResultShapeUnavailable,
}

/// What closing a live read produced.
#[derive(Clone, Debug, Eq, PartialEq)]
#[must_use]
pub enum WorthQueryApplicationLiveCloseOutcome {
    /// The subscription ended cleanly with its read-completion evidence.
    Completed(crate::domain_computation::provider_session::WorthQueryGraphReadCompletion),
    /// The subscription had already ended or could not end cleanly.
    Unavailable,
}

/// Report that a live read's buffer overflowed and commit batches were missed.
///
/// The subscription ends; open a new live read and re-read current state.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct WorthQueryApplicationLiveOverflow {
    missed_commit_count: u64,
}

impl WorthQueryApplicationLiveOverflow {
    pub const fn new(missed_commit_count: u64) -> Self {
        Self {
            missed_commit_count,
        }
    }

    pub const fn missed_commit_batches(self) -> u64 {
        self.missed_commit_count
    }
}
