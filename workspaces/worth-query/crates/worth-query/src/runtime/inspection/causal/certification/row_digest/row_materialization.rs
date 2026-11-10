use super::*;
use crate::evidence_identity::WorthQueryEvidenceIdentityDerivationDenial as IdentityDerivationDenial;

impl CausalInspectionRepresentativeRowDigestSet {
    pub(in crate::runtime::inspection::causal::certification) fn from_query_artifact(
        kind: CausalInspectionRepresentativeKind,
        artifact: &QueryCausalInspectionArtifact,
    ) -> Result<Self, IdentityDerivationDenial> {
        let query_observation_receipt_digest =
            artifact.query_observation_identity().as_str().to_string();
        let causal_observation_anchor_digest =
            RepresentativeCausalObservationAnchorDigest::from_digest(
                artifact.causal_identity().as_str(),
            );
        let query_digest = representative_query_digest(
            &query_observation_receipt_digest,
            causal_observation_anchor_digest.as_str(),
        )?;
        let inspection_digest = inspection_digest(artifact).to_string();
        let evidence_reference_collection_digest =
            evidence_reference_collection_digest(artifact, kind)?;
        let slots = named_evidence_slots(artifact)?;
        let policy_digest = artifact_policy_digest(artifact).to_string();
        let receipt_digest = artifact_receipt_digest(artifact).to_string();
        let counter_snapshot_digest = artifact
            .performance()
            .performance_for_reporting()
            .to_string();
        let row_digest = row_digest(RowDigestParts {
            kind,
            query_digest: &query_digest,
            query_observation_receipt_digest: &query_observation_receipt_digest,
            causal_observation_anchor_digest: causal_observation_anchor_digest.as_str(),
            inspection_digest: Some(&inspection_digest),
            artifact_digest: Some(artifact.artifact_identity().as_str()),
            causal_envelope_digest: artifact.bridge_envelope_for_reporting(),
            evidence_reference_collection_digest: Some(&evidence_reference_collection_digest),
            relational_authority_digest: slots.relational_authority_digest.as_deref(),
            bridge_route_digest: slots.bridge_route_digest.as_deref(),
            bridge_evaluation_digest: slots.bridge_evaluation_digest.as_deref(),
            bridge_source_materialization_digest: slots
                .bridge_source_materialization_digest
                .as_deref(),
            bridge_structural_digest: slots.bridge_structural_digest.as_deref(),
            bridge_stream_digest: slots.bridge_stream_digest.as_deref(),
            bridge_preview_digest: slots.bridge_preview_digest.as_deref(),
            bridge_writeback_digest: slots.bridge_writeback_digest.as_deref(),
            bridge_replay_digest: slots.bridge_replay_digest.as_deref(),
            signal_invalidation_digest: slots.signal_invalidation_digest.as_deref(),
            signal_evaluation_digest: slots.signal_evaluation_digest.as_deref(),
            signal_forensic_availability_digest: slots
                .signal_forensic_availability_digest
                .as_deref(),
            signal_replay_cursor_digest: slots.signal_replay_cursor_digest.as_deref(),
            signal_lineage_digest: slots.signal_lineage_digest.as_deref(),
            signal_provenance_digest: slots.signal_provenance_digest.as_deref(),
            replay_posture_digest: slots.replay_posture_digest.as_deref(),
            materialization_policy_digest: Some(&policy_digest),
            redaction_policy_digest: Some(&policy_digest),
            materialization_receipt_digest: Some(&receipt_digest),
            counter_snapshot_digest: Some(&counter_snapshot_digest),
            failure_digest: None,
        })?;
        Ok(Self {
            kind,
            query_digest,
            query_observation_receipt_digest,
            causal_observation_anchor_digest,
            inspection_digest: Some(inspection_digest),
            artifact_digest: Some(artifact.artifact_identity().as_str().to_string()),
            causal_envelope_digest: artifact.bridge_envelope_for_reporting().map(str::to_string),
            evidence_reference_collection_digest: Some(evidence_reference_collection_digest),
            relational_authority_digest: slots.relational_authority_digest,
            bridge_route_digest: slots.bridge_route_digest,
            bridge_evaluation_digest: slots.bridge_evaluation_digest,
            bridge_source_materialization_digest: slots.bridge_source_materialization_digest,
            bridge_structural_digest: slots.bridge_structural_digest,
            bridge_stream_digest: slots.bridge_stream_digest,
            bridge_preview_digest: slots.bridge_preview_digest,
            bridge_writeback_digest: slots.bridge_writeback_digest,
            bridge_replay_digest: slots.bridge_replay_digest,
            signal_invalidation_digest: slots.signal_invalidation_digest,
            signal_evaluation_digest: slots.signal_evaluation_digest,
            signal_forensic_availability_digest: slots.signal_forensic_availability_digest,
            signal_replay_cursor_digest: slots.signal_replay_cursor_digest,
            signal_lineage_digest: slots.signal_lineage_digest,
            signal_provenance_digest: slots.signal_provenance_digest,
            replay_posture_digest: slots.replay_posture_digest,
            materialization_policy_digest: Some(policy_digest.clone()),
            redaction_policy_digest: Some(policy_digest),
            materialization_receipt_digest: Some(receipt_digest),
            counter_snapshot_digest: Some(counter_snapshot_digest),
            failure_digest: None,
            row_digest,
        })
    }

