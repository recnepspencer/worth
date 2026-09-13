use crate::OfflinePhysicalArtifactFamily;
use sha2::{Digest, Sha256};
use worth_store_physical_backend::{OfflineMediaClosureEntry, OfflineMediaConsistencyBasis};
use worth_store_physical_format::{
    PhysicalGeneration, PhysicalGenerationAuthority, PhysicalPageId, PhysicalSegmentId,
};

use super::candidate_evaluation::synthetic_observation_for_test;
use super::{
    compose_operational_truth, compose_operational_truth_with_owner_candidates,
    OfflineFileTruthEvidence, OfflineTruthEvidenceAdmissionDenial, OfflineTruthEvidenceSet,
    OperationalTruthCompositionBudget, OperationalTruthCompositionDenial,
};
use crate::{OfflineInspectionBudget, OfflineStoreInspection, UntrustedOfflineMediaSet};

#[test]
fn truth_evidence_collection_denies_owned_rows_before_growth() {
    let denial = OfflineTruthEvidenceSet::from_entries(
        [OfflineFileTruthEvidence::new("authority.manifest")],
        1,
    )
    .expect_err("one byte cannot own an evidence row");
    assert!(matches!(
        denial,
        OfflineTruthEvidenceAdmissionDenial::OwnedAllocationBudgetExceeded {
            admitted,
            limit: 1,
        } if admitted > 1
    ));
}

#[test]
fn truth_composition_reports_and_enforces_its_exact_owned_peak() {
    let directory = tempfile::tempdir().expect("temp directory");
    let path = directory.path().join("primary.page");
    let bytes = vec![7_u8; 4096];
    std::fs::write(&path, &bytes).expect("media");

    let broad = OperationalTruthCompositionBudget::bounded(1024 * 1024).expect("budget");
    let report = compose_operational_truth(
        inspect(&path, &bytes),
        &OfflineTruthEvidenceSet::default(),
        broad,
    )
    .expect("truth composition");
    let exact_peak = report.peak_owned_allocation_bytes();
    assert!(exact_peak > 1);
    assert!(exact_peak <= broad.maximum_owned_allocation_bytes());

    let tight = OperationalTruthCompositionBudget::bounded(exact_peak - 1).expect("tight budget");
    let denial = compose_operational_truth(
        inspect(&path, &bytes),
        &OfflineTruthEvidenceSet::default(),
        tight,
    )
    .expect_err("one byte below the observed peak must fail closed");
    assert!(matches!(
        denial,
        OperationalTruthCompositionDenial::OwnedAllocationBudgetExceeded {
            admitted,
            limit,
        } if admitted > limit && limit == exact_peak - 1
    ));
}

#[test]
fn operational_composition_preserves_conflicting_candidate_denial() {
    let directory = tempfile::tempdir().expect("temp directory");
    let path = directory.path().join("candidate.page");
    let bytes = b"candidate-media";
    std::fs::write(&path, bytes).expect("media");
    let mut no_interruption = || Ok(());
    let denial = compose_operational_truth_with_owner_candidates(
        inspect(&path, bytes),
        &OfflineTruthEvidenceSet::default(),
        vec![
            synthetic_observation_for_test(1),
            synthetic_observation_for_test(2),
        ],
        OperationalTruthCompositionBudget::bounded(64 * 1024).expect("budget"),
        &mut no_interruption,
    )
    .expect_err("conflicting owner frontiers must survive composition");
    assert!(matches!(
        denial,
        OperationalTruthCompositionDenial::ConflictingFrontierEvidence
    ));
}

#[test]
fn every_truth_row_retains_physical_identity_recovery_relevance_and_evidence_references() {
    let directory = tempfile::tempdir().expect("temp directory");
    let path = directory.path().join("primary.page");
    let bytes = b"owner-media";
    std::fs::write(&path, bytes).expect("media");
    let expected_digest = Sha256::digest(bytes).into();
    let evidence = OfflineTruthEvidenceSet::from_entries(
        [OfflineFileTruthEvidence::new(&path).with_expected_digest(expected_digest)],
        4096,
    )
    .expect("evidence");

    let report = compose_operational_truth(
        inspect(&path, bytes),
        &evidence,
        OperationalTruthCompositionBudget::bounded(64 * 1024).expect("budget"),
    )
    .expect("truth composition");
    let region = report.regions()[0].evidence();
    let references = region.evidence_references();

    assert_eq!(region.source(), path);
    assert_eq!(region.media_identity().length(), bytes.len() as u64);
    assert_ne!(region.media_identity().physical_key_fingerprint(), [0; 32]);
    assert_eq!(
        region.recovery_availability(),
        super::OfflineRecoveryAvailability::Unknown
    );
    assert_eq!(references.media_source_index(), 0);
    assert_eq!(references.observed_content_digest(), expected_digest);
    assert_eq!(references.declared_expected_digest(), Some(expected_digest));
    assert_eq!(references.security_scope_receipt(), None);
}

