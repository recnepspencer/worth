use crate::diagnostics::BridgeHistoricalEvaluationRecordIdentity;
use crate::error::BridgeDeliveryErrorKind;
use crate::facade::{
    BridgeAspectRegistrationId, BridgeFailureClass, BridgePolicyFieldKind,
    BridgePolicyRejectionKind, BridgePreviewLifecycleStateKind, BridgeReplayErrorKind,
    BridgeRouteErrorKind, BridgeSignalBranchIdentity, BridgeWritebackErrorKind,
    BridgeWritebackFailureClass, BridgeWritebackFamilyKind, BridgeWritebackOutcomeClass,
    BridgeWritebackStrategyClass, FineGrainedMatchStatus, SubscriptionSliceKind,
    TruthBranchIdentity, TruthCommitIdentity, TruthDeltaSurfaceKind, TruthSnapshotIdentity,
};
use crate::identity::BridgeIdentity;
use crate::routing::{BridgeInvalidationIdentity, BridgeRouteIdentity};
use crate::snapshot::BridgeTruthViewSelectorIdentity;
use crate::speculation::BridgePreviewSessionIdentity;
use sha2::{Digest, Sha256};
use std::fmt;

mod certification_digest_basis;
mod certification_digests;
mod certification_evidence;
mod simulation_evidence;
mod terminal_report_export;

