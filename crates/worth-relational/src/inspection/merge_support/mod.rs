use worth_relational_contracts::internal_port::merge::data::artifacts::correspondence_witness_rows::RelationalMergeCorrespondenceWitnessRowOwnerPort as _;
use worth_relational_contracts::internal_port::transactions::data::outcomes::plan_artifacts::MergeExecutionSummaryOwnerPort as _;
use worth_relational_contracts::internal_port::transactions::data::outcomes::plan_artifacts::PublishedMergeExecutionAuthorityOwnerPort as _;
use worth_foundational::{
    admit_requested_foundational_profile, attach_support_profiled_artifact,
    foundational_profile_progression_authority, request_foundational_profile_set,
    AdmissionReadinessProfile, CertificationPostureProfile, CompatibilityPostureProfile,
    DiagnosticRichnessProfile, ExecutionObjectiveProfile, FoundationalProfileAttachmentDenial,
    FoundationalProfileProgressionDenial, FoundationalProfileSet, FoundationalProfileSetInput,
    ObservationActivationProfile, RetentionDeliveryProfile, SupportPostureProfile,
};
use worth_proof::TransitionOutcome;

use crate::inspection::access::InspectionAccess;
use crate::inspection::data::{
    RelationalMergeSupportInspectionAbsenceKind,
    RelationalMergeSupportInspectionCompatibilityPosture, RelationalMergeSupportInspectionDenial,
    RelationalMergeSupportInspectionInput, RelationalMergeSupportInspectionRow,
    RelationalMergeSupportInspectionSurface, RelationalMergeSupportInspectionWitness,
};
use crate::transactions::data::{MergeExecutionSummary, PublishedMergeExecutionAuthority};

impl<'runtime> InspectionAccess<'runtime> {
    pub fn prepare_merge_support_inspection_witness(
        &self,
        execution_summary: &MergeExecutionSummary,
    ) -> Result<RelationalMergeSupportInspectionWitness, RelationalMergeSupportInspectionDenial>
    {
        if !execution_summary.retains_consistent_proof_packet_authority() {
            return Err(RelationalMergeSupportInspectionDenial::InconsistentRetainedProofAuthority);
        }
        support_inspection_witness(RelationalMergeSupportInspectionInput {
            request: execution_summary.request.clone(),
            branch_basis: execution_summary.branch_basis.clone(),
            proof_packet: Some(execution_summary.proof_packet.clone()),
            correspondence_witness: Some(execution_summary.correspondence_witness.clone()),
            schema_reconciliation_witness: Some(
                execution_summary.schema_reconciliation_witness.clone(),
            ),
            strategy_witness: Some(execution_summary.strategy_witness.clone()),
        })
    }

    pub fn prepare_published_merge_support_inspection_witness(
        &self,
        authority: &PublishedMergeExecutionAuthority,
    ) -> Result<RelationalMergeSupportInspectionWitness, RelationalMergeSupportInspectionDenial>
    {
        if !authority.retains_consistent_proof_packet_authority() {
            return Err(RelationalMergeSupportInspectionDenial::InconsistentRetainedProofAuthority);
        }
        self.prepare_merge_support_inspection_witness(&authority.execution_summary)
    }
}