#[test]
fn admitted_security_evidence_survives_truth_composition_with_its_exact_receipt() {
    use worth_store_security::{
        admit_store_authenticity_witness_observation,
        admitted_wrong_io_qos_security_scope_for_test, StoreAuthenticityCheck,
        StoreAuthenticityWitnessObservationDeclaration,
    };

    let directory = tempfile::tempdir().expect("temp directory");
    let path = directory.path().join("secured.page");
    let bytes = b"authenticated-owner-media";
    std::fs::write(&path, bytes).expect("media");
    let digest: [u8; 32] = Sha256::digest(bytes).into();
    let admitted_scope = admitted_wrong_io_qos_security_scope_for_test();
    let authenticity_scope = admitted_scope.witnesses().authenticity_scope();
    let authenticity = StoreAuthenticityCheck::for_requirement(authenticity_scope.requirement())
        .with_security_scope(authenticity_scope)
        .with_physical_identity(digest)
        .with_witness(admit_store_authenticity_witness_observation(
            authenticity_scope,
            digest,
            StoreAuthenticityWitnessObservationDeclaration::verified(),
        ))
        .admit()
        .expect("owner authenticity evidence");
    let receipt = admitted_scope.receipt();
    let evidence = OfflineTruthEvidenceSet::from_entries(
        [OfflineFileTruthEvidence::from_admitted_security_evidence(
            &path,
            digest,
            authenticity,
            receipt,
        )
        .expect("security evidence")],
        4096,
    )
    .expect("evidence set");

    let report = compose_operational_truth(
        inspect(&path, bytes),
        &evidence,
        OperationalTruthCompositionBudget::bounded(64 * 1024).expect("budget"),
    )
    .expect("truth composition");
    let region = report.regions()[0].evidence();

    assert_eq!(region.security_scope(), Some(receipt.identity()));
    assert_eq!(
        region.evidence_references().security_scope_receipt(),
        Some(receipt.receipt_id())
    );
}

#[test]
fn owner_decoded_duplicate_physical_claims_become_one_explicit_overlap_conflict() {
    let directory = tempfile::tempdir().expect("temp directory");
    let first = directory.path().join("first.page");
    let second = directory.path().join("second.page");
    std::fs::write(&first, b"first").expect("first media");
    std::fs::write(&second, b"second").expect("second media");
    let mut walked = inspect_paths(&[(&first, b"first"), (&second, b"second")]);
    let generation = PhysicalGeneration::from_raw(7).expect("generation");
    let owner = PhysicalGenerationAuthority::for_canonical_physical_format()
        .page_cell(
            PhysicalSegmentId::from_raw(3).expect("segment"),
            PhysicalPageId::from_raw(9).expect("page"),
        )
        .with_page_generation(generation)
        .owner();
    walked
        .bind_owner_observations(
            [&first, &second]
                .into_iter()
                .map(|path| {
                    crate::inspection::OwnerDecodedArtifactBinding::with_physical_owner(
                        path.clone(),
                        OfflinePhysicalArtifactFamily::Page,
                        generation.get(),
                        owner,
                    )
                    .expect("owner binding")
                })
                .collect(),
        )
        .expect("owner observations");

    let report = compose_operational_truth(
        walked,
        &OfflineTruthEvidenceSet::default(),
        OperationalTruthCompositionBudget::bounded(64 * 1024).expect("budget"),
    )
    .expect("truth composition");

    assert_eq!(report.regions().len(), 1);
    match &report.regions()[0] {
        super::OperationalTruthRegion::OverlapConflict {
            representative,
            additional_claims,
            claimants,
        } => {
            assert_eq!(representative.physical_owner(), Some(owner));
            assert_eq!(additional_claims.len(), 1);
            assert_eq!(additional_claims[0].physical_owner(), Some(owner));
            assert_eq!(claimants, &[first, second]);
        }
        other => panic!("duplicate owner claims must be explicit: {other:?}"),
    }
}

fn inspect(path: &std::path::Path, bytes: &[u8]) -> crate::StructurallyWalkedMedia {
    let closure = OfflineMediaConsistencyBasis::content_addressed_closure(
        "truth-composition",
        [
            OfflineMediaClosureEntry::new(path, bytes.len() as u64, Sha256::digest(bytes).into())
                .expect("closure row"),
        ],
    )
    .expect("content closure");
    OfflineStoreInspection::open(UntrustedOfflineMediaSet::from_root(
        path.parent().expect("test file has parent"),
        closure,
    ))
    .budget(OfflineInspectionBudget::bounded(64, 8192).expect("inspection budget"))
    .start()
    .expect("start")
    .finish()
    .expect("finish")
}

fn inspect_paths(paths: &[(&std::path::PathBuf, &[u8])]) -> crate::StructurallyWalkedMedia {
    let closure = OfflineMediaConsistencyBasis::content_addressed_closure(
        "truth-overlap",
        paths.iter().map(|(path, bytes)| {
            OfflineMediaClosureEntry::new(path, bytes.len() as u64, Sha256::digest(bytes).into())
                .expect("closure row")
        }),
    )
    .expect("content closure");
    OfflineStoreInspection::open(UntrustedOfflineMediaSet::from_root(
        paths[0].0.parent().expect("test files have parent"),
        closure,
    ))
    .budget(OfflineInspectionBudget::bounded(64, 8192).expect("inspection budget"))
    .start()
    .expect("start")
    .finish()
    .expect("finish")
}
