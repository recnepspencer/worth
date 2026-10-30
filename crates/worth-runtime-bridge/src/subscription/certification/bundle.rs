use std::sync::Arc;

use sha2::{Digest, Sha256};

use super::{
    BridgeSubscriptionBundleField, BridgeSubscriptionBundleFieldState,
    BridgeSubscriptionCertificationAssemblyPlan, BridgeSubscriptionCertificationAssemblyRejection,
    BridgeSubscriptionCertificationAssemblyRejectionKind,
    BridgeSubscriptionCertificationCompletenessReport, BridgeSubscriptionCertificationCostProfile,
    BridgeSubscriptionCertificationCounterSnapshot, BridgeSubscriptionCertificationScratch,
    BridgeSubscriptionCertificationSemanticDigests,
    BridgeSubscriptionCertificationSemanticSourceKind,
};

pub const BRIDGE_SUBSCRIPTION_CERTIFICATION_BUNDLE_SCHEMA_V1: &str =
    "bridge-subscription-certification-bundle-v1";
pub const BRIDGE_SUBSCRIPTION_CERTIFICATION_DIGEST_ALGORITHM_V1: &str = "sha256";

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BridgeSubscriptionCertificationBundleDraft {
    schema_version: Arc<str>,
    digest_algorithm: Arc<str>,
    assembly_plan_digest: Arc<str>,
    cost_profile_digest: Arc<str>,
    scratch_digest: Arc<str>,
    fields: Vec<BridgeSubscriptionBundleField>,
    semantic_digests: BridgeSubscriptionCertificationSemanticDigests,
    completeness_report: BridgeSubscriptionCertificationCompletenessReport,
    counters: BridgeSubscriptionCertificationCounterSnapshot,
    canonical_basis: Arc<str>,
    digest: Arc<str>,
}

impl BridgeSubscriptionCertificationBundleDraft {
    pub(crate) fn assemble(
        assembly_plan: BridgeSubscriptionCertificationAssemblyPlan,
        cost_profile: BridgeSubscriptionCertificationCostProfile,
        scratch: BridgeSubscriptionCertificationScratch,
    ) -> Result<Self, BridgeSubscriptionCertificationAssemblyRejection> {
        let counters = BridgeSubscriptionCertificationCounterSnapshot::combine([
            *assembly_plan.counters(),
            *cost_profile.counters(),
            *scratch.counters(),
            BridgeSubscriptionCertificationCounterSnapshot::from_bundle(),
        ]);
        Self::admit_source_artifact_budget(&assembly_plan, &cost_profile, &scratch, counters)?;
        Self::admit_bundle_field_budget(&assembly_plan, &cost_profile, &scratch, counters)?;
        Self::admit_scratch_capacity(&assembly_plan, &cost_profile, &scratch, counters)?;
        let semantic_digests = BridgeSubscriptionCertificationSemanticDigests::from_assembly_parts(
            &assembly_plan,
            &cost_profile,
            &scratch,
            &counters,
        );
        let preview_records_state = if assembly_plan
            .semantic_source_digests()
            .source_digest_for(BridgeSubscriptionCertificationSemanticSourceKind::Residue)
            .source_present()
        {
            BridgeSubscriptionBundleFieldState::Present
        } else {
            BridgeSubscriptionBundleFieldState::NotExercised
        };
        let mut fields = vec![
            BridgeSubscriptionBundleField::new(
                "bundle_header",
                BridgeSubscriptionBundleFieldState::Present,
                format!(
                    "{}:{}",
                    assembly_plan.bundle_schema_version(),
                    assembly_plan.bundle_digest_algorithm()
                ),
            ),
            BridgeSubscriptionBundleField::new(
                "source_artifact_index",
                BridgeSubscriptionBundleFieldState::Present,
                assembly_plan.source_artifact_index_digest(),
            ),
            BridgeSubscriptionBundleField::new(
                "manifest",
                BridgeSubscriptionBundleFieldState::Present,
                assembly_plan.manifest_digest(),
            ),
            BridgeSubscriptionBundleField::new(
                "preview_records",
                preview_records_state,
                semantic_digests.residue_digest(),
            ),
            BridgeSubscriptionBundleField::new(
                "comparison_inputs",
                BridgeSubscriptionBundleFieldState::Present,
                semantic_digests.digest(),
            ),
            BridgeSubscriptionBundleField::new(
                "semantic_digests",
                BridgeSubscriptionBundleFieldState::Present,
                assembly_plan.digest(),
            ),
            BridgeSubscriptionBundleField::new(
                "counter_snapshot",
                BridgeSubscriptionBundleFieldState::Present,
                counters.digest(),
            ),
            BridgeSubscriptionBundleField::new(
                "completeness_report",
                BridgeSubscriptionBundleFieldState::Present,
                "bundle-completeness-report",
            ),
        ];
        fields.sort_by(|left, right| left.field_name().cmp(right.field_name()));
        let completeness_report = BridgeSubscriptionCertificationCompletenessReport::from_fields(
            assembly_plan.expected_field_count(),
            &fields,
        );
        let field_digests = fields
            .iter()
            .map(BridgeSubscriptionBundleField::digest)
            .collect::<Vec<_>>()
            .join(",");
        let canonical_basis = Arc::<str>::from(format!(
            "bridge-subscription-certification-bundle-draft|schema={}|algorithm={}|plan={}|cost-profile={}|scratch={}|semantic-digests={}|completeness={}|counters={}|fields={field_digests}",
            assembly_plan.bundle_schema_version(),
            assembly_plan.bundle_digest_algorithm(),
            assembly_plan.digest(),
            cost_profile.digest(),
            scratch.digest(),
            semantic_digests.digest(),
            completeness_report.digest(),
            counters.digest(),
        ));
        let digest = Sha256::digest(canonical_basis.as_bytes());
        Ok(Self {
            schema_version: Arc::from(assembly_plan.bundle_schema_version()),
            digest_algorithm: Arc::from(assembly_plan.bundle_digest_algorithm()),
            assembly_plan_digest: Arc::from(assembly_plan.digest()),
            cost_profile_digest: Arc::from(cost_profile.digest()),
            scratch_digest: Arc::from(scratch.digest()),
            fields,
            semantic_digests,
            completeness_report,
            counters,
            canonical_basis,
            digest: Arc::from(format!(
                "bridge-subscription-certification-bundle-draft:sha256:{digest:x}"
            )),
        })
    }