    pub(in crate::runtime::inspection::causal::certification) fn from_missing_evidence(
        kind: CausalInspectionRepresentativeKind,
        family: CausalEvidenceFamily,
        failure_digest: &str,
    ) -> Result<Self, IdentityDerivationDenial> {
        Self::from_failure(kind, family.as_str(), failure_digest)
    }

    pub(in crate::runtime::inspection::causal::certification) fn from_failure(
        kind: CausalInspectionRepresentativeKind,
        failure_class: &str,
        failure_digest: &str,
    ) -> Result<Self, IdentityDerivationDenial> {
        let query_observation_receipt_digest = "denied-before-materialization".to_string();
        let causal_observation_anchor_digest =
            RepresentativeCausalObservationAnchorDigest::from_digest(
                WorthQueryEvidenceIdentity::compose(
                    WorthQueryEvidenceScope::CausalInspectionCertificationFailureEvidence,
                )
                .field_shape(
                    WorthQueryEvidenceTag::new("identity_family"),
                    "causal_inspection_failure_anchor_v1",
                )
                .field_shape(WorthQueryEvidenceTag::new("kind"), kind.as_str())
                .field_shape(WorthQueryEvidenceTag::new("failure_class"), failure_class)
                .field_value(WorthQueryEvidenceTag::new("failure"), failure_digest)
                .seal()?
                .as_str()
                .to_string(),
            );
        let query_digest = representative_query_digest(
            &query_observation_receipt_digest,
            causal_observation_anchor_digest.as_str(),
        )?;
        let counter_snapshot_digest = WorthQueryEvidenceIdentity::compose(
            WorthQueryEvidenceScope::CausalInspectionCertificationFailureEvidence,
        )
        .field_shape(
            WorthQueryEvidenceTag::new("identity_family"),
            "causal_inspection_failure_counter_snapshot_v1",
        )
        .field_shape(WorthQueryEvidenceTag::new("failure_class"), failure_class)
        .field_usize(WorthQueryEvidenceTag::new("anchor"), 1)
        .field_usize(WorthQueryEvidenceTag::new("reference_resolution"), 1)
        .field_usize(WorthQueryEvidenceTag::new("admission"), 0)
        .field_usize(WorthQueryEvidenceTag::new("materialization"), 0)
        .seal()?
        .as_str()
        .to_string();
        let row_digest = row_digest(RowDigestParts {
            kind,
            query_digest: &query_digest,
            query_observation_receipt_digest: &query_observation_receipt_digest,
            causal_observation_anchor_digest: causal_observation_anchor_digest.as_str(),
            inspection_digest: None,
            artifact_digest: None,
            causal_envelope_digest: None,
            evidence_reference_collection_digest: None,
            relational_authority_digest: None,
            bridge_route_digest: None,
            bridge_evaluation_digest: None,
            bridge_source_materialization_digest: None,
            bridge_structural_digest: None,
            bridge_stream_digest: None,
            bridge_preview_digest: None,
            bridge_writeback_digest: None,
            bridge_replay_digest: None,
            signal_invalidation_digest: None,
            signal_evaluation_digest: None,
            signal_forensic_availability_digest: None,
            signal_replay_cursor_digest: None,
            signal_lineage_digest: None,
            signal_provenance_digest: None,
            replay_posture_digest: None,
            materialization_policy_digest: None,
            redaction_policy_digest: None,
            materialization_receipt_digest: None,
            counter_snapshot_digest: Some(&counter_snapshot_digest),
            failure_digest: Some(failure_digest),
        })?;
        Ok(Self {
            kind,
            query_digest,
            query_observation_receipt_digest,
            causal_observation_anchor_digest,
            inspection_digest: None,
            artifact_digest: None,
            causal_envelope_digest: None,
            evidence_reference_collection_digest: None,
            relational_authority_digest: None,
            bridge_route_digest: None,
            bridge_evaluation_digest: None,
            bridge_source_materialization_digest: None,
            bridge_structural_digest: None,
            bridge_stream_digest: None,
            bridge_preview_digest: None,
            bridge_writeback_digest: None,
            bridge_replay_digest: None,
            signal_invalidation_digest: None,
            signal_evaluation_digest: None,
            signal_forensic_availability_digest: None,
            signal_replay_cursor_digest: None,
            signal_lineage_digest: None,
            signal_provenance_digest: None,
            replay_posture_digest: None,
            materialization_policy_digest: None,
            redaction_policy_digest: None,
            materialization_receipt_digest: None,
            counter_snapshot_digest: Some(counter_snapshot_digest),
            failure_digest: Some(failure_digest.to_string()),
            row_digest,
        })
    }

