use crate::evidence_identity::WorthQueryEvidenceIdentity;
use crate::evidence_identity::WorthQueryEvidenceIdentityDerivationDenial as IdentityDerivationDenial;

use super::super::super::evidence_identities::{
    diagnostic_assembly_receipt_identity, diagnostic_bundle_width_identity,
    diagnostic_counters_identity, diagnostic_semantic_labels_identity,
};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum BundleAssemblyPosture {
    ComposedFromCanonicalArtifacts,
    PartialRediscoveryDebtExplicit,
    PartialRediscoveryDenied,
}

impl BundleAssemblyPosture {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::ComposedFromCanonicalArtifacts => "composed_from_canonical_artifacts",
            Self::PartialRediscoveryDebtExplicit => "partial_rediscovery_debt_explicit",
            Self::PartialRediscoveryDenied => "partial_rediscovery_denied",
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct QuerySubscriptionDiagnosticBundleWidth {
    stage_evidence_count: usize,
    failure_evidence_count: usize,
    hostile_row_reference_count: usize,
    bundle_width_identity: WorthQueryEvidenceIdentity,
}

impl QuerySubscriptionDiagnosticBundleWidth {
    pub(super) fn new(
        stage_evidence_count: usize,
        failure_evidence_count: usize,
        hostile_row_reference_count: usize,
    ) -> Result<Self, IdentityDerivationDenial> {
        let bundle_width_identity = diagnostic_bundle_width_identity(
            stage_evidence_count,
            failure_evidence_count,
            hostile_row_reference_count,
        )?;
        Ok(Self {
            stage_evidence_count,
            failure_evidence_count,
            hostile_row_reference_count,
            bundle_width_identity,
        })
    }

    pub fn stage_evidence_count(&self) -> usize {
        self.stage_evidence_count
    }

    pub fn failure_evidence_count(&self) -> usize {
        self.failure_evidence_count
    }

    pub fn hostile_row_reference_count(&self) -> usize {
        self.hostile_row_reference_count
    }

    pub fn bundle_width_identity(&self) -> &WorthQueryEvidenceIdentity {
        &self.bundle_width_identity
    }
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct QuerySubscriptionDiagnosticCounters {
    diagnostic_trace_emission_count: u64,
    diagnostic_bundle_emission_count: u64,
    denied_bundle_emission_count: u64,
    diagnostic_missing_stage_denial_count: u64,
    diagnostic_bundle_width: u64,
}

impl QuerySubscriptionDiagnosticCounters {
    pub fn evidence_identity(
        &self,
    ) -> Result<WorthQueryEvidenceIdentity, IdentityDerivationDenial> {
        Ok(diagnostic_counters_identity(
            self.diagnostic_trace_emission_count,
            self.diagnostic_bundle_emission_count,
            self.denied_bundle_emission_count,
            self.diagnostic_missing_stage_denial_count,
            self.diagnostic_bundle_width,
        )?)
    }

    pub fn diagnostic_trace_emission_count(&self) -> u64 {
        self.diagnostic_trace_emission_count
    }

    pub fn diagnostic_bundle_emission_count(&self) -> u64 {
        self.diagnostic_bundle_emission_count
    }

    pub fn denied_bundle_emission_count(&self) -> u64 {
        self.denied_bundle_emission_count
    }

    pub fn diagnostic_missing_stage_denial_count(&self) -> u64 {
        self.diagnostic_missing_stage_denial_count
    }

    pub fn diagnostic_bundle_width(&self) -> u64 {
        self.diagnostic_bundle_width
    }

    pub(super) fn missing_stage_denied() -> Self {
        Self {
            diagnostic_missing_stage_denial_count: 1,
            ..Default::default()
        }
    }

    pub(crate) fn trace_emitted(width: u64) -> Self {
        Self {
            diagnostic_trace_emission_count: 1,
            diagnostic_bundle_width: width,
            ..Default::default()
        }
    }

    pub(crate) fn admitted_bundle_emitted(trace_count: u64, width: u64) -> Self {
        Self {
            diagnostic_trace_emission_count: trace_count,
            diagnostic_bundle_emission_count: 1,
            diagnostic_bundle_width: width,
            ..Default::default()
        }
    }