    fn admit_source_artifact_budget(
        assembly_plan: &BridgeSubscriptionCertificationAssemblyPlan,
        cost_profile: &BridgeSubscriptionCertificationCostProfile,
        scratch: &BridgeSubscriptionCertificationScratch,
        counters: BridgeSubscriptionCertificationCounterSnapshot,
    ) -> Result<(), BridgeSubscriptionCertificationAssemblyRejection> {
        if assembly_plan.selected_record_count() <= cost_profile.max_source_artifact_entries() {
            return Ok(());
        }
        Err(Self::assembly_rejection(
            BridgeSubscriptionCertificationAssemblyRejectionKind::SourceArtifactBudgetExceeded,
            assembly_plan,
            cost_profile,
            scratch,
            counters,
        ))
    }

    fn admit_bundle_field_budget(
        assembly_plan: &BridgeSubscriptionCertificationAssemblyPlan,
        cost_profile: &BridgeSubscriptionCertificationCostProfile,
        scratch: &BridgeSubscriptionCertificationScratch,
        counters: BridgeSubscriptionCertificationCounterSnapshot,
    ) -> Result<(), BridgeSubscriptionCertificationAssemblyRejection> {
        if assembly_plan.expected_field_count() <= cost_profile.max_bundle_field_count() {
            return Ok(());
        }
        Err(Self::assembly_rejection(
            BridgeSubscriptionCertificationAssemblyRejectionKind::BundleFieldBudgetExceeded,
            assembly_plan,
            cost_profile,
            scratch,
            counters,
        ))
    }

    fn admit_scratch_capacity(
        assembly_plan: &BridgeSubscriptionCertificationAssemblyPlan,
        cost_profile: &BridgeSubscriptionCertificationCostProfile,
        scratch: &BridgeSubscriptionCertificationScratch,
        counters: BridgeSubscriptionCertificationCounterSnapshot,
    ) -> Result<(), BridgeSubscriptionCertificationAssemblyRejection> {
        if scratch.scratch_capacity() >= assembly_plan.expected_field_count() {
            return Ok(());
        }
        Err(Self::assembly_rejection(
            BridgeSubscriptionCertificationAssemblyRejectionKind::ScratchCapacityTooSmall,
            assembly_plan,
            cost_profile,
            scratch,
            counters,
        ))
    }

