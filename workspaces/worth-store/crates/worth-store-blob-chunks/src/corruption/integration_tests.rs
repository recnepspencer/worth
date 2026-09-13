use crate::corruption::test_support::quarantined_read_corruption;
use crate::corruption::{
    classify_generation_posture, construct_quarantine_diagnostics,
    observe_physical_pre_decode_denial,
};
use crate::handoffs::reject_physical_handoff_from_pre_decode_denial;
use crate::lifecycle::generation_registry_test_support::current_authority;
use crate::publication::test_support::publish_generation_with_bytes_and_chunk_size;
use crate::test_support::frontier_for;
use crate::{
    classify_blob_damage_before_decode, AuthoritativeBlobCorruptionPosture, BlobChunkOrdinal,
    BlobChunkQuarantine, BlobCorruptedChunkLocalization, BlobCorruptionDenial,
    BlobCorruptionDetectionSource, BlobCorruptionImportReadmission, BlobCorruptionPlacementClass,
    BlobCorruptionReferenceEdges, BlobDamageCase, BlobDamageEvidence, BlobObjectClassification,
    BlobQuarantineAuthority, BlobQuarantineDiagnostics, BlobQuarantineLifecycleState,
    BlobQuarantineRepairCapability,
};
use worth_proof::TransitionOutcome;

#[test]
fn integration_paths_classify_all_five_damage_cases_before_decode() {
    let (published, visible) =
        publish_generation_with_bytes_and_chunk_size("phase11.all-cases", b"aaaabbbbcccc", 4);
    let frontier = frontier_for("phase11.all-cases", b"aaaabbbbcccc", 4);
    let edges = BlobCorruptionReferenceEdges::from_reachability_staging_identity(
        published.staging_identity(),
    )
    .expect("staging identity should bind");

    let checksum = BlobCorruptedChunkLocalization::from_read_corruption(
        visible.clone(),
        frontier.clone(),
        BlobChunkOrdinal::first(),
        BlobCorruptionPlacementClass::LocalPhysical,
        edges.clone(),
    )
    .expect("checksum path should localize");
    assert_eq!(checksum.damage_case(), BlobDamageCase::ChecksumMismatch);

    let missing = BlobCorruptedChunkLocalization::from_detected_source(
        BlobCorruptionDetectionSource::ColdFetch,
        visible.clone(),
        frontier.clone(),
        BlobChunkOrdinal::first().next(),
        BlobCorruptionPlacementClass::ExternalCold,
        edges.clone(),
    )
    .expect("cold fetch should localize");
    assert_eq!(missing.damage_case(), BlobDamageCase::MissingChunk);

    let authenticity = BlobCorruptedChunkLocalization::from_detected_source(
        BlobCorruptionDetectionSource::CapsuleMaterialization,
        visible.clone(),
        frontier.clone(),
        BlobChunkOrdinal::first().next().next(),
        BlobCorruptionPlacementClass::CapsuleMaterialization,
        edges.clone(),
    )
    .expect("capsule materialization should localize");
    assert_eq!(
        authenticity.damage_case(),
        BlobDamageCase::AuthenticityFailure
    );

    let import = BlobCorruptedChunkLocalization::from_detected_source(
        BlobCorruptionDetectionSource::ImportReadmission,
        visible.clone(),
        frontier.clone(),
        BlobChunkOrdinal::first(),
        BlobCorruptionPlacementClass::ImportStaging,
        edges.clone(),
    )
    .expect("import readmission should localize");
    assert_eq!(import.damage_case(), BlobDamageCase::CrossScopeImport);

    let mismatched_frontier = frontier_for("phase11.all-cases.other", b"ddddeeee", 4);
    let stale = BlobCorruptedChunkLocalization::from_read_corruption(
        visible,
        mismatched_frontier,
        BlobChunkOrdinal::first(),
        BlobCorruptionPlacementClass::LocalPhysical,
        edges,
    )
    .expect_err("stale generation should deny before decode");
    match stale {
        BlobCorruptionDenial::GenerationFrontierMismatch { damage_case, .. } => {
            assert_eq!(damage_case, BlobDamageCase::StaleGeneration);
        }
        other => panic!("unexpected denial: {other:?}"),
    }
}

#[test]
fn localization_denial_threads_classified_damage_case() {
    let (published, visible) =
        publish_generation_with_bytes_and_chunk_size("phase11.damage-thread", b"aaaabbbb", 4);
    let frontier = frontier_for("phase11.damage-thread", b"aaaabbbb", 4);
    let edges = BlobCorruptionReferenceEdges::from_reachability_staging_identity(
        published.staging_identity(),
    )
    .expect("staging identity should bind");

    let denied = BlobCorruptedChunkLocalization::from_read_corruption(
        visible,
        frontier,
        BlobChunkOrdinal::first().next().next().next(),
        BlobCorruptionPlacementClass::LocalPhysical,
        edges,
    )
    .expect_err("missing chunk ordinal should deny");

    match denied {
        BlobCorruptionDenial::CorruptOrdinalNotInPublishedFrontier { damage_case, .. } => {
            assert_eq!(damage_case, BlobDamageCase::MissingChunk);
        }
        other => panic!("unexpected denial: {other:?}"),
    }
}

