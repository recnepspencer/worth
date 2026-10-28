use std::collections::BTreeSet;

use crate::domain_computation::evidence_identity::{
    WorthQueryEvidenceIdentity, WorthQueryEvidenceScope, WorthQueryEvidenceTag,
};

use super::observation_identity::{
    CausalObservationAnchorCountersIdentity, CausalObservationAnchorDigest,
    CausalObservationAnchorFailureIdentity,
};
use super::receipt_types::{
    CausalInspectionReason, CausalObservationOutcome, QueryObservationReceipt,
};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CausalObservationMissingReferencePosture {
    Complete,
    MissingOptional,
    MissingRequiredDenied,
}

impl CausalObservationMissingReferencePosture {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Complete => "complete",
            Self::MissingOptional => "missing_optional",
            Self::MissingRequiredDenied => "missing_required_denied",
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CausalObservationAnchorCounters {
    source_receipt_family_count: usize,
    reference_family_count: usize,
    missing_reference_posture: CausalObservationMissingReferencePosture,
    anchor_digest_width: usize,
    runtime_graph_scan_count: usize,
    diagnostics_retention_scan_count: usize,
    counter_identity: CausalObservationAnchorCountersIdentity,
}

impl CausalObservationAnchorCounters {
    fn new(
        reference_family_count: usize,
        missing_reference_posture: CausalObservationMissingReferencePosture,
        anchor_digest_width: usize,
    ) -> Self {
        let counter_identity = WorthQueryEvidenceIdentity::compose(
            WorthQueryEvidenceScope::CausalObservationAnchorCounters,
        )
        .field_usize(WorthQueryEvidenceTag::new("source_receipt_family_count"), 1)
        .field_usize(
            WorthQueryEvidenceTag::new("reference_family_count"),
            reference_family_count,
        )
        .field_shape(
            WorthQueryEvidenceTag::new("missing_reference_posture"),
            missing_reference_posture.as_str(),
        )
        .field_usize(
            WorthQueryEvidenceTag::new("anchor_digest_width"),
            anchor_digest_width,
        )
        .field_usize(WorthQueryEvidenceTag::new("runtime_graph_scan_count"), 0)
        .field_usize(
            WorthQueryEvidenceTag::new("diagnostics_retention_scan_count"),
            0,
        )
        .seal()
        .into();
        Self {
            source_receipt_family_count: 1,
            reference_family_count,
            missing_reference_posture,
            anchor_digest_width,
            runtime_graph_scan_count: 0,
            diagnostics_retention_scan_count: 0,
            counter_identity,
        }
    }

    pub fn source_receipt_family_count(&self) -> usize {
        self.source_receipt_family_count
    }

    pub fn reference_family_count(&self) -> usize {
        self.reference_family_count
    }

    pub fn missing_reference_posture(&self) -> CausalObservationMissingReferencePosture {
        self.missing_reference_posture
    }

    pub fn anchor_digest_width(&self) -> usize {
        self.anchor_digest_width
    }

    pub fn runtime_graph_scan_count(&self) -> usize {
        self.runtime_graph_scan_count
    }

    pub fn diagnostics_retention_scan_count(&self) -> usize {
        self.diagnostics_retention_scan_count
    }

    pub fn counter_snapshot(&self) -> &str {
        self.counter_identity.as_str()
    }