    fn assembly_rejection(
        rejection_kind: BridgeSubscriptionCertificationAssemblyRejectionKind,
        assembly_plan: &BridgeSubscriptionCertificationAssemblyPlan,
        cost_profile: &BridgeSubscriptionCertificationCostProfile,
        scratch: &BridgeSubscriptionCertificationScratch,
        counters: BridgeSubscriptionCertificationCounterSnapshot,
    ) -> BridgeSubscriptionCertificationAssemblyRejection {
        BridgeSubscriptionCertificationAssemblyRejection::new(
            rejection_kind,
            assembly_plan.digest(),
            cost_profile.digest(),
            scratch.digest(),
            counters,
        )
    }

    pub(crate) fn seal(self) -> BridgeSubscriptionCertificationBundleSealed {
        let canonical_basis = Arc::<str>::from(format!(
            "bridge-subscription-certification-bundle-sealed|draft={}",
            self.digest
        ));
        let digest = Sha256::digest(canonical_basis.as_bytes());
        BridgeSubscriptionCertificationBundleSealed {
            schema_version: self.schema_version,
            digest_algorithm: self.digest_algorithm,
            assembly_plan_digest: self.assembly_plan_digest,
            cost_profile_digest: self.cost_profile_digest,
            scratch_digest: self.scratch_digest,
            fields: self.fields,
            semantic_digests: self.semantic_digests,
            completeness_report: self.completeness_report,
            counters: self.counters,
            canonical_basis,
            digest: Arc::from(format!(
                "bridge-subscription-certification-bundle-sealed:sha256:{digest:x}"
            )),
        }
    }

    pub fn fields(&self) -> &[BridgeSubscriptionBundleField] {
        &self.fields
    }

    pub fn counters(&self) -> &BridgeSubscriptionCertificationCounterSnapshot {
        &self.counters
    }

    pub fn semantic_digests(&self) -> &BridgeSubscriptionCertificationSemanticDigests {
        &self.semantic_digests
    }

    pub fn completeness_report(&self) -> &BridgeSubscriptionCertificationCompletenessReport {
        &self.completeness_report
    }

    pub fn digest(&self) -> &str {
        self.digest.as_ref()
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BridgeSubscriptionCertificationBundleSealed {
    schema_version: Arc<str>,
    digest_algorithm: Arc<str>,
    assembly_plan_digest: Arc<str>,
    cost_profile_digest: Arc<str>,
    scratch_digest: Arc<str>,
    fields: Vec<BridgeSubscriptionBundleField>,
    semantic_digests: BridgeSubscriptionCertificationSemanticDigests,
    completeness_report: BridgeSubscriptionCertificationCompletenessReport,
    counters: BridgeSubscriptionCertificationCounterSnapshot,
    canonical_basis: Arc<str>,
    digest: Arc<str>,
}

impl BridgeSubscriptionCertificationBundleSealed {
    pub fn schema_version(&self) -> &str {
        self.schema_version.as_ref()
    }

    pub fn digest_algorithm(&self) -> &str {
        self.digest_algorithm.as_ref()
    }

    pub fn fields(&self) -> &[BridgeSubscriptionBundleField] {
        &self.fields
    }

    pub fn counters(&self) -> &BridgeSubscriptionCertificationCounterSnapshot {
        &self.counters
    }

    pub fn semantic_digests(&self) -> &BridgeSubscriptionCertificationSemanticDigests {
        &self.semantic_digests
    }

    pub fn completeness_report(&self) -> &BridgeSubscriptionCertificationCompletenessReport {
        &self.completeness_report
    }

    pub fn assembly_plan_digest(&self) -> &str {
        self.assembly_plan_digest.as_ref()
    }

    pub fn cost_profile_digest(&self) -> &str {
        self.cost_profile_digest.as_ref()
    }

    pub fn scratch_digest(&self) -> &str {
        self.scratch_digest.as_ref()
    }

    pub fn digest(&self) -> &str {
        self.digest.as_ref()
    }
}