#[test]
fn classify_generation_posture_transition_wraps_quarantine_classification() {
    let (published, visible) =
        publish_generation_with_bytes_and_chunk_size("phase11.posture", b"aaaabbbb", 4);
    let quarantine = quarantined_read_corruption("phase11.posture", &published, visible);
    let posture =
        classify_generation_posture(&quarantine, BlobObjectClassification::authoritative())
            .authoritative_posture()
            .expect("authoritative posture should classify");
    assert_eq!(
        posture.state(),
        BlobQuarantineLifecycleState::RepairRequiredAuthoritative
    );
}

#[test]
fn import_readmission_admits_from_authoritative_restore_posture() {
    let (published, visible) =
        publish_generation_with_bytes_and_chunk_size("phase11.readmission", b"aaaabbbb", 4);
    let frontier = frontier_for("phase11.readmission", b"aaaabbbb", 4);
    let edges = BlobCorruptionReferenceEdges::from_reachability_staging_identity(
        published.staging_identity(),
    )
    .expect("staging identity should bind");
    let localized = BlobCorruptedChunkLocalization::from_detected_source(
        BlobCorruptionDetectionSource::ImportReadmission,
        visible,
        frontier,
        BlobChunkOrdinal::first(),
        BlobCorruptionPlacementClass::ImportStaging,
        edges,
    )
    .expect("import corruption should localize");
    let quarantine = BlobChunkQuarantine::seal(
        localized,
        BlobQuarantineAuthority::from_current_store_authority(current_authority(
            "phase11.readmission.quarantine",
            "quarantine",
        )),
    );
    assert_eq!(
        quarantine.state(),
        BlobQuarantineLifecycleState::ImportCorrupt
    );
    let posture: AuthoritativeBlobCorruptionPosture =
        classify_generation_posture(&quarantine, BlobObjectClassification::authoritative())
            .authoritative_restore_posture()
            .expect("restore posture should classify");
    match BlobCorruptionImportReadmission::admit_from_posture(
        posture,
        current_authority("phase11.readmission.authority", "readmission"),
    ) {
        TransitionOutcome::Success(_) => {}
        other => panic!("expected import readmission admission: {other:?}"),
    }
}

#[test]
fn quarantine_diagnostics_expose_repair_capability_not_read_authority() {
    let (published, visible) =
        publish_generation_with_bytes_and_chunk_size("phase11.diagnostics", b"aaaabbbb", 4);
    let quarantine =
        quarantined_read_corruption("phase11.diagnostics", &published, visible.clone());
    let diagnostics: BlobQuarantineDiagnostics = construct_quarantine_diagnostics(
        quarantine.clone(),
        quarantine.localization().damage_case(),
    );
    assert!(matches!(
        diagnostics.repair_capability(),
        BlobQuarantineRepairCapability::ClassifyGenerationPosture
    ));

    let frontier = frontier_for("phase11.diagnostics.import", b"aaaabbbb", 4);
    let edges = BlobCorruptionReferenceEdges::from_reachability_staging_identity(
        published.staging_identity(),
    )
    .expect("staging identity should bind");
    let import_localized = BlobCorruptedChunkLocalization::from_detected_source(
        BlobCorruptionDetectionSource::ImportReadmission,
        visible,
        frontier,
        BlobChunkOrdinal::first(),
        BlobCorruptionPlacementClass::ImportStaging,
        edges,
    )
    .expect("import corruption should localize");
    let import_quarantine = BlobChunkQuarantine::seal(
        import_localized,
        BlobQuarantineAuthority::from_current_store_authority(current_authority(
            "phase11.diagnostics.import.quarantine",
            "quarantine",
        )),
    );
    let import_diagnostics = construct_quarantine_diagnostics(
        import_quarantine.clone(),
        import_quarantine.localization().damage_case(),
    );
    assert!(matches!(
        import_diagnostics.repair_capability(),
        BlobQuarantineRepairCapability::AdmitImportReadmission(_)
    ));
}

#[test]
fn owner_local_physical_observations_preserve_blob_damage_classification() {
    for expected in [
        BlobDamageCase::ChecksumMismatch,
        BlobDamageCase::AuthenticityFailure,
        BlobDamageCase::MissingChunk,
        BlobDamageCase::StaleGeneration,
    ] {
        assert_eq!(
            classify_blob_damage_before_decode(BlobDamageEvidence::PhysicalObservation(expected)),
            expected
        );
    }
}

#[test]
fn physical_handoff_rejects_lower_evidence_as_blob_authority() {
    let (classification, rejection) =
        observe_physical_pre_decode_denial(BlobDamageCase::ChecksumMismatch);
    assert_eq!(
        classification.damage_case(),
        BlobDamageCase::ChecksumMismatch
    );
    assert!(matches!(
        rejection,
        BlobCorruptionDenial::LowerPhysicalEvidenceRejected {
            damage_case: BlobDamageCase::ChecksumMismatch,
            ..
        }
    ));

    let (handoff_classification, handoff_rejection) =
        reject_physical_handoff_from_pre_decode_denial(BlobDamageCase::ChecksumMismatch);
    assert_eq!(
        handoff_classification.damage_case(),
        BlobDamageCase::ChecksumMismatch
    );
    assert!(matches!(
        handoff_rejection,
        BlobCorruptionDenial::LowerPhysicalEvidenceRejected {
            damage_case: BlobDamageCase::ChecksumMismatch,
            ..
        }
    ));
}