pub(crate) fn support_inspection_witness(
    input: RelationalMergeSupportInspectionInput,
) -> Result<RelationalMergeSupportInspectionWitness, RelationalMergeSupportInspectionDenial> {
    let requested = support_profile().map_err(|error| {
        RelationalMergeSupportInspectionDenial::IllegalSupportProfile(format!("{error:?}"))
    })?;
    let admitted = match admit_requested_foundational_profile(
        request_foundational_profile_set(requested),
        requested,
        None,
        foundational_profile_progression_authority(),
    ) {
        TransitionOutcome::Success(admitted) => admitted,
        TransitionOutcome::Denied(FoundationalProfileProgressionDenial::MissingExplicitNarrowingRecord)
        | TransitionOutcome::Denied(
            FoundationalProfileProgressionDenial::NarrowingRecordKindMismatch,
        )
        | TransitionOutcome::Denied(
            FoundationalProfileProgressionDenial::RequestedAndAdmittedProfilesMayDifferInOnlyOneFamily,
        )
        | TransitionOutcome::Denied(
            FoundationalProfileProgressionDenial::RequestedAndAdmittedProfilesMayOnlyNarrow,
        )
        | TransitionOutcome::Denied(
            FoundationalProfileProgressionDenial::AdmissionReadinessCannotChangeAcrossProfileProgression,
        )
        | TransitionOutcome::Denied(
            FoundationalProfileProgressionDenial::ResolutionLedgerDoesNotMatchProfileChange,
        )
        | TransitionOutcome::Denied(
            FoundationalProfileProgressionDenial::ResolutionRelationMismatch(_),
        ) => return Err(RelationalMergeSupportInspectionDenial::RequestedProfileAdmissionDenied),
        TransitionOutcome::Deferred(_) => {
            return Err(RelationalMergeSupportInspectionDenial::RequestedProfileAdmissionDeferred);
        }
        TransitionOutcome::Stale(_) => {
            return Err(RelationalMergeSupportInspectionDenial::RequestedProfileAdmissionStale);
        }
        TransitionOutcome::RebindRequired(_) => {
            return Err(
                RelationalMergeSupportInspectionDenial::RequestedProfileAdmissionRebindRequired,
            );
        }
        TransitionOutcome::Failed(_) => {
            return Err(RelationalMergeSupportInspectionDenial::RequestedProfileAdmissionFailed);
        }
    };

    let rows = rows_for_input(&input);
    let surface = RelationalMergeSupportInspectionSurface::retained(
        input.request.request_digest().to_string(),
        input.branch_basis.basis_digest(),
        std::sync::Arc::from(rows),
    );
    let support_artifact = match attach_support_profiled_artifact(
        admitted,
        requested,
        None,
        surface,
        foundational_profile_progression_authority(),
    ) {
        TransitionOutcome::Success(artifact) => artifact,
        TransitionOutcome::Denied(
            FoundationalProfileAttachmentDenial::ProgressionDenied(_)
            | FoundationalProfileAttachmentDenial::SupportArtifactsCannotCarryInternalOnlySupportPosture
            | FoundationalProfileAttachmentDenial::ProofBearingArtifactsRequireAdmittedReadiness,
        ) => return Err(RelationalMergeSupportInspectionDenial::SupportAttachmentDenied),
        TransitionOutcome::Deferred(_) => {
            return Err(RelationalMergeSupportInspectionDenial::SupportAttachmentDeferred);
        }
        TransitionOutcome::Stale(_) => {
            return Err(RelationalMergeSupportInspectionDenial::SupportAttachmentStale);
        }
        TransitionOutcome::RebindRequired(_) => {
            return Err(RelationalMergeSupportInspectionDenial::SupportAttachmentRebindRequired);
        }
        TransitionOutcome::Failed(_) => {
            return Err(RelationalMergeSupportInspectionDenial::SupportAttachmentFailed);
        }
    };

    Ok(RelationalMergeSupportInspectionWitness::retained(
        support_artifact,
    ))
}

fn support_profile(
) -> Result<FoundationalProfileSet, worth_foundational::FoundationalProfileCompositionDenial> {
    FoundationalProfileSet::new(FoundationalProfileSetInput {
        diagnostic_richness: DiagnosticRichnessProfile::Forensic,
        support_posture: SupportPostureProfile::SupportReady,
        compatibility_posture: CompatibilityPostureProfile::CompatibilityRequired,
        admission_readiness: AdmissionReadinessProfile::Admitted,
        retention_delivery: RetentionDeliveryProfile::Retained,
        certification_posture: CertificationPostureProfile::Uncertified,
        execution_objective: ExecutionObjectiveProfile::Balanced,
        observation_activation: ObservationActivationProfile::Continuous,
    })
}

fn rows_for_input(
    input: &RelationalMergeSupportInspectionInput,
) -> Vec<RelationalMergeSupportInspectionRow> {
    vec![
        RelationalMergeSupportInspectionRow::branch_basis(
            input.branch_basis.basis_digest(),
            input.branch_basis.source_head().commit_id,
            input.branch_basis.target_head().commit_id,
            input.branch_basis.merge_base().commit().commit_id,
        ),
        request_admission_row(input),
        correspondence_row(input),
        schema_row(input),
        strategy_row(input),
        RelationalMergeSupportInspectionRow::compatibility(
            RelationalMergeSupportInspectionCompatibilityPosture::UnavailablePhaseDependency,
            Some(
                RelationalMergeSupportInspectionAbsenceKind::MissingCompatibilityWitnessPhaseDependency,
            ),
        ),
    ]
}

