use super::identity::{
    compose_causal_inspection_request_failure_identity, compose_causal_inspection_request_identity,
    compose_causal_inspection_target_identity, CausalInspectionRequestIdentity,
    CausalInspectionTargetIdentity,
};
use super::inventory::CausalEvidenceFamily;
use super::observation_identity::{CausalObservationTargetHandle, CausalResultShapeContextHandle};
use super::reference::{CausalEvidenceReference, CausalEvidenceReferenceSet};
use crate::domain_computation::evidence_identity::{
    WorthQueryEvidenceIdentity, WorthQueryEvidenceScope, WorthQueryEvidenceTag,
};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CausalInspectionTarget {
    observation_target: CausalObservationTargetHandle,
    result_shape_context: CausalResultShapeContextHandle,
    target_identity: CausalInspectionTargetIdentity,
}

impl CausalInspectionTarget {
    fn new(
        observation_target: CausalObservationTargetHandle,
        result_shape_context: CausalResultShapeContextHandle,
    ) -> Result<Self, CausalInspectionRequestError> {
        if observation_target.identity().as_str().is_empty() {
            return Err(CausalInspectionRequestError::new(
                CausalInspectionRequestErrorKind::EmptyObservationTarget,
                "causal inspection targets require an observation target digest",
                &[],
            ));
        }
        if result_shape_context.identity().as_str().is_empty() {
            return Err(CausalInspectionRequestError::new(
                CausalInspectionRequestErrorKind::EmptyResultShapeContext,
                "causal inspection targets require a result-shape context digest",
                &[observation_target.identity().evidence_identity().clone()],
            ));
        }
        Ok(Self {
            target_identity: compose_causal_inspection_target_identity(
                &observation_target,
                &result_shape_context,
            ),
            observation_target,
            result_shape_context,
        })
    }

    pub fn observation_target_for_reporting(&self) -> &str {
        self.observation_target.identity().as_str()
    }

    pub(in crate::domain_computation::primary_graph) fn observation_target_identity(
        &self,
    ) -> &super::observation_identity::CausalObservationTargetIdentity {
        self.observation_target.identity()
    }

    pub fn result_shape_context_for_reporting(&self) -> &str {
        self.result_shape_context.identity().as_str()
    }

    pub(in crate::domain_computation::primary_graph) fn result_shape_context_identity(
        &self,
    ) -> &super::observation_identity::CausalResultShapeContextIdentity {
        self.result_shape_context.identity()
    }

    pub fn target_for_reporting(&self) -> &str {
        self.target_identity.as_str()
    }

    pub fn observation_target(&self) -> &CausalObservationTargetHandle {
        &self.observation_target
    }

    pub fn result_shape_context(&self) -> &CausalResultShapeContextHandle {
        &self.result_shape_context
    }

