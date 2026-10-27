use crate::domain_computation::evidence_identity::{
    WorthQueryAuthorityLane, WorthQueryReadExecutionEngine,
};
use crate::domain_computation::evidence_identity::{
    WorthQueryEvidenceIdentity, WorthQueryEvidenceScope, WorthQueryEvidenceTag,
};

use super::inventory::CausalEvidenceFamily;
use super::observation_identity::{
    CausalEvidenceReferenceDigest, CausalEvidenceReferenceInput, CausalObservationBasisIdentity,
    CausalObservationQueryIdentity, CausalObservationReceiptIdentity,
    CausalObservationTargetHandle, CausalQueryObservationReceiptIdentity,
    CausalResultShapeContextHandle,
};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum QueryObservationReceiptFamily {
    WriteReceipt,
    IntentReceipt,
    IntentDenial,
    BranchIntentReceipt,
    PreviewOutcome,
    ReadReceipt,
    Fixture,
}

impl QueryObservationReceiptFamily {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::WriteReceipt => "write_receipt",
            Self::IntentReceipt => "intent_receipt",
            Self::IntentDenial => "intent_denial",
            Self::BranchIntentReceipt => "branch_intent_receipt",
            Self::PreviewOutcome => "preview_outcome",
            Self::ReadReceipt => "read_receipt",
            Self::Fixture => "fixture",
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CausalObservationOutcome {
    Changed,
    Suppressed,
    Denied,
    BranchPreview,
    Replayed,
}

impl CausalObservationOutcome {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Changed => "changed",
            Self::Suppressed => "suppressed",
            Self::Denied => "denied",
            Self::BranchPreview => "branch_preview",
            Self::Replayed => "replayed",
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CausalInspectionReason {
    ChangedResult,
    SuppressedResult,
    DeniedResult,
    BranchPreviewResult,
    HistoricalReplayResult,
}

impl CausalInspectionReason {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::ChangedResult => "changed_result",
            Self::SuppressedResult => "suppressed_result",
            Self::DeniedResult => "denied_result",
            Self::BranchPreviewResult => "branch_preview_result",
            Self::HistoricalReplayResult => "historical_replay_result",
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum CausalObservationBasisPosture {
    AuthorityLane(WorthQueryAuthorityLane),
    ReadExecution(WorthQueryReadExecutionEngine),
    HistoricalReplayCertification,
    Fixture,
}

impl CausalObservationBasisPosture {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::AuthorityLane(lane) => lane.as_str(),
            Self::ReadExecution(engine) => engine.as_str(),
            Self::HistoricalReplayCertification => "historical_replay_certification",
            Self::Fixture => "fixture-basis-posture",
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CausalObservationEvidenceIdentity {
    family: CausalEvidenceFamily,
    reference_digest: CausalEvidenceReferenceDigest,
    evidence_identity: WorthQueryEvidenceIdentity,
}

impl CausalObservationEvidenceIdentity {
    pub fn new(
        family: CausalEvidenceFamily,
        reference_digest: impl Into<CausalEvidenceReferenceInput>,
    ) -> Self {
        let reference_digest = into_reference_digest(reference_digest.into());
        let evidence_identity =
            WorthQueryEvidenceIdentity::compose(WorthQueryEvidenceScope::CausalEvidenceReference)
                .field_shape(WorthQueryEvidenceTag::new("family"), family.as_str())
                .field_evidence_identity(
                    WorthQueryEvidenceTag::new("reference"),
                    reference_digest.evidence_identity(),
                )
                .seal();
        Self {
            family,
            reference_digest,
            evidence_identity,
        }
    }

    pub fn family(&self) -> CausalEvidenceFamily {
        self.family
    }

    pub fn reference_digest(&self) -> &CausalEvidenceReferenceDigest {
        &self.reference_digest
    }

