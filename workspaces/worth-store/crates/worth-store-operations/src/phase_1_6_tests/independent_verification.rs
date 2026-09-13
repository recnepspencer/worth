use super::support::*;
use worth_store_offline_verifier::OfflinePhysicalArtifactFamily;
use worth_store_offline_verifier::{
    ObservedRecoveryFrontier, OfflineStructuralIdentification, RecoveryCandidateConfidence,
};

#[test]
fn phases_five_and_six_run_the_full_backup_truth_ladder_on_real_media() {
    let scenario = BackupScenario::new("complete-ladder");
    let authority = scenario.authority();
    let custody = backup_custody(&authority);
    let control = scenario.control_store();
    let intent = OnlineBackupIntent::new(
        OperationalOperationId::new("backup-complete-ladder").expect("operation id"),
        scenario.coordinates(),
        scenario.cut_manifest(),
        custody,
    );
    let admitted = intent
        .admit_cut(&authority, &control, &scenario.leases)
        .expect("stable cut admission");
    for artifact in scenario.references() {
        let evidence =
            ExecutedReachabilityEvidence::for_certification_reference(artifact.reclaim_reference());
        let hazards = HazardLeaseTable::with_capacity(
            HazardLeaseTableCapacity::bounded_slots(1).expect("capacity"),
        )
        .live_index_snapshot();
        let proof = ReclaimEligibilityProof::admit(
            evidence,
            hazards,
            scenario
                .leases
                .live_index_snapshot()
                .expect("active backup lease index"),
        )
        .expect("reclaim decision");
        assert!(matches!(
            proof.try_reclaim(),
            Err(ReclaimDenial::BlockedByBackupCut {
                cut_identity,
                overlapping_artifacts: 1,
                ..
            }) if cut_identity == admitted.cut().identity()
        ));
        assert_eq!(proof.counters().active_backup_leases(), 1);
        assert_eq!(proof.counters().backup_overlapping_artifacts(), 1);
    }
    let operation_id = OperationalOperationId::new("backup-complete-ladder").expect("operation id");
    let completion = admitted
        .materialize(&scenario.target, 7, &control)
        .expect("materialization session")
        .finish()
        .expect("materialization");
    assert!(completion.counters().peak_buffer_bytes() <= 7);
    assert_eq!(
        completion.counters().output_bytes_written(),
        scenario.total_bytes()
    );
    assert!(completion.counters().manifest_bytes_written() > 0);
    assert_eq!(
        completion.counters().total_output_bytes_written(),
        Some(
            completion.counters().output_bytes_written()
                + completion.counters().manifest_bytes_written()
        )
    );
    let (materialized, cut) = completion.into_parts();
    let verification_budget =
        OfflineInspectionBudget::bounded(4 * 1024, scenario.total_bytes() + 64 * 1024)
            .expect("budget");
    let structural = verify_materialized_backup(materialized, verification_budget)
        .expect("independent structural verification");
    assert_eq!(structural.report().defects(), &[]);
    assert_eq!(structural.report().owner_verified_artifacts(), 7);
    assert_eq!(
        structural.report().owner_verified_bytes(),
        scenario.total_bytes()
    );
    let manifest_bytes =
        std::fs::metadata(structural.materialized().root().join("backup.manifest"))
            .expect("manifest metadata")
            .len();
    assert_eq!(
        structural.report().inspected_bytes(),
        scenario.total_bytes() * 2 + manifest_bytes * 2
    );
    assert_eq!(
        structural.report().admitted_read_bytes(),
        structural.report().inspected_bytes()
    );
    assert_eq!(
        structural.report().read_accounting(),
        BackupVerificationReadAccounting::Complete
    );
    assert_eq!(structural.report().inspected_files(), 16);
    assert!(structural.report().peak_buffer_bytes() <= 4 * 1024);
    assert!(structural.report().manifest_owned_allocation_bytes() > 0);
    assert!(
        structural.report().peak_owned_allocation_bytes()
            <= verification_budget.maximum_owned_allocation_bytes()
    );
    assert_eq!(structural.operational_truth().regions().len(), 8);
    assert!(structural
        .operational_truth()
        .regions()
        .iter()
        .all(|region| region.evidence().structural_identification()
            == OfflineStructuralIdentification::OwnerDecoded));
    let candidates = structural
        .operational_truth()
        .recovery_candidates()
        .candidates();
    assert_eq!(candidates.len(), 3);
    let family_count = |family| {
        candidates
            .iter()
            .filter(|candidate| candidate.family() == family)
            .count()
    };
    assert_eq!(family_count(OfflinePhysicalArtifactFamily::Manifest), 2);
    assert_eq!(family_count(OfflinePhysicalArtifactFamily::Wal), 1);
    assert!(candidates.iter().all(|candidate| {
        candidate.confidence() == RecoveryCandidateConfidence::OwnerAndIntegrityConfirmed
            && candidate.content_digest() != [0; 32]
    }));
    assert!(candidates.iter().any(|candidate| matches!(
        candidate.frontier(),
        ObservedRecoveryFrontier::RootManifest { generation: 1, .. }
    )));
    assert!(candidates.iter().any(|candidate| matches!(
        candidate.frontier(),
        ObservedRecoveryFrontier::Checkpoint {
            manifest_generation: 1,
            durable_checkpoint_lsn: 10,
            root_reference: 1,
            root_generation: 1,
            covered_lsn_start: 10,
            covered_lsn_end_exclusive: 12,
            redo_lsn: 10,
            ..
        }
    )));
    let checkpoint_digest = scenario
        .references()
        .iter()
        .find(|artifact| artifact.family() == BackupArtifactFamily::CheckpointManifest)
        .expect("checkpoint reference")
        .content_digest();
    assert!(candidates.iter().any(|candidate| {
        matches!(
            candidate.frontier(),
            ObservedRecoveryFrontier::Checkpoint { .. }
        ) && candidate.content_digest() == checkpoint_digest
    }));
    assert!(candidates.iter().any(|candidate| matches!(
        candidate.frontier(),
        ObservedRecoveryFrontier::WalSegment {
            generation: 1,
            start_lsn: 10,
            end_exclusive_lsn: 12,
            ..
        }
    )));
    let verified = record_independent_backup_verification(
        &operation_id,
        structural,
        cut,
        &control,
        &scenario.leases,
    )
    .expect("join verification and release cut");
    assert_eq!(
        scenario
            .leases
            .live_index_snapshot()
            .expect("lease registry")
            .active_leases(),
        0
    );
    let custody = backup_custody(&authority);
    let qualified = qualify_backup_custody(verified, &custody).expect("custody qualification");
    let admissible = admit_backup_for_production_restore(
        qualified,
        &authority,
        BackupRestoreAdmissionPolicy::production_default(),
    )
    .expect("production restore admission");
    assert_eq!(
        admissible.admission().admitting_authority(),
        authority.authority_identity()
    );

    let selection = TestControlStoreFencingProvider::selected(
        &authority,
        &control,
        ControlStoreGeneration::from_raw(4).expect("generation"),
    );
    let fencing = ControlStoreFencingAuthority::for_current_store(&authority, &selection);
    match control.inspect_generations(&fencing) {
        ControlStoreTrustPosture::Selected(selected) => {
            assert_eq!(selected.history_summary().record_count(), 4);
            assert_eq!(selected.history_summary().completed_backups(), 1);
            assert!(selected.active_backup_recovery_handles().is_empty());
        }
        posture => panic!("control state should be selected: {posture:?}"),
    }
}