    pub fn target_identity(&self) -> &CausalInspectionTargetIdentity {
        &self.target_identity
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CausalInspectionExplanationFamily {
    CrossRuntimeCausalExplanation,
    DurableCausalArchive,
    StoreBackedReplayReconstruction,
}

impl CausalInspectionExplanationFamily {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::CrossRuntimeCausalExplanation => "cross_runtime_causal_explanation",
            Self::DurableCausalArchive => "durable_causal_archive",
            Self::StoreBackedReplayReconstruction => "store_backed_replay_reconstruction",
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CausalInspectionRichness {
    ReferenceOnly,
    MaterializedDetail,
}

impl CausalInspectionRichness {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::ReferenceOnly => "reference_only",
            Self::MaterializedDetail => "materialized_detail",
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CausalInspectionRequest {
    reference_set: CausalEvidenceReferenceSet,
    target: CausalInspectionTarget,
    explanation_family: CausalInspectionExplanationFamily,
    requested_richness: CausalInspectionRichness,
    requested_evidence_families: Vec<CausalEvidenceFamily>,
    inspection_basis_digest: String,
    request_identity: CausalInspectionRequestIdentity,
}

impl CausalInspectionRequest {
    pub(super) fn new(
        reference_set: CausalEvidenceReferenceSet,
        target: CausalInspectionTarget,
        explanation_family: CausalInspectionExplanationFamily,
        requested_richness: CausalInspectionRichness,
        requested_evidence_families: Vec<CausalEvidenceFamily>,
    ) -> Self {
        let inspection_basis_digest = reference_set
            .anchor()
            .observation_receipt()
            .inspection_basis()
            .scoped_basis_digest()
            .to_string();
        let request_identity = request_digest(
            &reference_set,
            &target,
            explanation_family,
            requested_richness,
            &requested_evidence_families,
            &inspection_basis_digest,
        );
        Self {
            reference_set,
            target,
            explanation_family,
            requested_richness,
            requested_evidence_families,
            inspection_basis_digest,
            request_identity,
        }
    }

    pub fn reference_set(&self) -> &CausalEvidenceReferenceSet {
        &self.reference_set
    }

    pub fn target(&self) -> &CausalInspectionTarget {
        &self.target
    }

    pub fn explanation_family(&self) -> CausalInspectionExplanationFamily {
        self.explanation_family
    }

    pub fn requested_richness(&self) -> CausalInspectionRichness {
        self.requested_richness
    }

    pub fn requested_evidence_families(&self) -> &[CausalEvidenceFamily] {
        &self.requested_evidence_families
    }

    pub fn inspection_basis_digest(&self) -> &str {
        &self.inspection_basis_digest
    }

    pub fn request_for_reporting(&self) -> &str {
        self.request_identity.as_str()
    }

    pub fn request_identity(&self) -> &CausalInspectionRequestIdentity {
        &self.request_identity
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CausalInspectionRequestErrorKind {
    EmptyObservationTarget,
    EmptyResultShapeContext,
    TargetObservationMismatch,
    ResultShapeContextMismatch,
    RequestedEvidenceFamilyMissing,
}

impl CausalInspectionRequestErrorKind {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::EmptyObservationTarget => "empty_observation_target",
            Self::EmptyResultShapeContext => "empty_result_shape_context",
            Self::TargetObservationMismatch => "target_observation_mismatch",
            Self::ResultShapeContextMismatch => "result_shape_context_mismatch",
            Self::RequestedEvidenceFamilyMissing => "requested_evidence_family_missing",
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CausalInspectionRequestError {
    kind: CausalInspectionRequestErrorKind,
    message: &'static str,
    failure_identity: super::identity::CausalInspectionRequestFailureIdentity,
}

impl CausalInspectionRequestError {
    fn new(
        kind: CausalInspectionRequestErrorKind,
        message: &'static str,
        evidence: &[WorthQueryEvidenceIdentity],
    ) -> Self {
        Self {
            kind,
            message,
            failure_identity: compose_causal_inspection_request_failure_identity(
                kind.as_str(),
                message,
                evidence,
            ),
        }
    }

    pub fn kind(&self) -> CausalInspectionRequestErrorKind {
        self.kind
    }

    pub fn message(&self) -> &str {
        self.message
    }

    pub fn failure_for_reporting(&self) -> &str {
        self.failure_identity.as_str()
    }

    pub fn failure_identity(&self) -> &super::identity::CausalInspectionRequestFailureIdentity {
        &self.failure_identity
    }
}

pub fn causal_inspection_target(
    observation_target: CausalObservationTargetHandle,
    result_shape_context: CausalResultShapeContextHandle,
) -> Result<CausalInspectionTarget, CausalInspectionRequestError> {
    CausalInspectionTarget::new(observation_target, result_shape_context)
}

pub fn request_causal_inspection(
    reference_set: CausalEvidenceReferenceSet,
    target: CausalInspectionTarget,
    explanation_family: CausalInspectionExplanationFamily,
    requested_richness: CausalInspectionRichness,
    requested_evidence_families: &[CausalEvidenceFamily],
) -> Result<CausalInspectionRequest, CausalInspectionRequestError> {
    let receipt = reference_set.anchor().observation_receipt();
    if target.observation_target_identity() != receipt.observation_target().identity() {
        return Err(CausalInspectionRequestError::new(
            CausalInspectionRequestErrorKind::TargetObservationMismatch,
            "causal inspection targets must match the anchored Query observation target",
            &[
                target
                    .observation_target_identity()
                    .evidence_identity()
                    .clone(),
                receipt
                    .observation_target()
                    .identity()
                    .evidence_identity()
                    .clone(),
            ],
        ));
    }
    if target.result_shape_context_identity() != receipt.result_shape_context().identity() {
        return Err(CausalInspectionRequestError::new(
            CausalInspectionRequestErrorKind::ResultShapeContextMismatch,
            "causal inspection targets must match the anchored result-shape context",
            &[
                target
                    .result_shape_context_identity()
                    .evidence_identity()
                    .clone(),
                receipt
                    .result_shape_context()
                    .identity()
                    .evidence_identity()
                    .clone(),
            ],
        ));
    }

    let families = requested_families_or_resolved(&reference_set, requested_evidence_families)?;
    Ok(CausalInspectionRequest::new(
        reference_set,
        target,
        explanation_family,
        requested_richness,
        families,
    ))
}

fn requested_families_or_resolved(
    reference_set: &CausalEvidenceReferenceSet,
    requested_families: &[CausalEvidenceFamily],
) -> Result<Vec<CausalEvidenceFamily>, CausalInspectionRequestError> {
    if requested_families.is_empty() {
        return Ok(reference_set
            .references()
            .iter()
            .map(CausalEvidenceReference::family)
            .collect());
    }
    let missing_family = requested_families.iter().find(|family| {
        !reference_set
            .references()
            .iter()
            .any(|reference| reference.family() == **family)
    });
    if let Some(family) = missing_family {
        return Err(CausalInspectionRequestError::new(
            CausalInspectionRequestErrorKind::RequestedEvidenceFamilyMissing,
            "causal inspection requests may only ask for resolved evidence families",
            &[
                missing_family_failure_evidence(*family),
                reference_set
                    .reference_set_digest()
                    .evidence_identity()
                    .clone(),
            ],
        ));
    }
    Ok(requested_families.to_vec())
}

fn missing_family_failure_evidence(family: CausalEvidenceFamily) -> WorthQueryEvidenceIdentity {
    WorthQueryEvidenceIdentity::compose(WorthQueryEvidenceScope::CausalInspectionRequestFailure)
        .field_shape(
            WorthQueryEvidenceTag::new("role"),
            "missing-requested-evidence-family",
        )
        .field_shape(WorthQueryEvidenceTag::new("family"), family.as_str())
        .seal()
}

fn request_digest(
    reference_set: &CausalEvidenceReferenceSet,
    target: &CausalInspectionTarget,
    explanation_family: CausalInspectionExplanationFamily,
    requested_richness: CausalInspectionRichness,
    requested_evidence_families: &[CausalEvidenceFamily],
    inspection_basis_digest: &str,
) -> CausalInspectionRequestIdentity {
    compose_causal_inspection_request_identity(
        reference_set.anchor().anchor_digest(),
        reference_set.reference_set_digest(),
        target.target_identity(),
        explanation_family,
        requested_richness,
        requested_evidence_families,
        inspection_basis_digest,
    )
}