    pub fn evidence_identity(&self) -> &WorthQueryEvidenceIdentity {
        &self.evidence_identity
    }
}

fn into_reference_digest(
    reference_input: CausalEvidenceReferenceInput,
) -> CausalEvidenceReferenceDigest {
    match reference_input {
        CausalEvidenceReferenceInput::Typed(identity) => identity,
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct QueryObservationReceipt {
    family: QueryObservationReceiptFamily,
    observation_receipt_identity: CausalObservationReceiptIdentity,
    query_identity: CausalObservationQueryIdentity,
    basis_posture: CausalObservationBasisPosture,
    basis_identity: CausalObservationBasisIdentity,
    inspection_basis: worth_query_admission::facade::basis::ScopedInspectionBasis,
    result_shape_context: CausalResultShapeContextHandle,
    observation_target: CausalObservationTargetHandle,
    outcome: CausalObservationOutcome,
    evidence_identities: Vec<CausalObservationEvidenceIdentity>,
    receipt_identity: CausalQueryObservationReceiptIdentity,
}

impl QueryObservationReceipt {
    pub fn from_parts(parts: ObservationReceiptParts) -> Self {
        let receipt_identity = WorthQueryEvidenceIdentity::compose(
            WorthQueryEvidenceScope::CausalQueryObservationReceipt,
        )
        .field_shape(WorthQueryEvidenceTag::new("family"), parts.family.as_str())
        .field_evidence_identity(
            WorthQueryEvidenceTag::new("observation"),
            parts.observation_receipt_identity.evidence_identity(),
        )
        .field_evidence_identity(
            WorthQueryEvidenceTag::new("query"),
            parts.query_identity.evidence_identity(),
        )
        .field_shape(
            WorthQueryEvidenceTag::new("basis_posture"),
            parts.basis_posture.as_str(),
        )
        .field_evidence_identity(
            WorthQueryEvidenceTag::new("basis"),
            parts.basis_identity.evidence_identity(),
        )
        .field_shape(
            WorthQueryEvidenceTag::new("scoped_inspection_basis"),
            parts.inspection_basis.scoped_basis_digest(),
        )
        .field_evidence_identity(
            WorthQueryEvidenceTag::new("result_shape_context"),
            parts.result_shape_context.identity().evidence_identity(),
        )
        .field_evidence_identity(
            WorthQueryEvidenceTag::new("observation_target"),
            parts.observation_target.identity().evidence_identity(),
        )
        .field_shape(
            WorthQueryEvidenceTag::new("outcome"),
            parts.outcome.as_str(),
        )
        .field_evidence_identity_sequence(
            WorthQueryEvidenceTag::new("evidence"),
            parts
                .evidence_identities
                .iter()
                .map(CausalObservationEvidenceIdentity::evidence_identity),
        )
        .seal()
        .into();
        Self {
            family: parts.family,
            observation_receipt_identity: parts.observation_receipt_identity,
            query_identity: parts.query_identity,
            basis_posture: parts.basis_posture,
            basis_identity: parts.basis_identity,
            inspection_basis: parts.inspection_basis,
            result_shape_context: parts.result_shape_context,
            observation_target: parts.observation_target,
            outcome: parts.outcome,
            evidence_identities: parts.evidence_identities,
            receipt_identity,
        }
    }

    pub fn family(&self) -> QueryObservationReceiptFamily {
        self.family
    }

    pub fn observation_receipt_identity(&self) -> &CausalObservationReceiptIdentity {
        &self.observation_receipt_identity
    }

    pub fn query_identity(&self) -> &CausalObservationQueryIdentity {
        &self.query_identity
    }

    pub fn basis_posture(&self) -> &str {
        self.basis_posture.as_str()
    }

    pub fn basis_posture_kind(&self) -> &CausalObservationBasisPosture {
        &self.basis_posture
    }

    pub fn basis_identity(&self) -> &CausalObservationBasisIdentity {
        &self.basis_identity
    }

    pub fn inspection_basis(&self) -> &worth_query_admission::facade::basis::ScopedInspectionBasis {
        &self.inspection_basis
    }

    pub fn result_shape_context(&self) -> &CausalResultShapeContextHandle {
        &self.result_shape_context
    }

    pub fn observation_target(&self) -> &CausalObservationTargetHandle {
        &self.observation_target
    }

    pub fn outcome(&self) -> CausalObservationOutcome {
        self.outcome
    }

    pub fn evidence_identities(&self) -> &[CausalObservationEvidenceIdentity] {
        &self.evidence_identities
    }

    pub fn receipt_identity(&self) -> &CausalQueryObservationReceiptIdentity {
        &self.receipt_identity
    }
}

pub struct ObservationReceiptParts {
    pub family: QueryObservationReceiptFamily,
    pub observation_receipt_identity: CausalObservationReceiptIdentity,
    pub query_identity: CausalObservationQueryIdentity,
    pub basis_posture: CausalObservationBasisPosture,
    pub basis_identity: CausalObservationBasisIdentity,
    pub inspection_basis: worth_query_admission::facade::basis::ScopedInspectionBasis,
    pub result_shape_context: CausalResultShapeContextHandle,
    pub observation_target: CausalObservationTargetHandle,
    pub outcome: CausalObservationOutcome,
    pub evidence_identities: Vec<CausalObservationEvidenceIdentity>,
}
