mod builders;
mod evidence;
mod lane_fixtures;
mod lanes;
mod rows;
#[cfg(test)]
mod tests;

use crate::harness::certification::{digest_parts, CanonicalCertificationRow, CertificationMatrix};

pub const MILESTONE_NINE_THREE_REQUIRED_CANONICAL_ROW_NAMES: &[&str] = &[
    "detail-family-support-and-parity",
    "inspector-family-support-and-parity",
    "ordered-collection-family-support-and-parity",
    "grouped-collection-family-support-and-parity",
    "bounded-materialization-family-support-and-parity",
    "preview-family-lifecycle-certification-bundle",
    "continuation-family-support-sync",
    "family-coverage-certification-closure",
    "declaration-family-drift-vs-lifecycle-churn-distinctness",
    "basis-policy-viewshape-family-coverage-closure",
    "support-matrix-scale-honesty",
];

pub const MILESTONE_NINE_THREE_REQUIRED_REJECTION_ROW_NAMES: &[&str] = &[
    "uncertified-family-support-overclaim-forbidden",
    "store-backed-restart-support-overclaim-forbidden",
    "durable-replay-support-overclaim-forbidden",
    "bridge-parity-declaration-source-mismatch",
    "bridge-parity-signal-strategy-source-mismatch",
    "diagnostic-bundle-missing-hostile-row-forbidden",
    "runtime-certification-cross-family-row-mix-forbidden",
    "generic-family-certification-shortcut-forbidden",
];

