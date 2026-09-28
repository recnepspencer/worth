use crate::domain_computation::primary_graph::{
    WorthQueryOperationAuthorizationDenial, WorthQueryOperationAuthorizationDenialKind,
};

/// The specific reason reading the next page of a continuation was refused.
///
/// No page was returned. Stale, foreign, and generation causes mean the
/// continuation can no longer be used: run the query again. Capacity, limit,
/// cancellation, and deadline causes concern this attempt only.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WorthQueryApplicationContinuationDenialKind {
    /// The continuation belongs to a different plan or runtime.
    ForeignPlan,
    /// The installed query changed since the continuation was issued.
    StaleInstalledQuery,
    /// The principal no longer resolves as it did.
    StalePrincipal,
    /// The request scope no longer resolves as it did.
    StaleScope,
    /// Authorization refused the page; the full denial is attached.
    Authorization(WorthQueryOperationAuthorizationDenialKind),
    /// The read was cancelled.
    Cancelled,
    /// The read reached its deadline.
    DeadlineExceeded,
    /// The retained basis for the continuation is not available.
    BasisUnavailable,
    /// No capacity remains to retain the basis.
    RetentionCapacityExhausted,
    /// The runtime ran out of basis-retention identities.
    RetentionIdentityExhausted,
    /// The runtime ran out of snapshot identities.
    SnapshotIdentityExhausted,
    /// The retained basis has expired.
    ExpiredBasis,
    /// The basis used for the page could not be released cleanly.
    BasisReleaseFailed,
    /// The predicate index the query needs is unavailable.
    PredicateIndexUnavailable,
    /// A predicate lookup exceeded its bound.
    PredicateLookupOverflow,
    /// The page would exceed the result limit.
    ResultLimitExceeded,
    /// The read produced a result count the query shape does not allow.
    CardinalityMismatch,
    /// A graph traversal the query needs is unavailable.
    TraversalUnavailable,
    /// The ordered index the continuation seeks through is unavailable.
    ContinuationIndexUnavailable,
    /// The continuation's resume boundary was rejected.
    ContinuationBoundaryRejected,
    /// The requested page width is not valid for this continuation.
    ContinuationPageWidthInvalid,
    /// The index generation changed since the continuation was issued.
    ContinuationGenerationChanged,
    /// The result projection is unavailable.
    ProjectionUnavailable,
    /// Projecting a result row failed.
    Projection(super::super::WorthQueryApplicationProjectionDenialKind),
    /// The page would exceed the result buffer limit.
    ResultBufferLimitExceeded,
    /// The page would exceed the work limit.
    WorkLimitExceeded,
}

/// Refusal to read the next page of a query continuation.
///
/// No page was returned. [`Self::kind`] says why; for authorization causes,
/// [`Self::authorization_denial`] carries the full denial.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorthQueryApplicationContinuationDenial {
    kind: WorthQueryApplicationContinuationDenialKind,
    authorization_denial: Option<Box<WorthQueryOperationAuthorizationDenial>>,
    subject: String,
}

impl WorthQueryApplicationContinuationDenial {
    pub(super) fn new(
        kind: WorthQueryApplicationContinuationDenialKind,
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
            kind: WorthQueryApplicationContinuationDenialKind::Authorization(denial.kind()),
            subject: denial.subject().to_string(),
            authorization_denial: Some(Box::new(denial)),
        }
    }

    pub const fn kind(&self) -> WorthQueryApplicationContinuationDenialKind {
        self.kind
    }

    pub fn subject(&self) -> &str {
        &self.subject
    }

    pub fn authorization_denial(&self) -> Option<&WorthQueryOperationAuthorizationDenial> {
        self.authorization_denial.as_deref()
    }
}

impl std::fmt::Display for WorthQueryApplicationContinuationDenial {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            formatter,
            "application-query continuation denied: {:?} ({})",
            self.kind, self.subject
        )
    }
}

impl std::error::Error for WorthQueryApplicationContinuationDenial {}
