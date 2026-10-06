use worth_query_admission::facade::{
    application_query::WorthQueryApplicationQueryParameterDenialKind,
    graph_read_access::WorthQueryGraphReadPlanReviewDenialKind,
};
use worth_query_installation::facade::WorthQueryApplicationQueryInstallationDenialKind;

use crate::domain_computation::primary_graph::{
    WorthQueryOperationAuthorizationDenial, WorthQueryOperationAuthorizationDenialKind,
};

/// The specific reason a query read was refused at admission.
///
/// Admission runs before any read, so nothing was read or disclosed. Families:
/// installed query and parameters; principal and scope; authorization; basis
/// and branch selection; capacity and identity exhaustion; continuation
/// binding; disclosure governance; and work limits.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum WorthQueryApplicationQueryAdmissionDenialKind {
    /// The query is not installed as requested.
    InstalledQuery(WorthQueryApplicationQueryInstallationDenialKind),
    /// The principal belongs to a different runtime.
    ForeignPrincipal,
    /// The scope belongs to a different runtime.
    ForeignScope,
    /// The principal no longer resolves as it did.
    StalePrincipal,
    /// The request scope no longer resolves as it did.
    StaleScope,
    /// The request scope is not the kind the query binding expects.
    ScopeTypeMismatch,
    /// Authorization refused the read; the full denial is attached.
    Authorization(WorthQueryOperationAuthorizationDenialKind),
    /// Admission was cancelled.
    Cancelled,
    /// Admission reached its deadline.
    DeadlineExceeded,
    /// The requested basis is not supported for this query.
    BasisUnsupported,
    /// The basis belongs to a different runtime.
    ForeignBasis,
    /// The basis is no longer current.
    StaleBasis,
    /// The basis belongs to a different provider.
    WrongProviderBasis,
    /// The retained basis has expired.
    ExpiredBasis,
    /// The basis could not be selected.
    BasisUnavailable,
    /// The branch's materialization is suspended, so it cannot be read now.
    BranchMaterializationSuspended,
    /// The commit receipt for a historical read belongs to a different runtime.
    ForeignHistoricalReceipt,
    /// The runtime support the read needs is unavailable.
    RuntimeSupportUnavailable,
    /// The installed limit on concurrently active snapshots was reached.
    ActiveSnapshotCapacityExhausted { maximum_active_snapshots: usize },
    /// The runtime ran out of snapshot identities.
    SnapshotIdentityExhausted,
    /// No capacity remains to retain the basis.
    RetentionCapacityExhausted,
    /// The runtime ran out of basis-retention identities.
    RetentionIdentityExhausted,
    /// The continuation belongs to a different runtime or plan.
    ForeignContinuation,
    /// The continuation is no longer current.
    StaleContinuation,
    /// The parameters differ from the ones the continuation was issued for.
    ContinuationParameterMismatch,
    /// The scope differs from the one the continuation was issued for.
    ContinuationScopeMismatch,
    /// The provider differs from the one the continuation was issued for.
    ContinuationProviderMismatch,
    /// The requested page width is not supported.
    ContinuationPageWidthUnsupported,
    /// The query does not support the requested read lane.
    LaneUnsupported,
    /// The query's results need governed disclosure and none was admitted.
    DisclosureGovernanceRequired,
    /// The installed disclosure contract is invalid.
    DisclosureContractInvalid,
    /// The disclosure authorization does not match this read.
    DisclosureAuthorizationMismatch,
    /// The authorization does not cover a disclosure the query computes over internally.
    InternalComputationDenied,
    /// The query parameters were refused.
    Parameter(WorthQueryApplicationQueryParameterDenialKind),
    /// The read would exceed its work limit.
    WorkLimitExceeded,
    /// The carried request cannot hold a fresh retained-parameter preparation.
    ReadmissionPreparationMemoryExhausted,
    /// The bounded canonical work for admission was refused.
    CanonicalWorkDenied,
    /// The graph read plan failed review.
    GraphReadPlan(WorthQueryGraphReadPlanReviewDenialKind),
    /// Graph work for the read could not be admitted.
    GraphWorkAdmissionUnavailable,
    /// The runtime cannot execute the query's shape.
    ExecutionShapeUnsupported,
}

/// Refusal to admit an application query read.
///
/// Nothing was read or disclosed. [`Self::kind`] says why; for authorization
/// causes, [`Self::authorization_denial`] carries the full denial.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorthQueryApplicationQueryAdmissionDenial {
    kind: WorthQueryApplicationQueryAdmissionDenialKind,
    authorization_denial: Option<Box<WorthQueryOperationAuthorizationDenial>>,
    subject: String,
}

impl WorthQueryApplicationQueryAdmissionDenial {
    pub(in crate::domain_computation::primary_graph) fn new(
        kind: WorthQueryApplicationQueryAdmissionDenialKind,
        subject: impl Into<String>,
    ) -> Self {
        Self {
            kind,
            authorization_denial: None,
            subject: subject.into(),
        }
    }

    pub(super) fn from_authorization(denial: WorthQueryOperationAuthorizationDenial) -> Self {
        Self {
            kind: WorthQueryApplicationQueryAdmissionDenialKind::Authorization(denial.kind()),
            // The boxed denial owns the sole subject. Duplicating it here
            // would add an unretained allocation to the admitted path.
            subject: String::new(),
            authorization_denial: Some(Box::new(denial)),
        }
    }

    pub const fn kind(&self) -> WorthQueryApplicationQueryAdmissionDenialKind {
        self.kind
    }

    pub fn subject(&self) -> &str {
        self.authorization_denial
            .as_ref()
            .map_or(self.subject.as_str(), |denial| denial.subject())
    }

    pub fn authorization_denial(&self) -> Option<&WorthQueryOperationAuthorizationDenial> {
        self.authorization_denial.as_deref()
    }

    pub(super) fn into_authorization_denial(
        self,
    ) -> Option<WorthQueryOperationAuthorizationDenial> {
        self.authorization_denial.map(|denial| *denial)
    }
}

impl std::fmt::Display for WorthQueryApplicationQueryAdmissionDenial {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            formatter,
            "application query admission denied: {:?} ({})",
            self.kind,
            self.subject()
        )
    }
}

impl std::error::Error for WorthQueryApplicationQueryAdmissionDenial {}
