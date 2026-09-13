use super::support::*;

#[test]
fn caller_observation_cannot_admit_arbitrary_bytes_as_an_owner_artifact() {
    let scenario = BackupScenario::new("forged-source-owner");
    let page_index = scenario
        .references()
        .iter()
        .position(|artifact| artifact.family() == BackupArtifactFamily::Page)
        .expect("page artifact");
    let original = &scenario.references()[page_index];
    let forged_path = scenario.source.join("forged-page.media");
    std::fs::write(&forged_path, b"not a physical page").expect("forged source");
    let forged = BackupArtifactReference::declare_untrusted_physical_observation(
        UntrustedBackupArtifactClaim {
            family: original.family(),
            format: original.format(),
            identity: original.identity().to_owned(),
            generation: original.generation(),
            coverage: original.coverage().clone(),
        },
        observe_physical_backup_artifact(forged_path, 4 * 1024).expect("untrusted observation"),
        original.reclaim_reference(),
    )
    .expect("an observation is only a declaration, not owner authority");
    let mut references = scenario.references().to_vec();
    references[page_index] = forged;
    let manifest = BackupCutManifest::canonical(references).expect("syntactic cut manifest");
    let authority = scenario.authority();

    let denial = OnlineBackupIntent::new(
        OperationalOperationId::new("backup-forged-source-owner").expect("operation"),
        scenario.coordinates(),
        manifest,
        backup_custody(&authority),
    )
    .admit_cut(&authority, &scenario.control_store(), &scenario.leases)
    .expect_err("owner decode must precede lease authority");

    assert!(matches!(
        &denial,
        OnlineBackupAdmissionDenial::SourceVerification(denial)
            if matches!(
                denial.source(),
                worth_store_offline_verifier::BackupCutSourceVerificationDenial::Defects(_)
            )
    ));
    assert_eq!(
        scenario
            .leases
            .live_index_snapshot()
            .expect("lease registry")
            .active_leases(),
        0
    );
    assert_eq!(
        std::fs::read_dir(&scenario.target).expect("target").count(),
        0,
        "invalid owner bytes must fail before output allocation"
    );
}

#[test]
fn source_verification_honors_cancellation_before_owner_media_reads() {
    let scenario = BackupScenario::new("cancel-source-verification");
    let cancellation = worth_store_offline_verifier::OfflineInspectionCancellation::new();
    cancellation.cancel();
    let denial = worth_store_offline_verifier::verify_backup_cut_sources_with_cancellation(
        &scenario.cut_manifest(),
        OfflineInspectionBudget::bounded(4 * 1024, u64::MAX).expect("budget"),
        cancellation,
    )
    .expect_err("pre-cancelled source verification must not decode owner media");
    assert!(matches!(
        denial,
        worth_store_offline_verifier::BackupCutSourceVerificationDenial::Inspection(
            worth_store_offline_verifier::OfflineInspectionDenial::Cancelled
        )
    ));
}

#[test]
fn admission_cancellation_rolls_back_lease_reservation_and_returns_an_exact_retry() {
    let scenario = BackupScenario::new("cancel-source-admission");
    let authority = scenario.authority();
    let control = scenario.control_store();
    let manifest = scenario.cut_manifest();
    let budget = OfflineInspectionBudget::bounded(4 * 1024, manifest.total_bytes())
        .expect("exact read budget");
    let cancellation = worth_store_offline_verifier::OfflineInspectionCancellation::new();
    cancellation.cancel();
    let denial = OnlineBackupIntent::new(
        OperationalOperationId::new("backup-cancel-source-admission").expect("operation"),
        scenario.coordinates(),
        manifest,
        backup_custody(&authority),
    )
    .admit_cut_with_verification(&authority, &control, &scenario.leases, budget, cancellation)
    .expect_err("cancelled source verification cannot persist cut authority");
    let OnlineBackupAdmissionDenial::SourceVerification(denial) = denial else {
        panic!("cancellation must remain a source-verification denial");
    };
    assert!(matches!(
        denial.source(),
        worth_store_offline_verifier::BackupCutSourceVerificationDenial::Inspection(
            worth_store_offline_verifier::OfflineInspectionDenial::Cancelled
        )
    ));
    assert_eq!(
        scenario
            .leases
            .live_index_snapshot()
            .expect("rolled-back lease reservation")
            .active_leases(),
        0
    );
    assert_eq!(
        control
            .observe_selection_coordinates()
            .expect("control observation"),
        None,
        "cancelled preflight must not leave an unrecoverable opened workflow"
    );

    let (unverified, _) = denial.into_retry();
    let admitted = unverified
        .persist_with_verification(
            &control,
            &scenario.leases,
            budget,
            worth_store_offline_verifier::OfflineInspectionCancellation::new(),
        )
        .expect("same cut retries with a live cancellation scope");
    assert_eq!(
        admitted.source_verification().admitted_read_bytes(),
        budget.max_total_read_bytes()
    );
}

#[test]
fn admission_rejects_an_underfunded_read_budget_without_leaking_reachability() {
    let scenario = BackupScenario::new("source-read-budget");
    let authority = scenario.authority();
    let manifest = scenario.cut_manifest();
    let required = manifest.total_bytes();
    let budget = OfflineInspectionBudget::bounded(4 * 1024, required - 1).expect("short budget");
    let denial = OnlineBackupIntent::new(
        OperationalOperationId::new("backup-source-read-budget").expect("operation"),
        scenario.coordinates(),
        manifest,
        backup_custody(&authority),
    )
    .admit_cut_with_verification(
        &authority,
        &scenario.control_store(),
        &scenario.leases,
        budget,
        worth_store_offline_verifier::OfflineInspectionCancellation::new(),
    )
    .expect_err("read budget must be admitted before media traversal");
    assert!(matches!(
        denial,
        OnlineBackupAdmissionDenial::SourceVerification(ref denial)
            if matches!(
                denial.source(),
                worth_store_offline_verifier::BackupCutSourceVerificationDenial::ReadBudgetExceeded {
                    required: observed,
                    limit,
                } if *observed == required && *limit == required - 1
            )
    ));
    assert_eq!(
        scenario
            .leases
            .live_index_snapshot()
            .expect("rolled-back lease reservation")
            .active_leases(),
        0
    );
}

#[test]
fn source_verification_enforces_its_observed_owned_memory_peak() {
    let scenario = BackupScenario::new("source-owned-memory");
    let manifest = scenario.cut_manifest();
    let broad = OfflineInspectionBudget::bounded(4 * 1024, u64::MAX).expect("budget");
    let report = worth_store_offline_verifier::verify_backup_cut_sources(&manifest, broad)
        .expect("source verification");
    assert_eq!(
        report.read_accounting(),
        BackupVerificationReadAccounting::Complete
    );
    assert_eq!(report.inspected_bytes(), report.admitted_read_bytes());
    let exact_peak = report.peak_owned_allocation_bytes();
    let tight = broad
        .with_maximum_owned_allocation_bytes(exact_peak - 1)
        .expect("one-byte-under budget");
    let denial = worth_store_offline_verifier::verify_backup_cut_sources(&manifest, tight)
        .expect_err("one byte below the measured source-verification peak must deny");
    assert!(matches!(
        denial,
        worth_store_offline_verifier::BackupCutSourceVerificationDenial::OwnedAllocationBudgetExceeded {
            required,
            limit,
        } if required > limit && limit == exact_peak - 1
    ));
}
