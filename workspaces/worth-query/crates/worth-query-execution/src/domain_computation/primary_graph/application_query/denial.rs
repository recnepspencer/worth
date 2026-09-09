use worth_query_admission::facade::{
    application_query::WorthQueryApplicationQueryParameterDenialKind,
    graph_read_access::WorthQueryGraphReadPlanReviewDenialKind,
};
use worth_query_installation::facade::WorthQueryApplicationQueryInstallationDenialKind;

use crate::domain_computation::primary_graph::{
    WorthQueryOperationAuthorizationDenial, WorthQueryOperationAuthorizationDenialKind,
};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WorthQueryApplicationQueryAdmissionDenialKind {
    InstalledQuery(WorthQueryApplicationQueryInstallationDenialKind),
    ForeignPrincipal,
    ForeignScope,
    StalePrincipal,
    StaleScope,
    ScopeTypeMismatch,
    Authorization(WorthQueryOperationAuthorizationDenialKind),
    Cancelled,
    DeadlineExceeded,
    BasisUnsupported,
    ForeignBasis,
    StaleBasis,
    WrongProviderBasis,
    ExpiredBasis,
    BasisUnavailable,
    ForeignHistoricalReceipt,
    RuntimeSupportUnavailable,
    ActiveSnapshotCapacityExhausted { maximum_active_snapshots: usize },
    SnapshotIdentityExhausted,
    RetentionCapacityExhausted,
    RetentionIdentityExhausted,
    ForeignContinuation,
    StaleContinuation,
    ContinuationParameterMismatch,
    ContinuationScopeMismatch,
    ContinuationProviderMismatch,
    ContinuationPageWidthUnsupported,
    LaneUnsupported,
    DisclosureGovernanceRequired,
    DisclosureContractInvalid,
    DisclosureAuthorizationMismatch,
    InternalComputationDenied,
    Parameter(WorthQueryApplicationQueryParameterDenialKind),
    WorkLimitExceeded,
    CanonicalWorkDenied,
    GraphReadPlan(WorthQueryGraphReadPlanReviewDenialKind),
    GraphWorkAdmissionUnavailable,
    ExecutionShapeUnsupported,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorthQueryApplicationQueryAdmissionDenial {
    kind: WorthQueryApplicationQueryAdmissionDenialKind,
    authorization_denial: Option<Box<WorthQueryOperationAuthorizationDenial>>,
    subject: String,
}

impl WorthQueryApplicationQueryAdmissionDenial {
    pub(super) fn new(
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
            subject: denial.subject().to_string(),
            authorization_denial: Some(Box::new(denial)),
        }
    }

    pub const fn kind(&self) -> WorthQueryApplicationQueryAdmissionDenialKind {
        self.kind
    }

    pub fn subject(&self) -> &str {
        &self.subject
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
            self.kind, self.subject
        )
    }
}

impl std::error::Error for WorthQueryApplicationQueryAdmissionDenial {}