fn request_admission_row(
    input: &RelationalMergeSupportInspectionInput,
) -> RelationalMergeSupportInspectionRow {
    let proof_packet = input.proof_packet.as_ref();
    RelationalMergeSupportInspectionRow::request_admission(
        input.request.request_digest().to_string(),
        proof_packet.map(|packet| packet.packet_digest().to_string()),
        proof_packet.map(|packet| packet.admission_posture()),
        proof_packet
            .is_none()
            .then_some(RelationalMergeSupportInspectionAbsenceKind::MissingProofPacket),
    )
}

fn correspondence_row(
    input: &RelationalMergeSupportInspectionInput,
) -> RelationalMergeSupportInspectionRow {
    let Some(witness) = input.correspondence_witness.as_ref() else {
        return RelationalMergeSupportInspectionRow::correspondence(
            None,
            0,
            0,
            0,
            None,
            None,
            None,
            Some(RelationalMergeSupportInspectionAbsenceKind::MissingCorrespondenceWitness),
        );
    };
    let admitted_count = witness.admitted_rows().count();
    let denied_count = witness
        .rows()
        .iter()
        .filter(|row| {
            !matches!(
                row.posture(),
                crate::facade::merge::RelationalMergeCorrespondenceWitnessPosture::Admitted
                    | crate::facade::merge::RelationalMergeCorrespondenceWitnessPosture::UnavailableMissingTarget
            )
        })
        .count();
    let unavailable_count = witness
        .rows()
        .iter()
        .filter(|row| {
            matches!(
                row.posture(),
                crate::facade::merge::RelationalMergeCorrespondenceWitnessPosture::UnavailableMissingTarget
            )
        })
        .count();
    let sample = witness.rows().first().map(|row| row.candidate());
    RelationalMergeSupportInspectionRow::correspondence(
        Some(witness.witness_digest().to_string()),
        admitted_count,
        denied_count,
        unavailable_count,
        sample
            .as_ref()
            .map(|candidate| candidate.source_record.clone()),
        sample
            .as_ref()
            .and_then(|candidate| candidate.target_record.clone()),
        witness.rows().first().map(|row| row.posture()),
        None,
    )
}

fn schema_row(
    input: &RelationalMergeSupportInspectionInput,
) -> RelationalMergeSupportInspectionRow {
    let Some(witness) = input.schema_reconciliation_witness.as_ref() else {
        return RelationalMergeSupportInspectionRow::schema(
            None,
            0,
            0,
            None,
            None,
            None,
            None,
            Some(RelationalMergeSupportInspectionAbsenceKind::MissingSchemaReconciliationWitness),
        );
    };
    let reconciled_count = witness
        .rows()
        .iter()
        .filter(|row| {
            row.posture()
                == crate::facade::merge::RelationalSchemaReconciliationWitnessPosture::Reconciled
        })
        .count();
    let denied_count = witness.rows().len() - reconciled_count;
    let sample = witness.rows().first();
    RelationalMergeSupportInspectionRow::schema(
        Some(witness.witness_digest().to_string()),
        reconciled_count,
        denied_count,
        sample.map(|row| row.record().clone()),
        sample.and_then(|row| row.target_record().cloned()),
        sample.map(|row| row.posture()),
        sample.map(|row| row.decision_boundary()),
        None,
    )
}

fn strategy_row(
    input: &RelationalMergeSupportInspectionInput,
) -> RelationalMergeSupportInspectionRow {
    let Some(witness) = input.strategy_witness.as_ref() else {
        return RelationalMergeSupportInspectionRow::strategy(
            None,
            0,
            0,
            0,
            Some(RelationalMergeSupportInspectionAbsenceKind::MissingStrategyWitness),
        );
    };
    RelationalMergeSupportInspectionRow::strategy(
        Some(witness.witness_digest().to_string()),
        witness.aspect_policy_rows().len(),
        witness.topology_rows().len(),
        witness.deletion_rows().len(),
        None,
    )
}