    pub fn counter_identity(&self) -> &CausalObservationAnchorCountersIdentity {
        &self.counter_identity
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CausalObservationAnchorErrorKind {
    MissingObservationReceipt,
    MissingRequiredEvidenceReference,
    InspectionReasonOutcomeMismatch,
}

impl CausalObservationAnchorErrorKind {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::MissingObservationReceipt => "missing_observation_receipt",
            Self::MissingRequiredEvidenceReference => "missing_required_evidence_reference",
            Self::InspectionReasonOutcomeMismatch => "inspection_reason_outcome_mismatch",
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CausalObservationAnchorError {
    kind: CausalObservationAnchorErrorKind,
    message: &'static str,
    failure_identity: CausalObservationAnchorFailureIdentity,
}

impl CausalObservationAnchorError {
    fn new(
        kind: CausalObservationAnchorErrorKind,
        message: &'static str,
        evidence: &[String],
    ) -> Self {
        let failure_identity = WorthQueryEvidenceIdentity::compose(
            WorthQueryEvidenceScope::CausalObservationAnchorFailure,
        )
        .field_shape(WorthQueryEvidenceTag::new("kind"), kind.as_str())
        .field_value(WorthQueryEvidenceTag::new("message"), message)
        .field_value_sequence(
            WorthQueryEvidenceTag::new("evidence"),
            evidence.iter().map(String::as_str),
        )
        .seal()
        .into();
        Self {
            kind,
            message,
            failure_identity,
        }
    }

    pub fn kind(&self) -> CausalObservationAnchorErrorKind {
        self.kind
    }

    pub fn message(&self) -> &str {
        self.message
    }

    pub fn failure_digest(&self) -> &str {
        self.failure_identity.as_str()
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CausalObservationAnchor {
    observation_receipt: QueryObservationReceipt,
    inspection_reason: CausalInspectionReason,
    lower_runtime_evidence_family_count: usize,
    missing_reference_posture: CausalObservationMissingReferencePosture,
    anchor_digest: CausalObservationAnchorDigest,
    counters: CausalObservationAnchorCounters,
}

impl CausalObservationAnchor {
    pub fn observation_receipt(&self) -> &QueryObservationReceipt {
        &self.observation_receipt
    }

    pub fn inspection_reason(&self) -> CausalInspectionReason {
        self.inspection_reason
    }

    pub fn lower_runtime_evidence_family_count(&self) -> usize {
        self.lower_runtime_evidence_family_count
    }

    pub fn missing_reference_posture(&self) -> CausalObservationMissingReferencePosture {
        self.missing_reference_posture
    }

    pub fn anchor_digest(&self) -> &CausalObservationAnchorDigest {
        &self.anchor_digest
    }

    pub fn counters(&self) -> &CausalObservationAnchorCounters {
        &self.counters
    }
}

pub fn anchor_causal_observation(
    observation_receipt: impl Into<QueryObservationReceipt>,
    inspection_reason: CausalInspectionReason,
) -> Result<CausalObservationAnchor, CausalObservationAnchorError> {
    let observation_receipt = observation_receipt.into();
    if !inspection_reason_matches_observation_outcome(&inspection_reason, &observation_receipt) {
        return Err(CausalObservationAnchorError::new(
            CausalObservationAnchorErrorKind::InspectionReasonOutcomeMismatch,
            "causal observation anchor reason must match the observed Query outcome",
            &[
                format!("reason:{}", inspection_reason.as_str()),
                format!("outcome:{}", observation_receipt.outcome().as_str()),
            ],
        ));
    }
    if observation_receipt
        .observation_receipt_identity()
        .as_str()
        .is_empty()
    {
        return Err(CausalObservationAnchorError::new(
            CausalObservationAnchorErrorKind::MissingObservationReceipt,
            "causal observation anchors require one canonical Query observation receipt",
            &[format!("family:{}", observation_receipt.family().as_str())],
        ));
    }
    if observation_receipt.evidence_identities().is_empty() {
        return Err(CausalObservationAnchorError::new(
            CausalObservationAnchorErrorKind::MissingRequiredEvidenceReference,
            "causal observation anchors require at least one lower-runtime or Query evidence identity carried by the source receipt",
            &[format!(
                "observation:{}",
                observation_receipt.observation_receipt_identity().as_str()
            )],
        ));
    }
    let unique_families = observation_receipt
        .evidence_identities()
        .iter()
        .map(|identity| identity.family())
        .collect::<BTreeSet<_>>();
    let anchor_digest: CausalObservationAnchorDigest =
        WorthQueryEvidenceIdentity::compose(WorthQueryEvidenceScope::CausalObservationAnchor)
            .field_shape(
                WorthQueryEvidenceTag::new("reason"),
                inspection_reason.as_str(),
            )
            .field_evidence_identity(
                WorthQueryEvidenceTag::new("receipt"),
                observation_receipt.receipt_identity().evidence_identity(),
            )
            .field_evidence_identity(
                WorthQueryEvidenceTag::new("observation"),
                observation_receipt
                    .observation_receipt_identity()
                    .evidence_identity(),
            )
            .field_evidence_identity(
                WorthQueryEvidenceTag::new("query"),
                observation_receipt.query_identity().evidence_identity(),
            )
            .field_shape(
                WorthQueryEvidenceTag::new("basis_posture"),
                observation_receipt.basis_posture(),
            )
            .field_evidence_identity(
                WorthQueryEvidenceTag::new("basis"),
                observation_receipt.basis_identity().evidence_identity(),
            )
            .field_evidence_identity(
                WorthQueryEvidenceTag::new("result_shape_context"),
                observation_receipt
                    .result_shape_context()
                    .identity()
                    .evidence_identity(),
            )
            .field_evidence_identity(
                WorthQueryEvidenceTag::new("observation_target"),
                observation_receipt
                    .observation_target()
                    .identity()
                    .evidence_identity(),
            )
            .field_shape(
                WorthQueryEvidenceTag::new("outcome"),
                observation_receipt.outcome().as_str(),
            )
            .field_evidence_identity_sequence(
                WorthQueryEvidenceTag::new("evidence"),
                observation_receipt
                    .evidence_identities()
                    .iter()
                    .map(|identity| identity.evidence_identity()),
            )
            .seal()
            .into();
    let missing_reference_posture = CausalObservationMissingReferencePosture::Complete;
    let counters = CausalObservationAnchorCounters::new(
        unique_families.len(),
        missing_reference_posture,
        anchor_digest.as_str().len(),
    );

    Ok(CausalObservationAnchor {
        observation_receipt,
        inspection_reason,
        lower_runtime_evidence_family_count: unique_families.len(),
        missing_reference_posture,
        anchor_digest,
        counters,
    })
}

fn inspection_reason_matches_observation_outcome(
    inspection_reason: &CausalInspectionReason,
    observation_receipt: &QueryObservationReceipt,
) -> bool {
    matches!(
        (inspection_reason, observation_receipt.outcome()),
        (
            CausalInspectionReason::ChangedResult,
            CausalObservationOutcome::Changed
        ) | (
            CausalInspectionReason::SuppressedResult,
            CausalObservationOutcome::Suppressed
        ) | (
            CausalInspectionReason::DeniedResult,
            CausalObservationOutcome::Denied
        ) | (
            CausalInspectionReason::BranchPreviewResult,
            CausalObservationOutcome::BranchPreview
        ) | (
            CausalInspectionReason::HistoricalReplayResult,
            CausalObservationOutcome::Replayed
        )
    )
}