#[test]
fn complete_read_budget_is_admitted_before_component_media_is_walked() {
    let scenario = BackupScenario::new("early-read-admission");
    let authority = scenario.authority();
    let control = scenario.control_store();
    let admitted = OnlineBackupIntent::new(
        OperationalOperationId::new("backup-early-read-admission").expect("operation id"),
        scenario.coordinates(),
        scenario.cut_manifest(),
        backup_custody(&authority),
    )
    .admit_cut(&authority, &control, &scenario.leases)
    .expect("cut");
    let completion = admitted
        .materialize(&scenario.target, 11, &control)
        .expect("session")
        .finish()
        .expect("materialize");
    let (materialized, _cut) = completion.into_parts();
    let component = materialized.manifest().artifacts()[0].output_name();
    std::fs::write(
        materialized.root().join(component),
        b"observable-corruption",
    )
    .expect("controlled corruption");
    let manifest_bytes = std::fs::metadata(materialized.root().join("backup.manifest"))
        .expect("manifest metadata")
        .len();
    let denial = verify_materialized_backup(
        materialized,
        OfflineInspectionBudget::bounded(4 * 1024, manifest_bytes).expect("tight read budget"),
    )
    .expect_err("the complete two-pass read must be admitted before component inspection");
    assert!(matches!(
        denial,
        BackupStructuralVerificationDenial::Inspection(
            OfflineInspectionDenial::ReadBudgetExceeded { admitted, limit }
        ) if admitted > limit && limit == manifest_bytes
    ));
}