    pub fn kind(&self) -> CausalInspectionRepresentativeKind {
        self.kind
    }

    pub fn row_digest(&self) -> &str {
        &self.row_digest
    }

    pub fn query_digest(&self) -> &str {
        &self.query_digest
    }

    pub fn query_observation_receipt_digest(&self) -> &str {
        &self.query_observation_receipt_digest
    }

    pub fn causal_observation_anchor_digest(&self) -> &str {
        self.causal_observation_anchor_digest.as_str()
    }

    pub fn inspection_digest(&self) -> Option<&str> {
        self.inspection_digest.as_deref()
    }

    pub fn artifact_digest(&self) -> Option<&str> {
        self.artifact_digest.as_deref()
    }

    pub fn causal_envelope_digest(&self) -> Option<&str> {
        self.causal_envelope_digest.as_deref()
    }

    pub fn evidence_reference_collection_digest(&self) -> Option<&str> {
        self.evidence_reference_collection_digest.as_deref()
    }

    pub fn bridge_route_digest(&self) -> Option<&str> {
        self.bridge_route_digest.as_deref()
    }

    pub fn bridge_evaluation_digest(&self) -> Option<&str> {
        self.bridge_evaluation_digest.as_deref()
    }

    pub fn bridge_source_materialization_digest(&self) -> Option<&str> {
        self.bridge_source_materialization_digest.as_deref()
    }

    pub fn bridge_structural_digest(&self) -> Option<&str> {
        self.bridge_structural_digest.as_deref()
    }

    pub fn bridge_stream_digest(&self) -> Option<&str> {
        self.bridge_stream_digest.as_deref()
    }

    pub fn bridge_preview_digest(&self) -> Option<&str> {
        self.bridge_preview_digest.as_deref()
    }

    pub fn bridge_writeback_digest(&self) -> Option<&str> {
        self.bridge_writeback_digest.as_deref()
    }

    pub fn bridge_replay_digest(&self) -> Option<&str> {
        self.bridge_replay_digest.as_deref()
    }

    pub fn signal_invalidation_digest(&self) -> Option<&str> {
        self.signal_invalidation_digest.as_deref()
    }

    pub fn signal_evaluation_digest(&self) -> Option<&str> {
        self.signal_evaluation_digest.as_deref()
    }

    pub fn signal_forensic_availability_digest(&self) -> Option<&str> {
        self.signal_forensic_availability_digest.as_deref()
    }

    pub fn signal_replay_cursor_digest(&self) -> Option<&str> {
        self.signal_replay_cursor_digest.as_deref()
    }

    pub fn signal_lineage_digest(&self) -> Option<&str> {
        self.signal_lineage_digest.as_deref()
    }

    pub fn signal_provenance_digest(&self) -> Option<&str> {
        self.signal_provenance_digest.as_deref()
    }

    pub fn replay_posture_digest(&self) -> Option<&str> {
        self.replay_posture_digest.as_deref()
    }

    pub fn relational_authority_digest(&self) -> Option<&str> {
        self.relational_authority_digest.as_deref()
    }

    pub fn populated_named_evidence_slot_count(&self) -> usize {
        [
            self.relational_authority_digest(),
            self.bridge_route_digest(),
            self.bridge_evaluation_digest(),
            self.bridge_source_materialization_digest(),
            self.bridge_structural_digest(),
            self.bridge_stream_digest(),
            self.bridge_preview_digest(),
            self.bridge_writeback_digest(),
            self.bridge_replay_digest(),
            self.signal_invalidation_digest(),
            self.signal_evaluation_digest(),
            self.signal_forensic_availability_digest(),
            self.signal_replay_cursor_digest(),
            self.signal_lineage_digest(),
            self.signal_provenance_digest(),
            self.replay_posture_digest(),
        ]
        .into_iter()
        .flatten()
        .count()
    }

    pub fn materialization_policy_digest(&self) -> Option<&str> {
        self.materialization_policy_digest.as_deref()
    }

    pub fn redaction_policy_digest(&self) -> Option<&str> {
        self.redaction_policy_digest.as_deref()
    }

    pub fn materialization_receipt_digest(&self) -> Option<&str> {
        self.materialization_receipt_digest.as_deref()
    }

    pub fn failure_digest(&self) -> Option<&str> {
        self.failure_digest.as_deref()
    }

    pub fn counter_snapshot_digest(&self) -> Option<&str> {
        self.counter_snapshot_digest.as_deref()
    }
}
