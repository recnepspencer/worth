use crate::basis_lifecycle::BasisOperationLane;
use crate::domain_installation::operation_authority_chain::{
    mint_operation_phase_proof, operation_phase_basis, WorthQueryOperationPhaseProof,
    WorthQueryPromotionOnReferencePhase,
};
use crate::domain_installation::operation_identity_basis::canonical_operation_identity;
use crate::domain_installation::{
    WorthQueryOperationProjectionRole, WorthQueryOperationPromotionContract,
    WorthQueryOperationPublicationContract, WorthQueryPublishedWorkflow,
};
use crate::evidence_identity::WorthQueryEvidenceIdentityDerivationDenial as IdentityDerivationDenial;
use worth_proof::TransitionOutcome;
use worth_schema_graph::facade::{
    lower_graph_promotion_identity_basis, CarryingArtifactIdentity, DurableReferenceKind,
    PromotionRequest, SubelementKey,
};

use super::promotion_identity::{admit_graph_promotion_identity, WorthQueryPromotedGraphIdentity};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorthQueryDurableReferenceIntent {
    reference_kind: DurableReferenceKind,
    carrying_projection_role: WorthQueryOperationProjectionRole,
    lineage_evidence_index: usize,
    lineage_subject_ordinal: usize,
}

impl WorthQueryDurableReferenceIntent {
    pub const fn new(
        reference_kind: DurableReferenceKind,
        carrying_projection_role: WorthQueryOperationProjectionRole,
        lineage_evidence_index: usize,
        lineage_subject_ordinal: usize,
    ) -> Self {
        Self {
            reference_kind,
            carrying_projection_role,
            lineage_evidence_index,
            lineage_subject_ordinal,
        }
    }

