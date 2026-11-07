use crate::evidence_identity::WorthQueryEvidenceIdentityDerivationDenial as IdentityDerivationDenial;
use crate::{WorthQueryEvidenceIdentity, WorthQueryEvidenceScope, WorthQueryEvidenceTag};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum QuerySubscriptionRuntimeCertificationErrorKind {
    EvidenceIdentityDerivationDenied(IdentityDerivationDenial),
    CoverageResolutionDenied,
    CoverageFamilyMissing,
    CoverageFamilyMismatch,
    CertificationSupportClassDenied,
    CertificationSupportPostureDenied,
    ScopeFamilyMismatch,
    ScopeSourceMismatch,
    CoverageScopeMissingAdmittedRow,
    MissingHostileCoverage,
    UncoveredFamily,
}

impl QuerySubscriptionRuntimeCertificationErrorKind {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::EvidenceIdentityDerivationDenied(_) => "evidence_identity_derivation_denied",
            Self::CoverageResolutionDenied => "coverage_resolution_denied",
            Self::CoverageFamilyMissing => "coverage_family_missing",
            Self::CoverageFamilyMismatch => "coverage_family_mismatch",
            Self::CertificationSupportClassDenied => "certification_support_class_denied",
            Self::CertificationSupportPostureDenied => "certification_support_posture_denied",
            Self::ScopeFamilyMismatch => "scope_family_mismatch",
            Self::ScopeSourceMismatch => "scope_source_mismatch",
            Self::CoverageScopeMissingAdmittedRow => "coverage_scope_missing_admitted_row",
            Self::MissingHostileCoverage => "missing_hostile_coverage",
            Self::UncoveredFamily => "uncovered_family",
        }
    }
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct QuerySubscriptionRuntimeCertificationCounters {
    certification_scope_emission_count: u64,
    certified_family_count: u64,
    hostile_row_coverage_count: u64,
    uncovered_family_denial_count: u64,
    family_coverage_index_lookup_count: u64,
    family_coverage_matrix_scan_debt_count: u64,
}

impl QuerySubscriptionRuntimeCertificationCounters {
    pub fn counter_identity(&self) -> WorthQueryEvidenceIdentity {
        WorthQueryEvidenceIdentity::compose(WorthQueryEvidenceScope::SubscriptionActivationReceipt)
            .field_shape(
                WorthQueryEvidenceTag::new("identity_family"),
                "query_subscription_runtime_certification_counters_v1",
            )
            .field_value(
                WorthQueryEvidenceTag::new("scope_emission"),
                self.certification_scope_emission_count.to_string(),
            )
            .field_value(
                WorthQueryEvidenceTag::new("certified_family"),
                self.certified_family_count.to_string(),
            )
            .field_value(
                WorthQueryEvidenceTag::new("hostile_row_coverage"),
                self.hostile_row_coverage_count.to_string(),
            )
            .field_value(
                WorthQueryEvidenceTag::new("uncovered_family_denial"),
                self.uncovered_family_denial_count.to_string(),
            )
            .field_value(
                WorthQueryEvidenceTag::new("family_coverage_index_lookup"),
                self.family_coverage_index_lookup_count.to_string(),
            )
            .field_value(
                WorthQueryEvidenceTag::new("family_coverage_matrix_scan_debt"),
                self.family_coverage_matrix_scan_debt_count.to_string(),
            )
            .seal()
            .expect("fixed counter grammar contains only primitive values and literals")
    }

    pub fn certification_scope_emission_count(&self) -> u64 {
        self.certification_scope_emission_count
    }

    pub fn certified_family_count(&self) -> u64 {
        self.certified_family_count
    }

    pub fn hostile_row_coverage_count(&self) -> u64 {
        self.hostile_row_coverage_count
    }

    pub fn uncovered_family_denial_count(&self) -> u64 {
        self.uncovered_family_denial_count
    }

    pub fn family_coverage_index_lookup_count(&self) -> u64 {
        self.family_coverage_index_lookup_count
    }

    pub fn family_coverage_matrix_scan_debt_count(&self) -> u64 {
        self.family_coverage_matrix_scan_debt_count
    }

    pub(crate) fn scope_emitted() -> Self {
        Self {
            certification_scope_emission_count: 1,
            ..Default::default()
        }
    }

    pub(crate) fn certified(
        hostile_row_count: usize,
        posture: crate::subscription::runtime_certification::CoverageResolutionPosture,
    ) -> Self {
        Self {
            certified_family_count: 1,
            hostile_row_coverage_count: hostile_row_count as u64,
            family_coverage_index_lookup_count: u64::from(
                posture
                    == crate::subscription::runtime_certification::CoverageResolutionPosture::IndexedCoverageSet,
            ),
            family_coverage_matrix_scan_debt_count: u64::from(
                posture
                    == crate::subscription::runtime_certification::CoverageResolutionPosture::MatrixScanDebtExplicit,
            ),
            ..Default::default()
        }
    }

    pub(crate) fn uncovered_family(posture_requires_scan_debt: bool) -> Self {
        Self {
            uncovered_family_denial_count: 1,
            family_coverage_index_lookup_count: 1,
            family_coverage_matrix_scan_debt_count: u64::from(posture_requires_scan_debt),
            ..Default::default()
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct QuerySubscriptionRuntimeCertificationError {
    error_kind: QuerySubscriptionRuntimeCertificationErrorKind,
    message: &'static str,
    pub(in crate::subscription) failure_identity:
        Result<WorthQueryEvidenceIdentity, IdentityDerivationDenial>,
    counters: QuerySubscriptionRuntimeCertificationCounters,
}

impl QuerySubscriptionRuntimeCertificationError {
    pub(in crate::subscription) fn identity_refused(
        cause: IdentityDerivationDenial,
        counters: QuerySubscriptionRuntimeCertificationCounters,
    ) -> Self {
        Self {
            error_kind:
                QuerySubscriptionRuntimeCertificationErrorKind::EvidenceIdentityDerivationDenied(
                    cause,
                ),
            message: "runtime certification evidence identity exceeded its digest bounds",
            failure_identity: Err(cause),
            counters,
        }
    }

    pub(crate) fn new(
        error_kind: QuerySubscriptionRuntimeCertificationErrorKind,
        message: &'static str,
        evidence: Result<Vec<WorthQueryEvidenceIdentity>, IdentityDerivationDenial>,
        counters: QuerySubscriptionRuntimeCertificationCounters,
    ) -> Self {
        let failure_identity = (|| {
            let evidence = evidence?;
            Ok(WorthQueryEvidenceIdentity::compose(
                WorthQueryEvidenceScope::SubscriptionActivationReceipt,
            )
            .field_shape(
                WorthQueryEvidenceTag::new("identity_family"),
                "query_subscription_runtime_certification_error_v1",
            )
            .field_shape(WorthQueryEvidenceTag::new("kind"), error_kind.as_str())
            .field_value(WorthQueryEvidenceTag::new("message"), message)
            .field_evidence_identity_sequence(
                WorthQueryEvidenceTag::new("evidence"),
                evidence.iter(),
            )
            .seal()?)
        })();
        Self {
            error_kind,
            message,
            failure_identity,
            counters,
        }
    }

    pub fn error_kind(&self) -> &QuerySubscriptionRuntimeCertificationErrorKind {
        &self.error_kind
    }

    pub fn message(&self) -> &str {
        self.message
    }

    pub fn counters(&self) -> &QuerySubscriptionRuntimeCertificationCounters {
        &self.counters
    }
}
