use super::super::performance::QueryContextCounters;
use crate::evidence_identity::WorthQueryEvidenceIdentityDerivationDenial as IdentityDerivationDenial;

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum QueryContextFamily {
    CurrentBranchHead,
    BranchHead,
    HistoricalSnapshot,
    HistoricalCommit,
    PreviewDerivedHistorical,
    DiffComparison,
}

impl QueryContextFamily {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::CurrentBranchHead => "current_branch_head",
            Self::BranchHead => "branch_head",
            Self::HistoricalSnapshot => "historical_snapshot",
            Self::HistoricalCommit => "historical_commit",
            Self::PreviewDerivedHistorical => "preview_derived_historical",
            Self::DiffComparison => "diff_comparison",
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ComparisonBasisFamily {
    BranchToBranch,
    CurrentToHistorical,
    HistoricalToHistorical,
    PreviewToAuthoritative,
}

impl ComparisonBasisFamily {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::BranchToBranch => "branch_to_branch",
            Self::CurrentToHistorical => "current_to_historical",
            Self::HistoricalToHistorical => "historical_to_historical",
            Self::PreviewToAuthoritative => "preview_to_authoritative",
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum HistoricalAdmissionClass {
    RuntimeRetained,
    RuntimeReplay,
    RuntimeReconstruction,
    StoreDeferredDebt,
}

impl HistoricalAdmissionClass {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::RuntimeRetained => "runtime_retained",
            Self::RuntimeReplay => "runtime_replay",
            Self::RuntimeReconstruction => "runtime_reconstruction",
            Self::StoreDeferredDebt => "store_deferred_debt",
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum QueryContextDriftOutcome {
    BasisExact,
    ExplicitHistoricalDenial,
    ExplicitComparisonDenial,
}

impl QueryContextDriftOutcome {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::BasisExact => "basis_exact",
            Self::ExplicitHistoricalDenial => "explicit_historical_denial",
            Self::ExplicitComparisonDenial => "explicit_comparison_denial",
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum QueryContextAdmissionFailureClass {
    EvidenceIdentityDerivationDenied(IdentityDerivationDenial),
    UnsupportedHistoricalBasis,
    InvalidBasisPairing,
    PreviewProvenanceRequired,
    DiffScopeMismatch,
    AmbiguousComparisonBasis,
    StoreBackedHistoricalDeferred,
    BroadComparisonForbidden,
    ComparisonShapeMismatch,
    ComparisonBroadeningRequired,
    HistoricalPathTooBroadDenied,
    RawStorageDeltaLeakageForbidden,
    BasisSubstitutionForbidden,
    NonQueryOwnedHistoricalArtifact,
    UnsupportedHistoricalMaterializationPathClass,
    UnsupportedQueryContextBasisFamily,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct QueryContextAdmissionError {
    failure_class: QueryContextAdmissionFailureClass,
    message: &'static str,
    counters: QueryContextCounters,
}

impl QueryContextAdmissionError {
    pub(crate) fn identity_refused(
        cause: IdentityDerivationDenial,
        counters: QueryContextCounters,
    ) -> Self {
        Self::new(
            QueryContextAdmissionFailureClass::EvidenceIdentityDerivationDenied(cause),
            "query context evidence identity exceeded its digest bounds",
            counters,
        )
    }
    pub fn failure_class(&self) -> &QueryContextAdmissionFailureClass {
        &self.failure_class
    }

    pub fn message(&self) -> &'static str {
        self.message
    }

    pub fn counters(&self) -> &QueryContextCounters {
        &self.counters
    }

    pub(crate) fn new(
        failure_class: QueryContextAdmissionFailureClass,
        message: &'static str,
        counters: QueryContextCounters,
    ) -> Self {
        Self {
            failure_class,
            message,
            counters,
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct QueryBasisContextRequest {
    family: QueryContextFamily,
    declared_basis_label: String,
}

impl QueryBasisContextRequest {
    pub fn current_branch_head() -> Self {
        Self {
            family: QueryContextFamily::CurrentBranchHead,
            declared_basis_label: "current".to_string(),
        }
    }

    pub fn branch_head(branch_identity: impl Into<String>) -> Self {
        Self {
            family: QueryContextFamily::BranchHead,
            declared_basis_label: branch_identity.into(),
        }
    }

    pub fn historical_snapshot(basis_identity: impl Into<String>) -> Self {
        Self {
            family: QueryContextFamily::HistoricalSnapshot,
            declared_basis_label: basis_identity.into(),
        }
    }

    pub fn historical_commit(basis_identity: impl Into<String>) -> Self {
        Self {
            family: QueryContextFamily::HistoricalCommit,
            declared_basis_label: basis_identity.into(),
        }
    }

    pub fn preview_derived_historical(preview_identity: impl Into<String>) -> Self {
        Self {
            family: QueryContextFamily::PreviewDerivedHistorical,
            declared_basis_label: preview_identity.into(),
        }
    }

    pub fn family(&self) -> &QueryContextFamily {
        &self.family
    }

    pub fn declared_basis_label(&self) -> &str {
        &self.declared_basis_label
    }
}