#[test]
fn owner_semantic_verification_is_admitted_by_the_global_owned_memory_budget() {
    let scenario = BackupScenario::new("owner-memory-budget");
    let authority = scenario.authority();
    let control = scenario.control_store();
    let admitted = OnlineBackupIntent::new(
        OperationalOperationId::new("backup-owner-memory").expect("operation id"),
        scenario.coordinates(),
        scenario.cut_manifest(),
        backup_custody(&authority),
    )
    .admit_cut(&authority, &control, &scenario.leases)
    .expect("cut");
    let completion = admitted
        .materialize(&scenario.target, 17, &control)
        .expect("session")
        .finish()
        .expect("materialize");
    let (materialized, _cut) = completion.into_parts();
    let root = materialized.root().to_path_buf();
    let broad_budget = OfflineInspectionBudget::bounded(4 * 1024, u64::MAX).expect("budget");
    let verified = verify_materialized_backup(materialized, broad_budget).expect("verification");
    let exact_peak = verified.report().peak_owned_allocation_bytes();
    assert!(exact_peak > broad_budget.max_buffer_bytes() as u64);

    let reopened = BackupBundleFormatAuthority::canonical()
        .admit_materialized(&root)
        .expect("reopen materialized bundle");
    let tight_budget = broad_budget
        .with_maximum_owned_allocation_bytes(exact_peak - 1)
        .expect("tight owned-memory budget");
    let denial = verify_materialized_backup(reopened, tight_budget)
        .expect_err("one byte below the observed peak must fail closed");
    assert!(matches!(
        denial,
        BackupStructuralVerificationDenial::OwnedAllocationBudgetExceeded {
            phase: BackupVerificationAllocationPhase::OwnerSemanticVerification,
            admitted,
            limit,
        } if admitted > limit && limit == exact_peak - 1
    ));
}

#[test]
fn independent_verification_rejects_component_corruption_after_publication() {
    let scenario = BackupScenario::new("corrupt-component");
    let authority = scenario.authority();
    let control = scenario.control_store();
    let admitted = OnlineBackupIntent::new(
        OperationalOperationId::new("backup-corrupt-component").expect("operation id"),
        scenario.coordinates(),
        scenario.cut_manifest(),
        backup_custody(&authority),
    )
    .admit_cut(&authority, &control, &scenario.leases)
    .expect("cut");
    let completion = admitted
        .materialize(&scenario.target, 11, &control)
        .expect("session")
        .finish()
        .expect("materialize");
    let (materialized, _cut) = completion.into_parts();
    let first_component = materialized.manifest().artifacts()[0].output_name();
    std::fs::write(
        materialized.root().join(first_component),
        b"substituted-after-publication",
    )
    .expect("controlled defect");
    let result = verify_materialized_backup(
        materialized,
        OfflineInspectionBudget::bounded(4 * 1024, scenario.total_bytes() + 64 * 1024)
            .expect("budget"),
    );
    match result {
        Err(BackupStructuralVerificationDenial::Defects(report)) => {
            assert!(report.defects().iter().any(|defect| matches!(
                defect,
                BackupVerificationDefect::ComponentLengthMismatch { .. }
                    | BackupVerificationDefect::ComponentDigestMismatch { .. }
            )))
        }
        other => panic!("corrupt component must fail structural verification: {other:?}"),
    }
}

#[test]
fn independent_verification_rejects_nested_component_name_shadowing() {
    let scenario = BackupScenario::new("nested-shadow");
    let authority = scenario.authority();
    let control = scenario.control_store();
    let admitted = OnlineBackupIntent::new(
        OperationalOperationId::new("backup-nested-shadow").expect("operation id"),
        scenario.coordinates(),
        scenario.cut_manifest(),
        backup_custody(&authority),
    )
    .admit_cut(&authority, &control, &scenario.leases)
    .expect("cut");
    let completion = admitted
        .materialize(&scenario.target, 11, &control)
        .expect("session")
        .finish()
        .expect("materialize");
    let (materialized, _cut) = completion.into_parts();
    let shadow = materialized.root().join("nested");
    std::fs::create_dir_all(&shadow).expect("nested directory");
    let expected_name = materialized.manifest().artifacts()[0].output_name();
    std::fs::write(shadow.join(expected_name), b"shadow").expect("shadow component");
    let result = verify_materialized_backup(
        materialized,
        OfflineInspectionBudget::bounded(4 * 1024, scenario.total_bytes() + 64 * 1024)
            .expect("budget"),
    );
    match result {
        Err(BackupStructuralVerificationDenial::Defects(report)) => assert!(report
            .defects()
            .iter()
            .any(|defect| matches!(defect, BackupVerificationDefect::ExtraComponent { .. }))),
        other => panic!("nested shadow must fail structural verification: {other:?}"),
    }
}
