mod planning;

pub use planning::{
    CausalInspectionEstimatedCost, CausalInspectionPlan, CausalInspectionPlanExplanation,
};

use super::admission::{admit_causal_inspection, CausalInspectionProofFlow};
use super::admission_trace::CausalDecisionTraceIndex;
use super::anchor::{anchor_causal_observation, CausalObservationAnchorError};
use super::inventory::CausalEvidenceFamily;
use super::materialization::{
    CausalInspectionMaterializationPolicy, CausalInspectionRedactionPolicy,
};
use super::receipt_types::{
    CausalInspectionReason, CausalObservationOutcome, QueryObservationReceipt,
};
use super::reference::{
    CausalEvidenceReference, CausalEvidenceReferenceResolution,
    CausalEvidenceReferenceResolutionDenial, CausalEvidenceReferenceSet,
};
use super::reference_resolution::resolve_causal_evidence_references;
use super::request::{
    causal_inspection_target, request_causal_inspection, CausalInspectionExplanationFamily,
    CausalInspectionRequest, CausalInspectionRequestError, CausalInspectionRichness,
};
use crate::domain_computation::evidence_identity::hash_parts;
use worth_query_admission::facade::basis::ScopedInspectionBasis;

/// Intent-first entrypoint for inspecting why an existing Query observation happened.
///
/// The builder is a convenience layer over the proof primitives. It still anchors the
/// observation, resolves evidence references, builds a Query inspection request, and
/// admits that request before any runtime bridge envelope is assembled.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CausalInspection {
    receipt: QueryObservationReceipt,
    inspection_basis: ScopedInspectionBasis,
    reason: Option<CausalInspectionReason>,
    explanation_family: CausalInspectionExplanationFamily,
    richness: CausalInspectionRichness,
    requested_evidence_families: Vec<CausalEvidenceFamily>,
    redaction_policy: CausalInspectionRedactionPolicy,
    materialization_policy: CausalInspectionMaterializationPolicy,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CausalInspectionSupportPosture {
    Admitted,
    Advisory,
    Denied,
}

impl CausalInspectionSupportPosture {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Admitted => "admitted",
            Self::Advisory => "advisory",
            Self::Denied => "denied",
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum CausalInspectionPlanError {
    BasisMismatch(CausalInspectionBasisMismatch),
    Anchor(CausalObservationAnchorError),
    MissingEvidence(CausalEvidenceReferenceResolutionDenial),
    Request(CausalInspectionRequestError),
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CausalInspectionPlanErrorKind {
    BasisMismatch,
    Anchor,
    MissingEvidence,
    Request,
}

impl CausalInspectionPlanErrorKind {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::BasisMismatch => "basis_mismatch",
            Self::Anchor => "anchor",
            Self::MissingEvidence => "missing_evidence",
            Self::Request => "request",
        }
    }
}

impl CausalInspectionPlanError {
    pub fn kind(&self) -> CausalInspectionPlanErrorKind {
        match self {
            Self::BasisMismatch(_) => CausalInspectionPlanErrorKind::BasisMismatch,
            Self::Anchor(_) => CausalInspectionPlanErrorKind::Anchor,
            Self::MissingEvidence(_) => CausalInspectionPlanErrorKind::MissingEvidence,
            Self::Request(_) => CausalInspectionPlanErrorKind::Request,
        }
    }