    pub(crate) fn denied_bundle_emitted(trace_count: u64, width: u64) -> Self {
        Self {
            diagnostic_trace_emission_count: trace_count,
            denied_bundle_emission_count: 1,
            diagnostic_bundle_width: width,
            ..Default::default()
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DiagnosticAssemblyReceipt {
    bundle_assembly_posture: BundleAssemblyPosture,
    stage_evidence_composition_count: usize,
    semantic_label_carry_forward_count: usize,
    stage_rederivation_count: usize,
    bundle_width: QuerySubscriptionDiagnosticBundleWidth,
    assembly_receipt_identity: WorthQueryEvidenceIdentity,
}

impl DiagnosticAssemblyReceipt {
    pub(super) fn new(
        bundle_assembly_posture: BundleAssemblyPosture,
        stage_evidence_composition_count: usize,
        semantic_label_carry_forward_count: usize,
        stage_rederivation_count: usize,
        bundle_width: QuerySubscriptionDiagnosticBundleWidth,
    ) -> Result<Self, IdentityDerivationDenial> {
        let assembly_receipt_identity = diagnostic_assembly_receipt_identity(
            bundle_assembly_posture.as_str(),
            stage_evidence_composition_count,
            semantic_label_carry_forward_count,
            stage_rederivation_count,
            bundle_width.bundle_width_identity(),
        )?;
        Ok(Self {
            bundle_assembly_posture,
            stage_evidence_composition_count,
            semantic_label_carry_forward_count,
            stage_rederivation_count,
            bundle_width,
            assembly_receipt_identity,
        })
    }

    pub fn bundle_assembly_posture(&self) -> &BundleAssemblyPosture {
        &self.bundle_assembly_posture
    }

    pub fn stage_evidence_composition_count(&self) -> usize {
        self.stage_evidence_composition_count
    }

    pub fn semantic_label_carry_forward_count(&self) -> usize {
        self.semantic_label_carry_forward_count
    }

    pub fn stage_rederivation_count(&self) -> usize {
        self.stage_rederivation_count
    }

    pub fn bundle_width(&self) -> &QuerySubscriptionDiagnosticBundleWidth {
        &self.bundle_width
    }

    pub fn assembly_receipt_identity(&self) -> &WorthQueryEvidenceIdentity {
        &self.assembly_receipt_identity
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct QuerySubscriptionDiagnosticSemanticLabels {
    query_family_label: String,
    declaration_family_label: String,
    bridge_family_label: String,
    bridge_slice_labels: Vec<String>,
    basis_posture_label: String,
    signal_strategy_class_label: String,
    live_graph_access_posture_label: String,
    support_posture_label: String,
    denial_or_coverage_class_label: String,
    labels_identity: WorthQueryEvidenceIdentity,
}

impl QuerySubscriptionDiagnosticSemanticLabels {
    pub(super) fn new(
        query_family_label: String,
        declaration_family_label: String,
        bridge_family_label: String,
        bridge_slice_labels: Vec<String>,
        basis_posture_label: String,
        signal_strategy_class_label: String,
        live_graph_access_posture_label: String,
        support_posture_label: String,
        denial_or_coverage_class_label: String,
    ) -> Result<Self, IdentityDerivationDenial> {
        let labels_identity = diagnostic_semantic_labels_identity(
            &query_family_label,
            &declaration_family_label,
            &bridge_family_label,
            &bridge_slice_labels,
            &basis_posture_label,
            &signal_strategy_class_label,
            &live_graph_access_posture_label,
            &support_posture_label,
            &denial_or_coverage_class_label,
        )?;
        Ok(Self {
            query_family_label,
            declaration_family_label,
            bridge_family_label,
            bridge_slice_labels,
            basis_posture_label,
            signal_strategy_class_label,
            live_graph_access_posture_label,
            support_posture_label,
            denial_or_coverage_class_label,
            labels_identity,
        })
    }

    pub fn query_family_label(&self) -> &str {
        &self.query_family_label
    }

    pub fn declaration_family_label(&self) -> &str {
        &self.declaration_family_label
    }

    pub fn bridge_family_label(&self) -> &str {
        &self.bridge_family_label
    }

    pub fn bridge_slice_labels(&self) -> &[String] {
        &self.bridge_slice_labels
    }

    pub fn basis_posture_label(&self) -> &str {
        &self.basis_posture_label
    }

    pub fn signal_strategy_class_label(&self) -> &str {
        &self.signal_strategy_class_label
    }

    pub fn live_graph_access_posture_label(&self) -> &str {
        &self.live_graph_access_posture_label
    }

    pub fn support_posture_label(&self) -> &str {
        &self.support_posture_label
    }

    pub fn denial_or_coverage_class_label(&self) -> &str {
        &self.denial_or_coverage_class_label
    }

    pub fn labels_identity(&self) -> &WorthQueryEvidenceIdentity {
        &self.labels_identity
    }
}
