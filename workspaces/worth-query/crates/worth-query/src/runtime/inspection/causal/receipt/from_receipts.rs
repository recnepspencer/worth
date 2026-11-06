use super::*;
use crate::evidence_identity::WorthQueryEvidenceIdentityDerivationDenial as IdentityDerivationDenial;

impl QueryObservationReceipt {
    pub fn from_write_receipt_inspection(
        inspection: &WorthQueryWriteReceiptInspection,
        inspection_basis: crate::basis_lifecycle::ScopedInspectionBasis,
    ) -> Result<Self, IdentityDerivationDenial> {
        let commit_identity = inspection.commit_identity().evidence_identity();
        let mut evidence_identities = vec![
            CausalObservationEvidenceIdentity::new(
                CausalEvidenceFamily::QueryInspection,
                causal_evidence_reference_identity_digest(
                    CausalEvidenceFamily::QueryInspection,
                    inspection.inspection_identity(),
                ),
            ),
            CausalObservationEvidenceIdentity::new(
                CausalEvidenceFamily::RelationalAuthority,
                causal_evidence_reference_identity_digest(
                    CausalEvidenceFamily::RelationalAuthority,
                    &commit_identity,
                ),
            ),
        ];
        if let Some(causality) = inspection.causality_evidence() {
            let causality_digest = causality.causality_digest().evidence_identity();
            let route_digest = causality.route_digest().evidence_identity();
            let evaluation_surface_digest =
                causality.evaluation_surface_digest().evidence_identity();
            evidence_identities.push(CausalObservationEvidenceIdentity::new(
                CausalEvidenceFamily::QueryMutationCausality,
                causal_evidence_reference_identity_digest(
                    CausalEvidenceFamily::QueryMutationCausality,
                    causality_digest,
                )?,
            ));
            evidence_identities.push(CausalObservationEvidenceIdentity::new(
                CausalEvidenceFamily::BridgeRoute,
                causal_evidence_reference_identity_digest(
                    CausalEvidenceFamily::BridgeRoute,
                    route_digest,
                )?,
            ));
            evidence_identities.push(CausalObservationEvidenceIdentity::new(
                CausalEvidenceFamily::BridgeEvaluation,
                causal_evidence_reference_identity_digest(
                    CausalEvidenceFamily::BridgeEvaluation,
                    evaluation_surface_digest,
                )?,
            ));
        }
        if let Some(provenance) = inspection.provenance_evidence() {
            let feedback_provenance_digest =
                provenance.feedback_provenance_digest().evidence_identity();
            let execution_record_digest = provenance.execution_record_digest().evidence_identity();
            evidence_identities.push(CausalObservationEvidenceIdentity::new(
                CausalEvidenceFamily::QueryMutationProvenance,
                causal_evidence_reference_identity_digest(
                    CausalEvidenceFamily::QueryMutationProvenance,
                    feedback_provenance_digest,
                )?,
            ));
            evidence_identities.push(CausalObservationEvidenceIdentity::new(
                CausalEvidenceFamily::Provenance,
                causal_evidence_reference_identity_digest(
                    CausalEvidenceFamily::Provenance,
                    execution_record_digest,
                )?,
            ));
        }
        let snapshot_identity = inspection.snapshot_identity().evidence_identity()?;
        let target_identity = inspection
            .target_entity_identity()
            .or_else(|| inspection.declared_entity_identity())
            .map(|identity| identity.evidence_identity());
        let fallback_target_identity = target_handle(
            "write_receipt_unspecified_target",
            inspection.inspection_identity(),
            "mutation-target-unspecified",
        )?;
        Ok(Self::from_parts(ObservationReceiptParts {
            family: QueryObservationReceiptFamily::WriteReceipt,
            observation_receipt_identity: causal_observation_receipt_evidence_identity(
                QueryObservationReceiptFamily::WriteReceipt,
                inspection.inspection_identity(),
            )?,
            query_identity: write_observation_query_identity(inspection)?,
            basis_posture: CausalObservationBasisPosture::AuthorityLane(inspection.basis_lane()),
            basis_identity: causal_observation_basis_evidence_identity(
                &CausalObservationBasisPosture::AuthorityLane(inspection.basis_lane()),
                &snapshot_identity,
            )?,
            inspection_basis,
            result_shape_context: result_shape_handle(
                "write_receipt_mutation_family",
                inspection.inspection_identity(),
                inspection.mutation_family(),
            )?,
            observation_target: target_identity
                .as_ref()
                .map(CausalObservationTargetHandle::from_evidence_identity)
                .unwrap_or(fallback_target_identity),
            outcome: CausalObservationOutcome::Changed,
            evidence_identities,
        })?)
    }

