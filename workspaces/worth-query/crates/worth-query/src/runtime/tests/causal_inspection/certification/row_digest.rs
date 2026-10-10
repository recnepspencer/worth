use super::super::super::super::*;
use super::artifact_support::{
    admitted_artifact, advisory_artifacts, denied_artifact_and_missing_evidence,
};
use super::matrix_support::representative_matrix;
use super::slot_support::artifact_with_lower_runtime_slot_evidence;

#[test]
fn causal_inspection_representative_rows_expose_digest_inventory() {
    let host_request = crate::bridge_host_request::serial_request();
    let resource_request = worth_execution::ExecutionRequest::serial(&host_request);

    let changed = admitted_artifact(
        super::super::causal_truth_commit_identity("commit-query-cert-row-digest-changed"),
        resource_request,
    );
    let (_, redacted) = advisory_artifacts(
        super::super::causal_truth_commit_identity("commit-query-cert-row-digest-redacted"),
        resource_request,
    );
    let (denied, _) = denied_artifact_and_missing_evidence(resource_request);
    let row = CausalInspectionRepresentativeEvidence::from_query_artifact(
        CausalInspectionRepresentativeKind::ChangedResult,
        &changed,
    )
    .unwrap();
    let digest_set = row.row_digest_set();

    assert_eq!(
        digest_set.artifact_digest(),
        Some(changed.artifact_for_reporting())
    );
    assert_eq!(
        digest_set.causal_envelope_digest(),
        changed.bridge_envelope_for_reporting()
    );
    assert!(digest_set.inspection_digest().is_some());
    assert!(digest_set.evidence_reference_collection_digest().is_some());
    assert!(digest_set.materialization_policy_digest().is_some());
    assert!(digest_set.redaction_policy_digest().is_some());
    assert!(digest_set.materialization_receipt_digest().is_some());
    assert!(digest_set.counter_snapshot_digest().is_some());
    assert!(!digest_set.query_digest().is_empty());
    assert!(!digest_set.causal_observation_anchor_digest().is_empty());
    assert!(digest_set.bridge_route_digest().is_some());
    assert!(digest_set.bridge_evaluation_digest().is_none());
    assert!(digest_set.bridge_replay_digest().is_none());
    assert!(digest_set.signal_invalidation_digest().is_none());
    assert!(digest_set.signal_forensic_availability_digest().is_none());
    assert!(digest_set.failure_digest().is_none());

    let slot_artifact = artifact_with_lower_runtime_slot_evidence(
        super::super::causal_truth_commit_identity("commit-query-cert-row-digest-slots"),
        resource_request,
    );
    let slot_row = CausalInspectionRepresentativeEvidence::from_query_artifact(
        CausalInspectionRepresentativeKind::ChangedResult,
        &slot_artifact,
    )
    .unwrap();
    let slot_digest_set = slot_row.row_digest_set();

    assert!(slot_digest_set.relational_authority_digest().is_some());
    assert!(slot_digest_set.bridge_route_digest().is_some());
    assert!(slot_digest_set.bridge_evaluation_digest().is_some());
    assert!(slot_digest_set.bridge_preview_digest().is_some());
    assert!(slot_digest_set.signal_invalidation_digest().is_some());
    assert!(slot_digest_set
        .bridge_source_materialization_digest()
        .is_some());
    assert!(slot_digest_set.bridge_structural_digest().is_some());
    assert!(slot_digest_set.bridge_stream_digest().is_some());
    assert!(slot_digest_set.bridge_replay_digest().is_some());
    assert!(slot_digest_set.bridge_writeback_digest().is_some());
    assert!(slot_digest_set.signal_evaluation_digest().is_some());
    assert!(slot_digest_set
        .signal_forensic_availability_digest()
        .is_some());
    assert!(slot_digest_set.signal_replay_cursor_digest().is_some());
    assert!(slot_digest_set.signal_lineage_digest().is_some());
    assert!(slot_digest_set.signal_provenance_digest().is_some());
    assert!(slot_digest_set.replay_posture_digest().is_some());
    if let QueryCausalInspectionArtifact::Admitted(artifact) = &slot_artifact {
        assert!(artifact.evidence_references().iter().any(|reference| {
            reference.owner() == "signal" && reference.family() == "signal_provenance"
        }));
        assert!(artifact.evidence_references().iter().any(|reference| {
            reference.owner() == "signal" && reference.family() == "signal_replay_cursor"
        }));
    } else {
        panic!("slot fixture should materialize an admitted artifact");
    }
    assert_eq!(slot_digest_set.populated_named_evidence_slot_count(), 16);
    assert_eq!(
        slot_digest_set.populated_non_writeback_bridge_runtime_slot_count(),
        7
    );
    assert_eq!(slot_digest_set.populated_bridge_runtime_slot_count(), 8);
    assert_eq!(slot_digest_set.populated_signal_slot_count(), 6);
    assert!(slot_digest_set.has_retained_source_structural_stream_replay_slot_coverage());
    assert!(slot_digest_set.has_retained_source_structural_stream_writeback_replay_slot_coverage());
    assert!(slot_digest_set
        .has_signal_evaluation_forensic_replay_lineage_provenance_reference_coverage());
    assert!(slot_digest_set.has_replay_posture_coverage());

    let matrix = representative_matrix(&changed, &redacted, &denied, resource_request);
    assert_eq!(matrix.representative_digests().len(), 25);
    assert_eq!(matrix.row_digest_set_digests().len(), 25);
}

#[test]
fn causal_inspection_missing_representative_rows_expose_failure_inventory() {
    let row = CausalInspectionRepresentativeEvidence::from_missing_evidence(
        CausalInspectionRepresentativeKind::MissingSignalInvalidationEvidenceDenied,
        CausalEvidenceFamily::SignalInvalidation,
        "signal-invalidation-failure-digest",
    )
    .unwrap();
    let digest_set = row.row_digest_set();

    assert_eq!(
        digest_set.failure_digest(),
        Some("signal-invalidation-failure-digest")
    );
    assert_eq!(
        digest_set.kind(),
        CausalInspectionRepresentativeKind::MissingSignalInvalidationEvidenceDenied
    );
    assert!(digest_set.artifact_digest().is_none());
    assert!(digest_set.causal_envelope_digest().is_none());
    assert!(digest_set.evidence_reference_collection_digest().is_none());
    assert!(digest_set.materialization_receipt_digest().is_none());
    assert!(digest_set.bridge_route_digest().is_none());
    assert!(digest_set.bridge_replay_digest().is_none());
    assert!(digest_set.signal_invalidation_digest().is_none());
    assert!(digest_set.signal_forensic_availability_digest().is_none());
    assert!(digest_set.counter_snapshot_digest().is_some());
}