    pub fn failure_digest(&self) -> &str {
        match self {
            Self::BasisMismatch(error) => error.failure_digest(),
            Self::Anchor(error) => error.failure_digest(),
            Self::MissingEvidence(denial) => denial.failure_digest(),
            Self::Request(error) => error.failure_for_reporting(),
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CausalInspectionBasisMismatch {
    receipt_basis_digest: String,
    requested_basis_digest: String,
    failure_digest: String,
}

impl CausalInspectionBasisMismatch {
    fn new(receipt_basis: &ScopedInspectionBasis, requested_basis: &ScopedInspectionBasis) -> Self {
        let receipt_basis_digest = receipt_basis.scoped_basis_digest().to_string();
        let requested_basis_digest = requested_basis.scoped_basis_digest().to_string();
        let failure_digest = hash_parts(&[
            "causal_inspection_basis_mismatch_v1".to_string(),
            format!("receipt:{receipt_basis_digest}"),
            format!("requested:{requested_basis_digest}"),
        ]);
        Self {
            receipt_basis_digest,
            requested_basis_digest,
            failure_digest,
        }
    }

    pub fn receipt_basis_digest(&self) -> &str {
        &self.receipt_basis_digest
    }

    pub fn requested_basis_digest(&self) -> &str {
        &self.requested_basis_digest
    }

    pub fn failure_digest(&self) -> &str {
        &self.failure_digest
    }
}

impl CausalInspection {
    pub fn for_observation(
        receipt: impl Into<QueryObservationReceipt>,
        inspection_basis: ScopedInspectionBasis,
    ) -> Self {
        Self {
            receipt: receipt.into(),
            inspection_basis,
            reason: None,
            explanation_family: CausalInspectionExplanationFamily::CrossRuntimeCausalExplanation,
            richness: CausalInspectionRichness::ReferenceOnly,
            requested_evidence_families: Vec::new(),
            redaction_policy: CausalInspectionRedactionPolicy::PreserveDetail,
            materialization_policy:
                CausalInspectionMaterializationPolicy::OfflineInterpretableArtifact,
        }
    }

    pub fn why_changed(self) -> Self {
        self.because(CausalInspectionReason::ChangedResult)
    }

    pub fn why_suppressed(self) -> Self {
        self.because(CausalInspectionReason::SuppressedResult)
    }

    pub fn why_denied(self) -> Self {
        self.because(CausalInspectionReason::DeniedResult)
    }

    pub fn why_replayed(self) -> Self {
        self.because(CausalInspectionReason::HistoricalReplayResult)
    }

    pub fn why_previewed(self) -> Self {
        self.because(CausalInspectionReason::BranchPreviewResult)
    }

    pub fn why_temporal_wake(self) -> Self {
        self.because(CausalInspectionReason::ChangedResult)
            .evidence_families([
                CausalEvidenceFamily::QueryInspection,
                CausalEvidenceFamily::BridgeRoute,
                CausalEvidenceFamily::SignalInvalidation,
            ])
    }

    pub fn why_async_completion(self) -> Self {
        self.because(CausalInspectionReason::ChangedResult)
            .evidence_families([
                CausalEvidenceFamily::QueryInspection,
                CausalEvidenceFamily::BridgeRoute,
                CausalEvidenceFamily::SignalEvaluation,
            ])
    }

    pub fn why_remasked(self) -> Self {
        self.because(CausalInspectionReason::BranchPreviewResult)
            .evidence_families([
                CausalEvidenceFamily::QueryInspection,
                CausalEvidenceFamily::BridgeRoute,
                CausalEvidenceFamily::BridgePreview,
            ])
    }

    pub fn why_resume_mismatch(self) -> Self {
        self.because(CausalInspectionReason::HistoricalReplayResult)
            .evidence_families([
                CausalEvidenceFamily::QueryInspection,
                CausalEvidenceFamily::BridgeRoute,
                CausalEvidenceFamily::BridgeReplay,
                CausalEvidenceFamily::SignalReplayCursor,
                CausalEvidenceFamily::BridgeContinuity,
            ])
    }

    pub fn because(mut self, reason: CausalInspectionReason) -> Self {
        self.reason = Some(reason);
        self
    }

    pub fn reference_only(mut self) -> Self {
        self.richness = CausalInspectionRichness::ReferenceOnly;
        self
    }

    pub fn materialized_detail(mut self) -> Self {
        self.richness = CausalInspectionRichness::MaterializedDetail;
        self
    }

    pub fn redaction(mut self, policy: CausalInspectionRedactionPolicy) -> Self {
        self.redaction_policy = policy;
        self
    }

    pub fn materialization(mut self, policy: CausalInspectionMaterializationPolicy) -> Self {
        self.materialization_policy = policy;
        self
    }

    pub fn evidence_families<I>(mut self, families: I) -> Self
    where
        I: IntoIterator<Item = CausalEvidenceFamily>,
    {
        self.requested_evidence_families = families.into_iter().collect();
        self
    }

    pub fn include_all_retained_evidence(mut self) -> Self {
        self.requested_evidence_families.clear();
        self
    }

    pub fn durable_archive(mut self) -> Self {
        self.explanation_family = CausalInspectionExplanationFamily::DurableCausalArchive;
        self
    }

    pub fn store_backed_replay(mut self) -> Self {
        self.explanation_family =
            CausalInspectionExplanationFamily::StoreBackedReplayReconstruction;
        self
    }

    pub fn plan(self) -> Result<CausalInspectionPlan, CausalInspectionPlanError> {
        if self.receipt.inspection_basis() != &self.inspection_basis {
            return Err(CausalInspectionPlanError::BasisMismatch(
                CausalInspectionBasisMismatch::new(
                    self.receipt.inspection_basis(),
                    &self.inspection_basis,
                ),
            ));
        }
        let reason = self
            .reason
            .unwrap_or_else(|| planning::reason_for_outcome(self.receipt.outcome()));
        let anchor = anchor_causal_observation(self.receipt, reason)
            .map_err(CausalInspectionPlanError::Anchor)?;
        let target = causal_inspection_target(
            anchor.observation_receipt().observation_target().clone(),
            anchor.observation_receipt().result_shape_context().clone(),
        )
        .map_err(CausalInspectionPlanError::Request)?;
        let resolution =
            resolve_causal_evidence_references(anchor, &self.requested_evidence_families);
        let reference_set = match resolution {
            CausalEvidenceReferenceResolution::Resolved { reference_set, .. } => reference_set,
            CausalEvidenceReferenceResolution::MissingRequiredEvidence { denial, .. } => {
                return Err(CausalInspectionPlanError::MissingEvidence(denial));
            }
        };
        let request = request_causal_inspection(
            reference_set.clone(),
            target,
            self.explanation_family,
            self.richness,
            &self.requested_evidence_families,
        )
        .map_err(CausalInspectionPlanError::Request)?;
        let admission = admit_causal_inspection(request.clone());
        Ok(CausalInspectionPlan {
            inspection_basis: self.inspection_basis,
            reference_set,
            request,
            admission,
            redaction_policy: self.redaction_policy,
            materialization_policy: self.materialization_policy,
        })
    }
}