pub(super) use simulation_evidence::{
    PricingShockRankedMaterialDamageSet, PricingShockSimulationIterationTrace,
    PricingShockSimulationMaterialSummary, PricingShockSimulationSuite,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct PricingReferenceBundle {
    pub(super) source_branch: TruthBranchIdentity,
    pub(super) source_commit: TruthCommitIdentity,
    pub(super) route_snapshot: TruthSnapshotIdentity,
    pub(super) delivered_target_count: usize,
    pub(super) route_entry_count: usize,
    pub(super) evaluation_record_identity: BridgeHistoricalEvaluationRecordIdentity,
    pub(super) evaluation_selector_identity: BridgeTruthViewSelectorIdentity,
    pub(super) main_snapshot: TruthSnapshotIdentity,
    pub(super) main_rubber_cost_cents: i64,
    pub(super) speculative_truth_branch: TruthBranchIdentity,
    pub(super) speculative_signal_branch: BridgeSignalBranchIdentity,
    pub(super) speculative_snapshot: TruthSnapshotIdentity,
    pub(super) speculative_rubber_cost_cents: i64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct PricingAspectBundle {
    pub(super) route_identity: BridgeRouteIdentity,
    pub(super) snapshot: TruthSnapshotIdentity,
    pub(super) source_branch: TruthBranchIdentity,
    pub(super) source_commit: TruthCommitIdentity,
    pub(super) truth_surface_kind: TruthDeltaSurfaceKind,
    pub(super) fine_grained_match_status: FineGrainedMatchStatus,
    pub(super) aspect_registration_id: BridgeAspectRegistrationId,
    pub(super) subscription_slice_kind: SubscriptionSliceKind,
    pub(super) target_canonical_basis: String,
    pub(super) invalidation_target: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct PricingFailureBundle {
    pub(super) error_kind: BridgeDeliveryErrorKind,
    pub(super) failure_class: BridgeFailureClass,
    pub(super) source_commit: TruthCommitIdentity,
    pub(super) source_snapshot: TruthSnapshotIdentity,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct PricingReplayBundle {
    pub(super) source_commit: TruthCommitIdentity,
    pub(super) source_snapshot: TruthSnapshotIdentity,
    pub(super) route_identity: BridgeRouteIdentity,
    pub(super) invalidation_identity: BridgeInvalidationIdentity,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct PricingCertificationMatrix {
    pub(super) reference: PricingReferenceBundle,
    pub(super) replay: PricingReplayBundle,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct PricingDiscardBundle {
    pub(super) live_main_snapshot: TruthSnapshotIdentity,
    pub(super) speculative_rubber_cost_cents: i64,
    pub(super) post_discard_main_snapshot: TruthSnapshotIdentity,
    pub(super) post_discard_main_steel_cost_cents: i64,
    pub(super) lifecycle_state: BridgePreviewLifecycleStateKind,
    pub(super) discard_record_count: usize,
    pub(super) promotion_record_count: usize,
    pub(super) replay_outcome: BridgePreviewLifecycleStateKind,
    pub(super) has_discard_record: bool,
    pub(super) has_promotion_record: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct PricingPromotionBundle {
    pub(super) main_snapshot: TruthSnapshotIdentity,
    pub(super) speculative_snapshot: TruthSnapshotIdentity,
    pub(super) main_rubber_cost_cents: i64,
    pub(super) speculative_rubber_cost_cents: i64,
    pub(super) lifecycle_state: BridgePreviewLifecycleStateKind,
    pub(super) promotion_session_identity: BridgePreviewSessionIdentity,
    pub(super) authoritative_commit_boundary_digest: String,
    pub(super) authoritative_artifact_digest: String,
    pub(super) replay_outcome: BridgePreviewLifecycleStateKind,
    pub(super) has_promotion_explanation: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct PricingFanoutBundle {
    pub(super) total_deliveries: usize,
    pub(super) first_delivery_target_count: usize,
    pub(super) second_delivery_target_count: usize,
    pub(super) second_source_commit: TruthCommitIdentity,
    pub(super) second_snapshot: TruthSnapshotIdentity,
    pub(super) branch_snapshot: TruthSnapshotIdentity,
    pub(super) branch_steel_cost_cents: i64,
    pub(super) retained_target_count: usize,
    pub(super) first_target: String,
    pub(super) last_target: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct PricingRestartReplayBundle {
    pub(super) source_commit: TruthCommitIdentity,
    pub(super) source_snapshot: TruthSnapshotIdentity,
    pub(super) route_identity: BridgeRouteIdentity,
    pub(super) invalidation_identity: BridgeInvalidationIdentity,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct PricingRestartFailureBundle {
    pub(super) error_kind: BridgeReplayErrorKind,
    pub(super) replay_mismatch_count: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct PricingWritebackBundle {
    pub(super) family_kind: BridgeWritebackFamilyKind,
    pub(super) strategy_class: BridgeWritebackStrategyClass,
    pub(super) commit_outcome_class: BridgeWritebackOutcomeClass,
    pub(super) noop_outcome_class: BridgeWritebackOutcomeClass,
    pub(super) commit_replay_semantic_digest: String,
    pub(super) noop_replay_semantic_digest: String,
    pub(super) shared_authoritative_artifact: bool,
    pub(super) authority_commit_count: usize,
    pub(super) execution_request_count: usize,
    pub(super) execution_commit_count: usize,
    pub(super) execution_noop_count: usize,
    pub(super) rejection_error_kind: BridgeWritebackErrorKind,
    pub(super) rejection_failure_class: BridgeWritebackFailureClass,
    pub(super) rejection_request_emitted: bool,
    pub(super) rejection_receipt_emitted: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct PricingHistoricalProvenanceBundle {
    pub(super) main_commit: TruthCommitIdentity,
    pub(super) main_snapshot: TruthSnapshotIdentity,
    pub(super) main_regime: String,
    pub(super) main_external_factor_microunits: i64,
    pub(super) shock_commit: TruthCommitIdentity,
    pub(super) shock_snapshot: TruthSnapshotIdentity,
    pub(super) shock_regime: String,
    pub(super) shock_external_factor_microunits: i64,
    pub(super) shock_factor_delta_microunits: i64,
    pub(super) shock_trend_delta_microunits: i64,
    pub(super) shock_jump_delta_microunits: i64,
    pub(super) shock_delta_microunits: i64,
    pub(super) shock_multiplier_per_mille: i64,
    pub(super) representative_sku: String,
    pub(super) representative_retail_price_cents: i64,
    pub(super) representative_shipping_cost_cents: i64,
    pub(super) representative_fuel_shipping_component_cents: i64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct PricingPortfolioBlastRadiusBundle {
    pub(super) product_count: usize,
    pub(super) main_repricing_count: usize,
    pub(super) shock_repricing_count: usize,
    pub(super) main_margin_floor_breach_count: usize,
    pub(super) shock_margin_floor_breach_count: usize,
    pub(super) positive_retail_delta_count: usize,
    pub(super) total_retail_delta_cents: i64,
    pub(super) max_retail_delta_sku: String,
    pub(super) max_retail_delta_cents: i64,
    pub(super) top_margin_erosion_family: String,
    pub(super) top_margin_erosion_cents: i64,
    pub(super) most_shipping_sensitive_family: String,
    pub(super) most_shipping_sensitive_delta_cents: i64,
    pub(super) most_material_sensitive_family: String,
    pub(super) most_material_sensitive_delta_cents: i64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct PricingCrisisBundle {
    pub(super) crisis_name: String,
    pub(super) affected_product_count: usize,
    pub(super) main_total_retail_cents: i64,
    pub(super) crisis_total_retail_cents: i64,
    pub(super) total_retail_delta_cents: i64,
    pub(super) top_impacted_family: String,
    pub(super) top_impacted_family_delta_cents: i64,
    pub(super) dominant_shock_material: String,
    pub(super) dominant_shock_multiplier_per_mille: i64,
    pub(super) policy_pressure_family: String,
    pub(super) policy_pressure_bps: i64,
    pub(super) top_exposure_material: String,
    pub(super) top_exposure_material_delta_cents: i64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct PricingStrategyBundle {
    pub(super) hold_unprofitable_count: usize,
    pub(super) partial_absorb_unprofitable_count: usize,
    pub(super) targeted_reprice_positive_delta_count: usize,
    pub(super) targeted_reprice_total_delta_cents: i64,
    pub(super) hold_total_margin_delta_cents: i64,
    pub(super) partial_absorb_total_margin_delta_cents: i64,
    pub(super) targeted_reprice_margin_recovery_cents: i64,
    pub(super) recommended_strategy: String,
    pub(super) recommendation_reason: String,
    pub(super) promotion_strategy: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct PricingTrustAttackBundle {
    pub(super) replay_policy_error_kind: BridgePolicyRejectionKind,
    pub(super) replay_policy_failure_class: BridgePolicyFieldKind,
    pub(super) route_policy_error_kind: BridgeRouteErrorKind,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct PricingWorkloadCertificationBundle {
    pub(super) matrix: PricingCertificationMatrix,
    pub(super) aspect: PricingAspectBundle,
    pub(super) discard: PricingDiscardBundle,
    pub(super) promotion: PricingPromotionBundle,
    pub(super) fanout: PricingFanoutBundle,
    pub(super) restart_replay: PricingRestartReplayBundle,
    pub(super) restart_failure: PricingRestartFailureBundle,
    pub(super) writeback: PricingWritebackBundle,
    pub(super) provenance: PricingHistoricalProvenanceBundle,
    pub(super) portfolio: PricingPortfolioBlastRadiusBundle,
    pub(super) crisis: PricingCrisisBundle,
    pub(super) strategy: PricingStrategyBundle,
    pub(super) simulation: PricingShockSimulationSuite,
    pub(super) trust_attacks: PricingTrustAttackBundle,
    pub(super) hostile_failure: PricingFailureBundle,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(in crate::harness::tests) struct PricingCertificationBasisEntry {
    name: &'static str,
    value: String,
}

impl PricingCertificationBasisEntry {
    fn new(name: &'static str, value: impl PricingCertificationBasisValue) -> Self {
        Self {
            name,
            value: value.pricing_certification_basis_value(),
        }
    }

    fn debug(name: &'static str, value: impl fmt::Debug) -> Self {
        Self {
            name,
            value: format!("{value:?}"),
        }
    }

    fn canonical_line(&self) -> String {
        format!("{}={}", self.name, self.value)
    }
}

trait PricingCertificationBasisValue {
    fn pricing_certification_basis_value(self) -> String;
}

impl<T> PricingCertificationBasisValue for T
where
    T: fmt::Display,
{
    fn pricing_certification_basis_value(self) -> String {
        self.to_string()
    }
}

impl<Tag> PricingCertificationBasisValue for &BridgeIdentity<Tag> {
    fn pricing_certification_basis_value(self) -> String {
        format!("{self:?}")
    }
}

#[derive(Clone, Copy)]
pub(in crate::harness::tests) enum PricingCertificationDigestArtifact {
    Suite25Causality,
    Suite25Routing,
    Suite25Explanation,
    Suite25Replay,
    Suite25Discard,
    Suite25Promotion,
    Suite25Fanout,
    Suite25Writeback,
    Suite25HistoricalProvenance,
    Suite25ReferenceWorkload,
    RetainedWorkloadCertificationBundle,
    Suite26Failure,
    Suite26ReplayFailure,
    Suite26Diagnostics,
    Suite26ReferenceFailure,
    Suite27Certification,
}

impl PricingCertificationDigestArtifact {
    fn digest_domain(self) -> &'static str {
        match self {
            Self::Suite25Causality => "pricing-suite-25-causality",
            Self::Suite25Routing => "pricing-suite-25-routing",
            Self::Suite25Explanation => "pricing-suite-25-explanation",
            Self::Suite25Replay => "pricing-suite-25-replay",
            Self::Suite25Discard => "pricing-suite-25-discard",
            Self::Suite25Promotion => "pricing-suite-25-promotion",
            Self::Suite25Fanout => "pricing-suite-25-fanout",
            Self::Suite25Writeback => "pricing-suite-25-writeback",
            Self::Suite25HistoricalProvenance => "pricing-suite-25-historical-provenance",
            Self::Suite25ReferenceWorkload => "pricing-suite-25-reference",
            Self::RetainedWorkloadCertificationBundle => "pricing-workload-certification-bundle",
            Self::Suite26Failure => "pricing-suite-26-failure",
            Self::Suite26ReplayFailure => "pricing-suite-26-replay-failure",
            Self::Suite26Diagnostics => "pricing-suite-26-diagnostics",
            Self::Suite26ReferenceFailure => "pricing-suite-26-reference-failure",
            Self::Suite27Certification => "pricing-suite-27-certification",
        }
    }
}

fn derive_pricing_certification_digest_from_basis_entries(
    certification_artifact: PricingCertificationDigestArtifact,
    basis_entries: impl IntoIterator<Item = PricingCertificationBasisEntry>,
) -> String {
    let digest_domain = certification_artifact.digest_domain();
    let canonical_basis = format!(
        "{digest_domain}:{}",
        basis_entries
            .into_iter()
            .map(|entry| entry.canonical_line())
            .collect::<Vec<_>>()
            .join("|")
    );
    let digest = Sha256::digest(canonical_basis.as_bytes());
    format!("{digest:x}")
}