    pub fn from_intent_receipt_inspection(
        inspection: &WorthQueryIntentReceiptInspection,
        inspection_basis: crate::basis_lifecycle::ScopedInspectionBasis,
    ) -> Result<Self, IdentityDerivationDenial> {
        let snapshot_identity = inspection.snapshot_identity().evidence_identity()?;
        let commit_identity = inspection.commit_identity().evidence_identity();
        let outcome = if inspection.produced_mutation_digest().is_some() {
            CausalObservationOutcome::Changed
        } else {
            CausalObservationOutcome::Suppressed
        };
        Ok(Self::from_parts(ObservationReceiptParts {
            family: QueryObservationReceiptFamily::IntentReceipt,
            observation_receipt_identity: causal_observation_receipt_evidence_identity(
                QueryObservationReceiptFamily::IntentReceipt,
                inspection.receipt_identity(),
            )?,
            query_identity: causal_observation_query_evidence_identity(
                "intent_receipt",
                inspection.receipt_identity(),
            )?,
            basis_posture: CausalObservationBasisPosture::AuthorityLane(inspection.target_lane()),
            basis_identity: causal_observation_basis_evidence_identity(
                &CausalObservationBasisPosture::AuthorityLane(inspection.target_lane()),
                &snapshot_identity,
            )?,
            inspection_basis,
            result_shape_context: result_shape_handle(
                "intent_receipt_strategy",
                inspection.receipt_identity(),
                inspection.strategy_descriptor_digest(),
            )?,
            observation_target: target_handle(
                "intent_receipt_intent",
                inspection.receipt_identity(),
                inspection.intent_name(),
            )?,
            outcome,
            evidence_identities: vec![
                CausalObservationEvidenceIdentity::new(
                    CausalEvidenceFamily::QueryInspection,
                    causal_evidence_reference_identity_digest(
                        CausalEvidenceFamily::QueryInspection,
                        inspection.inspection_identity(),
                    ),
                ),
                CausalObservationEvidenceIdentity::new(
                    CausalEvidenceFamily::RelationalAuthority,
                    causal_evidence_reference_identity_digest(
                        CausalEvidenceFamily::RelationalAuthority,
                        &commit_identity,
                    ),
                ),
            ],
        })?)
    }

    pub fn from_intent_denial_inspection(
        inspection: &WorthQueryIntentDenialInspection,
        inspection_basis: crate::basis_lifecycle::ScopedInspectionBasis,
    ) -> Result<Self, IdentityDerivationDenial> {
        let snapshot_basis_identity = inspection
            .snapshot_evidence_identity()
            .unwrap_or_else(not_executed_snapshot_basis_identity);
        Ok(Self::from_parts(ObservationReceiptParts {
            family: QueryObservationReceiptFamily::IntentDenial,
            observation_receipt_identity: causal_observation_receipt_evidence_identity(
                QueryObservationReceiptFamily::IntentDenial,
                inspection.denial_identity(),
            )?,
            query_identity: causal_observation_query_evidence_identity(
                "intent_denial",
                inspection.denial_identity(),
            )?,
            basis_posture: CausalObservationBasisPosture::AuthorityLane(inspection.target_lane()),
            basis_identity: causal_observation_basis_evidence_identity(
                &CausalObservationBasisPosture::AuthorityLane(inspection.target_lane()),
                &snapshot_basis_identity,
            )?,
            inspection_basis,
            result_shape_context: result_shape_handle(
                "intent_denial_stage",
                inspection.denial_identity(),
                inspection.stage(),
            )?,
            observation_target: target_handle(
                "intent_denial_intent",
                inspection.denial_identity(),
                inspection.intent_name(),
            )?,
            outcome: CausalObservationOutcome::Denied,
            evidence_identities: vec![CausalObservationEvidenceIdentity::new(
                CausalEvidenceFamily::QueryInspection,
                causal_evidence_reference_identity_digest(
                    CausalEvidenceFamily::QueryInspection,
                    inspection.inspection_identity(),
                ),
            )],
        })?)
    }