pub const MILESTONE_NINE_THREE_REQUIRED_COMPILE_FAIL_TARGETS: &[&str] = &[
    "subscription_support_report_constructor_private.rs",
    "subscription_bridge_parity_explanation_constructor_private.rs",
    "subscription_runtime_certification_scope_constructor_private.rs",
    "subscription_diagnostic_bundle_constructor_private.rs",
    "subscription_runtime_certification_bundle_constructor_private.rs",
    "subscription_support_report_durable_overclaim_forbidden.rs",
    "subscription_bridge_parity_mismatched_declaration_forbidden.rs",
    "subscription_bridge_parity_mismatched_signal_strategy_forbidden.rs",
    "subscription_diagnostic_bundle_missing_hostile_coverage_forbidden.rs",
    "subscription_runtime_certification_uncertified_family_forbidden.rs",
];

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub enum MilestoneNineThreePerturbationClass {
    DetailFamilySupportAndParity,
    InspectorFamilySupportAndParity,
    OrderedCollectionFamilySupportAndParity,
    GroupedCollectionFamilySupportAndParity,
    BoundedMaterializationFamilySupportAndParity,
    PreviewFamilyLifecycleCertificationBundle,
    ContinuationFamilySupportSync,
    FamilyCoverageCertificationClosure,
    DeclarationFamilyDriftVsLifecycleChurnDistinctness,
    BasisPolicyViewshapeFamilyCoverageClosure,
    SupportMatrixScaleHonesty,
    UncertifiedFamilySupportOverclaimForbidden,
    StoreBackedRestartSupportOverclaimForbidden,
    DurableReplaySupportOverclaimForbidden,
    BridgeParityDeclarationSourceMismatch,
    BridgeParitySignalStrategySourceMismatch,
    DiagnosticBundleMissingHostileRowForbidden,
    RuntimeCertificationCrossFamilyRowMixForbidden,
    GenericFamilyCertificationShortcutForbidden,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum MilestoneNineThreeFailureClass {
    SupportDenied,
    BridgeParityDenied,
    RuntimeCertificationDenied,
    CompileFailBoundary,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MilestoneNineThreeCertificationBundle {
    pub query_family_label: String,
    pub declaration_family_label: String,
    pub bridge_family_label: String,
    pub support_class_label: String,
    pub support_resolution_posture_label: String,
    pub coverage_resolution_posture_label: String,
    pub query_digest: String,
    pub subscription_family_digest: String,
    pub subscription_declaration_digest: String,
    pub subscription_equivalence_digest: String,
    pub bridge_declaration_digest: String,
    pub bridge_basis_digest: String,
    pub signal_strategy_digest: String,
    pub support_report_digest: String,
    pub support_matrix_digest: String,
    pub support_lookup_receipt_digest: String,
    pub manual_bridge_witness_digest: String,
    pub bridge_parity_digest: String,
    pub bridge_parity_receipt_digest: String,
    pub diagnostic_trace_digest: String,
    pub admitted_diagnostic_bundle_digest: String,
    pub denied_diagnostic_bundle_digest: String,
    pub diagnostic_assembly_receipt_digest: String,
    pub lifecycle_certification_digest: String,
    pub runtime_certification_bundle_digest: String,
    pub certification_coverage_receipt_digest: String,
    pub continuation_digest: String,
    pub preview_isolation_digest: String,
    pub failure_digest: String,
    pub counter_snapshot: String,
    pub compile_fail_boundary_digest: String,
}

impl MilestoneNineThreeCertificationBundle {
    pub(super) fn semantic_signature(&self) -> String {
        digest_parts(&[
            format!("query_family:{}", self.query_family_label),
            format!("declaration_family:{}", self.declaration_family_label),
            format!("bridge_family:{}", self.bridge_family_label),
            format!("support_class:{}", self.support_class_label),
            format!("support_posture:{}", self.support_resolution_posture_label),
            format!(
                "coverage_posture:{}",
                self.coverage_resolution_posture_label
            ),
            format!("query:{}", self.query_digest),
            format!("family:{}", self.subscription_family_digest),
            format!("declaration:{}", self.subscription_declaration_digest),
            format!("equivalence:{}", self.subscription_equivalence_digest),
            format!("bridge:{}", self.bridge_declaration_digest),
            format!("basis:{}", self.bridge_basis_digest),
            format!("signal:{}", self.signal_strategy_digest),
            format!("support:{}", self.support_report_digest),
            format!("support_matrix:{}", self.support_matrix_digest),
            format!("support_lookup:{}", self.support_lookup_receipt_digest),
            format!("witness:{}", self.manual_bridge_witness_digest),
            format!("parity:{}", self.bridge_parity_digest),
            format!("parity_receipt:{}", self.bridge_parity_receipt_digest),
            format!("trace:{}", self.diagnostic_trace_digest),
            format!("admitted_bundle:{}", self.admitted_diagnostic_bundle_digest),
            format!(
                "diagnostic_receipt:{}",
                self.diagnostic_assembly_receipt_digest
            ),
            format!("lifecycle:{}", self.lifecycle_certification_digest),
            format!("bundle:{}", self.runtime_certification_bundle_digest),
            format!(
                "coverage_receipt:{}",
                self.certification_coverage_receipt_digest
            ),
            format!("continuation:{}", self.continuation_digest),
            format!("preview:{}", self.preview_isolation_digest),
        ])
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MilestoneNineThreeRejectionBundle {
    pub failure_class: MilestoneNineThreeFailureClass,
    pub failure_kind: String,
    pub failure_digest: String,
    pub denied_bundle_digest: String,
    pub counter_snapshot: String,
    pub compile_fail_boundary_digest: String,
}

pub type MilestoneNineThreeCertificationRow = CanonicalCertificationRow<
    MilestoneNineThreePerturbationClass,
    MilestoneNineThreeCertificationBundle,
>;
pub type MilestoneNineThreeCertificationMatrix = CertificationMatrix<
    MilestoneNineThreePerturbationClass,
    MilestoneNineThreeCertificationBundle,
    MilestoneNineThreeRejectionBundle,
>;

pub struct MilestoneNineThreeCertificationAdapter;

impl MilestoneNineThreeCertificationAdapter {
    pub fn query_subscription_bridge_parity_and_diagnostic_sufficiency_test(
    ) -> MilestoneNineThreeCertificationMatrix {
        MilestoneNineThreeCertificationMatrix {
            suite_name: "Query Subscription Bridge Parity And Diagnostic Sufficiency Test",
            rows: builders::canonical_rows(),
            rejection_rows: builders::rejection_rows(),
        }
    }
}
