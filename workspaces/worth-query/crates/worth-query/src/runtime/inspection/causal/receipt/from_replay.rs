use super::*;
use crate::evidence_identity::WorthQueryEvidenceIdentityDerivationDenial as IdentityDerivationDenial;

impl QueryObservationReceipt {
    pub fn from_read_receipt(
        receipt: &WorthQueryReadReceipt,
        inspection_basis: crate::basis_lifecycle::ScopedInspectionBasis,
    ) -> Result<Self, IdentityDerivationDenial> {
        let snapshot_evidence_identity = receipt.snapshot_evidence_identity();
        Ok(Self::from_parts(ObservationReceiptParts {
            family: QueryObservationReceiptFamily::ReadReceipt,
            observation_receipt_identity: read_observation_receipt_identity(receipt)?,
            query_identity: read_observation_query_identity(receipt, &snapshot_evidence_identity)?,
            basis_posture: CausalObservationBasisPosture::ReadExecution(
                receipt.execution_engine().clone(),
            ),
            basis_identity: causal_observation_basis_evidence_identity(
                &CausalObservationBasisPosture::ReadExecution(receipt.execution_engine().clone()),
                &snapshot_evidence_identity,
            )?,
            inspection_basis,
            result_shape_context: result_shape_handle(
                "read_receipt_graph",
                &snapshot_evidence_identity,
                receipt.read_graph_digest(),
            )?,
            observation_target: target_handle(
                "read_receipt_result",
                &snapshot_evidence_identity,
                receipt.result_digest(),
            )?,
            outcome: CausalObservationOutcome::Replayed,
            evidence_identities: vec![CausalObservationEvidenceIdentity::new(
                CausalEvidenceFamily::QueryInspection,
                read_observation_result_reference_digest(receipt, &snapshot_evidence_identity),
            )],
        }))
    }

    pub(crate) fn certification_historical_replay_fixture(
        label: &str,
    ) -> Result<Self, IdentityDerivationDenial> {
        let inspection_basis = crate::basis_lifecycle::basis_lifecycle()
            .historical_snapshot(format!("fixture-inspection:{label}"), true)
            .inspect()
            .expect("certification historical inspection basis should admit");
        let fixture_authority = fixture_authority_identity(label)?;
        Ok(Self::from_parts(ObservationReceiptParts {
            family: QueryObservationReceiptFamily::Fixture,
            observation_receipt_identity: causal_observation_receipt_evidence_identity(
                QueryObservationReceiptFamily::Fixture,
                &fixture_component_identity(label, "observation_receipt", "replayed")?,
            )?,
            query_identity: causal_observation_query_evidence_identity(
                "fixture_replayed",
                &fixture_component_identity(label, "query", "historical_replay")?,
            )?,
            basis_posture: CausalObservationBasisPosture::HistoricalReplayCertification,
            basis_identity: causal_observation_basis_evidence_identity(
                &CausalObservationBasisPosture::HistoricalReplayCertification,
                &fixture_component_identity(label, "basis", "historical_replay_certification")?,
            )?,
            inspection_basis,
            result_shape_context: result_shape_handle(
                "fixture_result_shape",
                &fixture_authority,
                format!("fixture-result-shape:{label}"),
            )?,
            observation_target: target_handle(
                "fixture_target",
                &fixture_authority,
                format!("fixture-target:{label}"),
            )?,
            outcome: CausalObservationOutcome::Replayed,
            evidence_identities: vec![CausalObservationEvidenceIdentity::new(
                CausalEvidenceFamily::QueryInspection,
                causal_evidence_reference_identity_digest(
                    CausalEvidenceFamily::QueryInspection,
                    &fixture_component_identity(label, "evidence_reference", "query_inspection"),
                ),
            )],
        }))
    }
}
