mod from_receipts;
mod from_replay;

use crate::evidence_identity::{
    WorthQueryEvidenceIdentity, WorthQueryEvidenceScope, WorthQueryEvidenceTag,
};
use crate::runtime::{
    WorthQueryBranchIntentReceiptInspection, WorthQueryIntentDenialInspection,
    WorthQueryIntentReceiptInspection, WorthQueryPreviewOutcomeInspection, WorthQueryReadReceipt,
    WorthQueryWriteReceiptInspection,
};

use super::inventory::CausalEvidenceFamily;
use super::observation_identity::{CausalObservationTargetHandle, CausalResultShapeContextHandle};
use super::receipt_helpers::{
    causal_evidence_reference_identity_digest, causal_observation_basis_evidence_identity,
    causal_observation_query_evidence_identity, causal_observation_receipt_evidence_identity,
    read_observation_query_identity, read_observation_receipt_identity,
    read_observation_result_reference_digest, write_observation_query_identity,
};
use super::receipt_types::{
    CausalObservationBasisPosture, CausalObservationEvidenceIdentity, CausalObservationOutcome,
    ObservationReceiptParts, QueryObservationReceiptFamily,
};

/// Directional adapter from supported engine receipts to the one causal core.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct QueryObservationReceipt(
    worth_query_execution::facade::causal_inspection::QueryObservationReceipt,
);
impl QueryObservationReceipt {
    fn from_parts(parts: ObservationReceiptParts) -> Self {
        Self(
            worth_query_execution::facade::causal_inspection::QueryObservationReceipt::from_parts(
                parts,
            ),
        )
    }
    #[cfg(test)]
    pub(in crate::runtime) fn inspection_basis_for_test(
        &self,
    ) -> crate::basis_lifecycle::ScopedInspectionBasis {
        self.0.inspection_basis().clone()
    }
}
impl std::ops::Deref for QueryObservationReceipt {
    type Target = worth_query_execution::facade::causal_inspection::QueryObservationReceipt;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
impl From<QueryObservationReceipt>
    for worth_query_execution::facade::causal_inspection::QueryObservationReceipt
{
    fn from(receipt: QueryObservationReceipt) -> Self {
        receipt.0
    }
}

impl QueryObservationReceipt {
    #[cfg(test)]
    pub(in crate::runtime) fn fixture(
        outcome: CausalObservationOutcome,
        evidence_identities: Vec<CausalObservationEvidenceIdentity>,
    ) -> Self {
        let inspection_basis = fixture_inspection_basis(outcome);
        let fixture_authority = fixture_authority_identity(outcome.as_str());
        Self::from_parts(ObservationReceiptParts {
            family: QueryObservationReceiptFamily::Fixture,
            observation_receipt_identity: causal_observation_receipt_evidence_identity(
                QueryObservationReceiptFamily::Fixture,
                &fixture_component_identity(outcome.as_str(), "observation_receipt", "fixture"),
            ),
            query_identity: causal_observation_query_evidence_identity(
                "fixture",
                &fixture_component_identity(outcome.as_str(), "query", "fixture"),
            ),
            basis_posture: CausalObservationBasisPosture::Fixture,
            basis_identity: causal_observation_basis_evidence_identity(
                &CausalObservationBasisPosture::Fixture,
                &fixture_component_identity(outcome.as_str(), "basis", "fixture"),
            ),
            inspection_basis,
            result_shape_context: result_shape_handle(
                "fixture_result_shape",
                &fixture_authority,
                "fixture-result-shape",
            ),
            observation_target: target_handle(
                "fixture_target",
                &fixture_authority,
                format!("fixture-target-{}", outcome.as_str()),
            ),
            outcome,
            evidence_identities,
        })
    }
}

#[cfg(test)]
pub(in crate::runtime) fn fixture_inspection_basis(
    outcome: CausalObservationOutcome,
) -> crate::basis_lifecycle::ScopedInspectionBasis {
    let lifecycle = crate::basis_lifecycle::basis_lifecycle();
    match outcome {
        CausalObservationOutcome::BranchPreview => lifecycle.preview("fixture-preview").inspect(),
        CausalObservationOutcome::Replayed => lifecycle
            .historical_snapshot("fixture-history", true)
            .inspect(),
        _ => lifecycle.current_head().inspect(),
    }
    .expect("fixture inspection basis should admit")
}

fn not_executed_snapshot_basis_identity() -> WorthQueryEvidenceIdentity {
    WorthQueryEvidenceIdentity::compose(WorthQueryEvidenceScope::CausalObservationBasis)
        .field_shape(
            WorthQueryEvidenceTag::new("identity_family"),
            "not_executed_snapshot_basis_v1",
        )
        .field_shape(
            WorthQueryEvidenceTag::new("execution_state"),
            "not-executed",
        )
        .seal()
}

fn fixture_authority_identity(label: &str) -> WorthQueryEvidenceIdentity {
    WorthQueryEvidenceIdentity::compose(WorthQueryEvidenceScope::CausalQueryObservationReceipt)
        .field_shape(WorthQueryEvidenceTag::new("role"), "fixture_authority")
        .field_value(WorthQueryEvidenceTag::new("label"), label)
        .seal()
}

fn fixture_component_identity(
    label: &str,
    component: &'static str,
    descriptor: &'static str,
) -> WorthQueryEvidenceIdentity {
    WorthQueryEvidenceIdentity::compose(WorthQueryEvidenceScope::CausalQueryObservationReceipt)
        .field_shape(WorthQueryEvidenceTag::new("role"), "fixture_component")
        .field_shape(WorthQueryEvidenceTag::new("component"), component)
        .field_value(WorthQueryEvidenceTag::new("label"), label)
        .field_value(WorthQueryEvidenceTag::new("descriptor"), descriptor)
        .seal()
}

fn result_shape_handle(
    role: &'static str,
    authority_identity: &WorthQueryEvidenceIdentity,
    descriptor: impl AsRef<str>,
) -> CausalResultShapeContextHandle {
    let identity =
        WorthQueryEvidenceIdentity::compose(WorthQueryEvidenceScope::CausalResultShapeContext)
            .field_shape(WorthQueryEvidenceTag::new("role"), role)
            .field_evidence_identity(WorthQueryEvidenceTag::new("authority"), authority_identity)
            .field_value(
                WorthQueryEvidenceTag::new("descriptor"),
                descriptor.as_ref(),
            )
            .seal();
    CausalResultShapeContextHandle::from_evidence_identity(&identity)
}

fn target_handle(
    role: &'static str,
    authority_identity: &WorthQueryEvidenceIdentity,
    descriptor: impl AsRef<str>,
) -> CausalObservationTargetHandle {
    let identity =
        WorthQueryEvidenceIdentity::compose(WorthQueryEvidenceScope::CausalObservationTarget)
            .field_shape(WorthQueryEvidenceTag::new("role"), role)
            .field_evidence_identity(WorthQueryEvidenceTag::new("authority"), authority_identity)
            .field_value(
                WorthQueryEvidenceTag::new("descriptor"),
                descriptor.as_ref(),
            )
            .seal();
    CausalObservationTargetHandle::from_evidence_identity(&identity)
}