    pub fn from_branch_intent_receipt_inspection(
        inspection: &WorthQueryBranchIntentReceiptInspection,
        inspection_basis: crate::basis_lifecycle::ScopedInspectionBasis,
    ) -> Result<Self, IdentityDerivationDenial> {
        Ok(Self::from_parts(ObservationReceiptParts {
            family: QueryObservationReceiptFamily::BranchIntentReceipt,
            observation_receipt_identity: causal_observation_receipt_evidence_identity(
                QueryObservationReceiptFamily::BranchIntentReceipt,
                inspection.receipt_identity(),
            )?,
            query_identity: causal_observation_query_evidence_identity(
                "branch_intent_receipt",
                inspection.receipt_identity(),
            )?,
            basis_posture: CausalObservationBasisPosture::AuthorityLane(inspection.target_lane()),
            basis_identity: causal_observation_basis_evidence_identity(
                &CausalObservationBasisPosture::AuthorityLane(inspection.target_lane()),
                inspection.basis_identity(),
            )?,
            inspection_basis,
            result_shape_context: result_shape_handle(
                "branch_intent_admission",
                inspection.receipt_identity(),
                inspection.admission_digest(),
            )?,
            observation_target: target_handle(
                "branch_intent_intent",
                inspection.receipt_identity(),
                inspection.intent_name(),
            )?,
            outcome: CausalObservationOutcome::BranchPreview,
            evidence_identities: vec![
                CausalObservationEvidenceIdentity::new(
                    CausalEvidenceFamily::QueryInspection,
                    causal_evidence_reference_identity_digest(
                        CausalEvidenceFamily::QueryInspection,
                        inspection.inspection_identity(),
                    ),
                ),
                CausalObservationEvidenceIdentity::new(
                    CausalEvidenceFamily::BridgePreview,
                    causal_evidence_reference_identity_digest(
                        CausalEvidenceFamily::BridgePreview,
                        inspection.admission_identity(),
                    ),
                ),
            ],
        })?)
    }

    pub fn from_preview_outcome_inspection(
        inspection: &WorthQueryPreviewOutcomeInspection,
        inspection_basis: crate::basis_lifecycle::ScopedInspectionBasis,
    ) -> Result<Self, IdentityDerivationDenial> {
        Ok(Self::from_parts(ObservationReceiptParts {
            family: QueryObservationReceiptFamily::PreviewOutcome,
            observation_receipt_identity: causal_observation_receipt_evidence_identity(
                QueryObservationReceiptFamily::PreviewOutcome,
                inspection.closeout_identity(),
            )?,
            query_identity: causal_observation_query_evidence_identity(
                "preview_outcome",
                inspection.session_label().identity_digest(),
            )?,
            basis_posture: CausalObservationBasisPosture::AuthorityLane(inspection.target_lane()),
            basis_identity: causal_observation_basis_evidence_identity(
                &CausalObservationBasisPosture::AuthorityLane(inspection.target_lane()),
                inspection.basis_identity(),
            )?,
            inspection_basis,
            result_shape_context: result_shape_handle(
                "preview_outcome_residue",
                inspection.closeout_identity(),
                inspection.residue_digest(),
            )?,
            observation_target: target_handle(
                "preview_outcome_session",
                inspection.closeout_identity(),
                inspection.session_label().display(),
            )?,
            outcome: CausalObservationOutcome::BranchPreview,
            evidence_identities: vec![
                CausalObservationEvidenceIdentity::new(
                    CausalEvidenceFamily::QueryInspection,
                    causal_evidence_reference_identity_digest(
                        CausalEvidenceFamily::QueryInspection,
                        inspection.inspection_identity(),
                    ),
                ),
                CausalObservationEvidenceIdentity::new(
                    CausalEvidenceFamily::BridgePreview,
                    causal_evidence_reference_identity_digest(
                        CausalEvidenceFamily::BridgePreview,
                        inspection.closeout_identity(),
                    ),
                ),
            ],
        })?)
    }
}