    pub const fn reference_kind(&self) -> DurableReferenceKind {
        self.reference_kind
    }
    pub const fn carrying_projection_role(&self) -> &WorthQueryOperationProjectionRole {
        &self.carrying_projection_role
    }
    pub const fn lineage_evidence_index(&self) -> usize {
        self.lineage_evidence_index
    }
    pub const fn lineage_subject_ordinal(&self) -> usize {
        self.lineage_subject_ordinal
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct WorthQueryPromotionOnReferenceCounters {
    pub promotion_contract_checks: usize,
    pub carrying_publication_checks: usize,
    pub lineage_evidence_lookups: usize,
    pub referenced_subelements: usize,
    pub unreferenced_subelement_scans: usize,
    pub unrelated_trace_scans: usize,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WorthQueryPromotionOnReferenceDenial {
    StaleInstallationGeneration,
    PromotionNotDeclared,
    CarryingPublicationMismatch,
    LineageMissing,
    LineageEvidenceMissing,
    LineageStageDoesNotCarryPublication,
    LineageIsNotAuthoritative,
    LineageSubjectMissing,
    LineageSubjectEntityBindingUnavailable,
    LineageSubjectNotCarriedByPublication,
    InvalidCarryingPublicationIdentity,
}

pub struct WorthQueryPromotionOnReferenceCapability {
    identity: String,
    publication_identity: String,
    lineage_report_identity: String,
    intent: WorthQueryDurableReferenceIntent,
    promoted_graph_identity: WorthQueryPromotedGraphIdentity,
    counters: WorthQueryPromotionOnReferenceCounters,
    proof: WorthQueryOperationPhaseProof<WorthQueryPromotionOnReferencePhase>,
}

impl WorthQueryPromotionOnReferenceCapability {
    pub fn identity(&self) -> &str {
        debug_assert_eq!(self.proof.payload().identity(), self.identity);
        &self.identity
    }
    pub fn publication_identity(&self) -> &str {
        &self.publication_identity
    }
    pub fn lineage_report_identity(&self) -> &str {
        &self.lineage_report_identity
    }
    pub fn intent(&self) -> &WorthQueryDurableReferenceIntent {
        &self.intent
    }
    pub fn promoted_graph_identity(&self) -> &WorthQueryPromotedGraphIdentity {
        &self.promoted_graph_identity
    }
    pub const fn counters(&self) -> WorthQueryPromotionOnReferenceCounters {
        self.counters
    }
}

pub type WorthQueryPromotionOnReferenceOutcome = TransitionOutcome<
    WorthQueryPromotionOnReferenceCapability,
    WorthQueryPromotionOnReferenceDenial,
    std::convert::Infallible,
    WorthQueryPromotionOnReferenceDenial,
    WorthQueryPromotionOnReferenceDenial,
    WorthQueryPromotionOnReferenceDenial,
>;

impl<D, O, F, L: BasisOperationLane> WorthQueryPublishedWorkflow<D, O, F, L> {
    pub fn admit_promotion_on_reference(
        &self,
        intent: WorthQueryDurableReferenceIntent,
    ) -> Result<WorthQueryPromotionOnReferenceOutcome, IdentityDerivationDenial> {
        let trace = self.trace();
        if !trace.bound().installation_is_current() {
            return Ok(TransitionOutcome::Stale(
                WorthQueryPromotionOnReferenceDenial::StaleInstallationGeneration,
            ));
        }
        let mut counters = WorthQueryPromotionOnReferenceCounters {
            promotion_contract_checks: 1,
            carrying_publication_checks: 1,
            lineage_evidence_lookups: 1,
            ..Default::default()
        };
        if trace.bound().definition().semantics().promotion
            != WorthQueryOperationPromotionContract::OnDurableReference
        {
            return Ok(denied(
                WorthQueryPromotionOnReferenceDenial::PromotionNotDeclared,
            ));
        }
        let WorthQueryOperationPublicationContract::DerivedProjection { projection_role } =
            &trace.bound().definition().semantics().publication
        else {
            return Ok(denied(
                WorthQueryPromotionOnReferenceDenial::CarryingPublicationMismatch,
            ));
        };
        if projection_role != intent.carrying_projection_role() {
            return Ok(denied(
                WorthQueryPromotionOnReferenceDenial::CarryingPublicationMismatch,
            ));
        }
        let Some(report) = trace.lineage_report() else {
            return Ok(denied(WorthQueryPromotionOnReferenceDenial::LineageMissing));
        };
        let Some(evidence) = report.evidence().get(intent.lineage_evidence_index()) else {
            return Ok(denied(
                WorthQueryPromotionOnReferenceDenial::LineageEvidenceMissing,
            ));
        };
        if evidence.stage_identity() != self.publication_stage_identity() {
            return Ok(denied(
                WorthQueryPromotionOnReferenceDenial::LineageStageDoesNotCarryPublication,
            ));
        }
        if !evidence.outcome().is_authoritative_continuity() {
            return Ok(denied(
                WorthQueryPromotionOnReferenceDenial::LineageIsNotAuthoritative,
            ));
        }
        let Some(subject) = evidence
            .outcome()
            .authoritative_subject_evidence_identity(intent.lineage_subject_ordinal())
        else {
            return Ok(denied(
                WorthQueryPromotionOnReferenceDenial::LineageSubjectMissing,
            ));
        };
        let Some(subject_entity) = evidence
            .outcome()
            .authoritative_subject_entity_identity(intent.lineage_subject_ordinal())
        else {
            return Ok(denied(
                WorthQueryPromotionOnReferenceDenial::LineageSubjectEntityBindingUnavailable,
            ));
        };
        if !self.publication_carries_entity(subject_entity) {
            return Ok(denied(
                WorthQueryPromotionOnReferenceDenial::LineageSubjectNotCarriedByPublication,
            ));
        }
        let Ok(subelement_key) = SubelementKey::new(subject.as_str()) else {
            return Ok(denied(
                WorthQueryPromotionOnReferenceDenial::LineageSubjectMissing,
            ));
        };
        let graph_request = PromotionRequest::new(intent.reference_kind(), subelement_key);
        counters.referenced_subelements += 1;
        let Ok(carrying_artifact) = CarryingArtifactIdentity::new(self.receipt_identity()) else {
            return Ok(denied(
                WorthQueryPromotionOnReferenceDenial::InvalidCarryingPublicationIdentity,
            ));
        };
        let promoted_graph_identity = admit_graph_promotion_identity(
            lower_graph_promotion_identity_basis(graph_request.clone(), carrying_artifact),
        );
        let identity = canonical_operation_identity(
            "promotion-on-reference-v2",
            vec![
                ("promotion.publication", self.receipt_identity().to_owned()),
                ("promotion.lineage_report", report.identity().to_owned()),
                (
                    "promotion.reference_kind",
                    graph_request.reference_kind().as_str().to_owned(),
                ),
                (
                    "promotion.subelement",
                    graph_request.subelement_key().as_str().to_owned(),
                ),
            ],
        )?;
        let proof = mint_operation_phase_proof(
            identity.clone(),
            Some(self.trace().phase_proof().payload().identity()),
            operation_phase_basis(self.trace().phase_proof()).clone(),
        );
        Ok(TransitionOutcome::Success(
            WorthQueryPromotionOnReferenceCapability {
                identity,
                publication_identity: self.receipt_identity().to_owned(),
                lineage_report_identity: report.identity().to_owned(),
                intent,
                promoted_graph_identity,
                counters,
                proof,
            },
        ))
    }
}

fn denied(denial: WorthQueryPromotionOnReferenceDenial) -> WorthQueryPromotionOnReferenceOutcome {
    TransitionOutcome::Denied(denial)
}
