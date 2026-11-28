use super::{PricingCertificationDigestArtifact, PricingWorkloadCertificationBundle};

#[derive(Debug, Clone, PartialEq, Eq)]
pub(in crate::harness::tests) struct PricingSuite25DigestEvidence {
    pub(in crate::harness::tests) causality_digest: String,
    pub(in crate::harness::tests) routing_digest: String,
    pub(in crate::harness::tests) explanation_digest: String,
    pub(in crate::harness::tests) replay_digest: String,
    pub(in crate::harness::tests) discard_digest: String,
    pub(in crate::harness::tests) promotion_digest: String,
    pub(in crate::harness::tests) fanout_digest: String,
    pub(in crate::harness::tests) writeback_digest: String,
    pub(in crate::harness::tests) historical_provenance_digest: String,
    pub(in crate::harness::tests) reference_workload_bundle_digest: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(in crate::harness::tests) struct PricingSuite26DigestEvidence {
    pub(in crate::harness::tests) failure_digest: String,
    pub(in crate::harness::tests) replay_failure_digest: String,
    pub(in crate::harness::tests) diagnostics_digest: String,
    pub(in crate::harness::tests) reference_workload_failure_bundle_digest: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(in crate::harness::tests) struct PricingSuite27DigestEvidence {
    pub(in crate::harness::tests) certification_bundle_digest: String,
    pub(in crate::harness::tests) reference_workload_bundle_digest: String,
}

impl PricingWorkloadCertificationBundle {
    pub(in crate::harness::tests) fn suite_25_digest_evidence(
        &self,
    ) -> PricingSuite25DigestEvidence {
        PricingSuite25DigestEvidence {
            causality_digest: super::derive_pricing_certification_digest_from_basis_entries(
                PricingCertificationDigestArtifact::Suite25Causality,
                self.suite_25_causality_basis_entries(),
            ),
            routing_digest: super::derive_pricing_certification_digest_from_basis_entries(
                PricingCertificationDigestArtifact::Suite25Routing,
                self.suite_25_routing_basis_entries(),
            ),
            explanation_digest: super::derive_pricing_certification_digest_from_basis_entries(
                PricingCertificationDigestArtifact::Suite25Explanation,
                self.suite_25_explanation_basis_entries(),
            ),
            replay_digest: super::derive_pricing_certification_digest_from_basis_entries(
                PricingCertificationDigestArtifact::Suite25Replay,
                self.suite_25_replay_basis_entries(),
            ),
            discard_digest: super::derive_pricing_certification_digest_from_basis_entries(
                PricingCertificationDigestArtifact::Suite25Discard,
                self.suite_25_discard_basis_entries(),
            ),
            promotion_digest: super::derive_pricing_certification_digest_from_basis_entries(
                PricingCertificationDigestArtifact::Suite25Promotion,
                self.suite_25_promotion_basis_entries(),
            ),
            fanout_digest: super::derive_pricing_certification_digest_from_basis_entries(
                PricingCertificationDigestArtifact::Suite25Fanout,
                self.suite_25_fanout_basis_entries(),
            ),
            writeback_digest: super::derive_pricing_certification_digest_from_basis_entries(
                PricingCertificationDigestArtifact::Suite25Writeback,
                self.suite_25_writeback_basis_entries(),
            ),
            historical_provenance_digest:
                super::derive_pricing_certification_digest_from_basis_entries(
                    PricingCertificationDigestArtifact::Suite25HistoricalProvenance,
                    self.suite_25_historical_provenance_basis_entries(),
                ),
            reference_workload_bundle_digest:
                super::derive_pricing_certification_digest_from_basis_entries(
                    PricingCertificationDigestArtifact::Suite25ReferenceWorkload,
                    self.reference_workload_basis_entries(),
                ),
        }
    }

    pub(in crate::harness::tests) fn retained_bundle_digest(&self) -> String {
        super::derive_pricing_certification_digest_from_basis_entries(
            PricingCertificationDigestArtifact::RetainedWorkloadCertificationBundle,
            self.core_summary_basis_entries(),
        )
    }

    pub(in crate::harness::tests) fn suite_26_digest_evidence(
        &self,
    ) -> PricingSuite26DigestEvidence {
        PricingSuite26DigestEvidence {
            failure_digest: super::derive_pricing_certification_digest_from_basis_entries(
                PricingCertificationDigestArtifact::Suite26Failure,
                self.failure_localization_basis_entries(),
            ),
            replay_failure_digest: super::derive_pricing_certification_digest_from_basis_entries(
                PricingCertificationDigestArtifact::Suite26ReplayFailure,
                self.replay_failure_basis_entries(),
            ),
            diagnostics_digest: super::derive_pricing_certification_digest_from_basis_entries(
                PricingCertificationDigestArtifact::Suite26Diagnostics,
                self.core_summary_basis_entries(),
            ),
            reference_workload_failure_bundle_digest:
                super::derive_pricing_certification_digest_from_basis_entries(
                    PricingCertificationDigestArtifact::Suite26ReferenceFailure,
                    self.reference_workload_failure_basis_entries(),
                ),
        }
    }

    pub(in crate::harness::tests) fn suite_27_digest_evidence(
        &self,
    ) -> PricingSuite27DigestEvidence {
        let suite_25 = self.suite_25_digest_evidence();
        PricingSuite27DigestEvidence {
            certification_bundle_digest:
                super::derive_pricing_certification_digest_from_basis_entries(
                    PricingCertificationDigestArtifact::Suite27Certification,
                    self.core_summary_basis_entries(),
                ),
            reference_workload_bundle_digest: suite_25.reference_workload_bundle_digest,
        }
    }
}
