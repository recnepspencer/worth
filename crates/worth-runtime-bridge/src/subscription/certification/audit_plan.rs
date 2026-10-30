use std::sync::Arc;

use sha2::{Digest, Sha256};

use super::{
    BridgeSubscriptionCertificationBundleSealed, BridgeSubscriptionCertificationComparisonReport,
    BridgeSubscriptionCertificationCounterSnapshot, BridgeSubscriptionOfflineAuditOutcomeSummary,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BridgeSubscriptionOfflineAuditBundleIndex {
    bundle_digests: Vec<Arc<str>>,
    counters: BridgeSubscriptionCertificationCounterSnapshot,
    canonical_basis: Arc<str>,
    digest: Arc<str>,
}

impl BridgeSubscriptionOfflineAuditBundleIndex {
    pub(crate) fn build(bundles: Vec<&BridgeSubscriptionCertificationBundleSealed>) -> Self {
        let mut bundle_digests = bundles
            .into_iter()
            .map(|bundle| Arc::<str>::from(bundle.digest()))
            .collect::<Vec<_>>();
        bundle_digests.sort();
        bundle_digests.dedup();
        let bundle_digest_basis = bundle_digests
            .iter()
            .map(Arc::as_ref)
            .collect::<Vec<_>>()
            .join(",");
        let canonical_basis = Arc::<str>::from(format!(
            "bridge-subscription-offline-audit-bundle-index|bundles={bundle_digest_basis}"
        ));
        let digest = Sha256::digest(canonical_basis.as_bytes());
        Self {
            counters:
                BridgeSubscriptionCertificationCounterSnapshot::from_offline_audit_bundle_index(
                    bundle_digests.len(),
                ),
            bundle_digests,
            canonical_basis,
            digest: Arc::from(format!(
                "bridge-subscription-offline-audit-bundle-index:sha256:{digest:x}"
            )),
        }
    }

    pub fn bundle_count(&self) -> usize {
        self.bundle_digests.len()
    }

    pub fn bundle_digests(&self) -> &[Arc<str>] {
        &self.bundle_digests
    }

    pub fn counters(&self) -> &BridgeSubscriptionCertificationCounterSnapshot {
        &self.counters
    }

    pub fn digest(&self) -> &str {
        self.digest.as_ref()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BridgeSubscriptionOfflineAuditPlanRejectionKind {
    EmptyBundleIndex,
    MissingComparisonReports,
    HostLogDependencyForbidden,
    LiveStateDependencyForbidden,
}

impl BridgeSubscriptionOfflineAuditPlanRejectionKind {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::EmptyBundleIndex => "empty_bundle_index",
            Self::MissingComparisonReports => "missing_comparison_reports",
            Self::HostLogDependencyForbidden => "host_log_dependency_forbidden",
            Self::LiveStateDependencyForbidden => "live_state_dependency_forbidden",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BridgeSubscriptionOfflineAuditPlanRejection {
    rejection_kind: BridgeSubscriptionOfflineAuditPlanRejectionKind,
    counters: BridgeSubscriptionCertificationCounterSnapshot,
    canonical_basis: Arc<str>,
    digest: Arc<str>,
}

impl BridgeSubscriptionOfflineAuditPlanRejection {
    fn new(
        rejection_kind: BridgeSubscriptionOfflineAuditPlanRejectionKind,
        counters: BridgeSubscriptionCertificationCounterSnapshot,
    ) -> Self {
        let canonical_basis = Arc::<str>::from(format!(
            "bridge-subscription-offline-audit-plan-rejection|kind={}|counters={}",
            rejection_kind.as_str(),
            counters.digest(),
        ));
        let digest = Sha256::digest(canonical_basis.as_bytes());
        Self {
            rejection_kind,
            counters,
            canonical_basis,
            digest: Arc::from(format!(
                "bridge-subscription-offline-audit-plan-rejection:sha256:{digest:x}"
            )),
        }
    }

    pub fn rejection_kind(&self) -> BridgeSubscriptionOfflineAuditPlanRejectionKind {
        self.rejection_kind
    }

    pub fn counters(&self) -> &BridgeSubscriptionCertificationCounterSnapshot {
        &self.counters
    }

    pub fn digest(&self) -> &str {
        self.digest.as_ref()
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BridgeSubscriptionOfflineAuditPlan {
    bundle_index_digest: Arc<str>,
    comparison_report_digests: Vec<Arc<str>>,
    outcome_summary: BridgeSubscriptionOfflineAuditOutcomeSummary,
    counters: BridgeSubscriptionCertificationCounterSnapshot,
    canonical_basis: Arc<str>,
    digest: Arc<str>,
}

impl BridgeSubscriptionOfflineAuditPlan {
    pub(crate) fn admit(
        bundle_index: &BridgeSubscriptionOfflineAuditBundleIndex,
        comparison_reports: Vec<&BridgeSubscriptionCertificationComparisonReport>,
        host_log_dependency_requested: bool,
        live_state_dependency_requested: bool,
    ) -> Result<Self, BridgeSubscriptionOfflineAuditPlanRejection> {
        if host_log_dependency_requested {
            return Err(BridgeSubscriptionOfflineAuditPlanRejection::new(
                BridgeSubscriptionOfflineAuditPlanRejectionKind::HostLogDependencyForbidden,
                BridgeSubscriptionCertificationCounterSnapshot::from_offline_audit_rejection(
                    true, false,
                ),
            ));
        }
        if live_state_dependency_requested {
            return Err(BridgeSubscriptionOfflineAuditPlanRejection::new(
                BridgeSubscriptionOfflineAuditPlanRejectionKind::LiveStateDependencyForbidden,
                BridgeSubscriptionCertificationCounterSnapshot::from_offline_audit_rejection(
                    false, true,
                ),
            ));
        }
        if bundle_index.bundle_count() == 0 {
            return Err(BridgeSubscriptionOfflineAuditPlanRejection::new(
                BridgeSubscriptionOfflineAuditPlanRejectionKind::EmptyBundleIndex,
                *bundle_index.counters(),
            ));
        }
        if comparison_reports.is_empty() {
            return Err(BridgeSubscriptionOfflineAuditPlanRejection::new(
                BridgeSubscriptionOfflineAuditPlanRejectionKind::MissingComparisonReports,
                *bundle_index.counters(),
            ));
        }
        let mut canonical_comparison_reports = comparison_reports;
        canonical_comparison_reports.sort_by(|left, right| left.digest().cmp(right.digest()));
        canonical_comparison_reports.dedup_by(|left, right| left.digest() == right.digest());
        let outcome_summary = BridgeSubscriptionOfflineAuditOutcomeSummary::from_comparison_reports(
            &canonical_comparison_reports,
        );
        let comparison_report_digests = canonical_comparison_reports
            .iter()
            .map(|report| Arc::<str>::from(report.digest()))
            .collect::<Vec<_>>();
        let counters = BridgeSubscriptionCertificationCounterSnapshot::combine([
            *bundle_index.counters(),
            BridgeSubscriptionCertificationCounterSnapshot::from_offline_audit_plan(
                comparison_report_digests.len(),
            ),
        ]);
        let comparison_digest_basis = comparison_report_digests
            .iter()
            .map(Arc::as_ref)
            .collect::<Vec<_>>()
            .join(",");
        let canonical_basis = Arc::<str>::from(format!(
            "bridge-subscription-offline-audit-plan|bundle-index={}|reports={comparison_digest_basis}|outcomes={}|counters={}",
            bundle_index.digest(),
            outcome_summary.digest(),
            counters.digest(),
        ));
        let digest = Sha256::digest(canonical_basis.as_bytes());
        Ok(Self {
            bundle_index_digest: Arc::from(bundle_index.digest()),
            comparison_report_digests,
            outcome_summary,
            counters,
            canonical_basis,
            digest: Arc::from(format!(
                "bridge-subscription-offline-audit-plan:sha256:{digest:x}"
            )),
        })
    }

    pub fn bundle_index_digest(&self) -> &str {
        self.bundle_index_digest.as_ref()
    }

    pub fn comparison_report_count(&self) -> usize {
        self.comparison_report_digests.len()
    }

    pub fn comparison_report_digests(&self) -> &[Arc<str>] {
        &self.comparison_report_digests
    }

    pub fn outcome_summary(&self) -> &BridgeSubscriptionOfflineAuditOutcomeSummary {
        &self.outcome_summary
    }

    pub fn counters(&self) -> &BridgeSubscriptionCertificationCounterSnapshot {
        &self.counters
    }

    pub fn digest(&self) -> &str {
        self.digest.as_ref()
    }
}
