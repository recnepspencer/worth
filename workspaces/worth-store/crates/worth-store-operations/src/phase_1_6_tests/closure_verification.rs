use super::support::*;

#[test]
fn independent_verification_rejects_cross_component_wal_closure_tampering() {
    let scenario = BackupScenario::new("wal-closure-defect");
    let authority = scenario.authority();
    let control = scenario.control_store();
    let admitted = OnlineBackupIntent::new(
        OperationalOperationId::new("backup-wal-closure-defect").expect("operation id"),
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
    let original = materialized.manifest();
    let mut rows = original.artifacts().to_vec();
    let wal = rows
        .iter_mut()
        .find(|row| {
            row.family() == worth_store_physical_format::BackupBundleArtifactFamily::WalSegment
        })
        .expect("WAL row");
    *wal = worth_store_physical_format::BackupBundleArtifactManifestRow::new(
        wal.family(),
        wal.format(),
        wal.identity(),
        wal.output_name(),
        wal.generation(),
        wal.bytes(),
        wal.content_digest(),
        worth_store_physical_format::BackupBundleArtifactCoverage::WalSegment {
            start_lsn: 10,
            end_exclusive_lsn: 11,
        },
        wal.reclaim_owner(),
    )
    .expect("individually valid defective WAL row");
    let defective = BackupBundleManifest::canonical(
        worth_store_physical_format::BackupBundleManifestDeclaration::from_manifest_with_artifacts(
            original, rows,
        ),
    )
    .expect("structurally encoded producer defect");
    std::fs::write(
        materialized.root().join("backup.manifest"),
        BackupBundleFormatAuthority::canonical()
            .encode_manifest(&defective)
            .expect("defective manifest"),
    )
    .expect("write controlled producer defect");
    let independently_opened = BackupBundleFormatAuthority::canonical()
        .admit_materialized(materialized.root())
        .expect("fresh process admits the individually valid rows");

    let result = verify_materialized_backup(
        independently_opened,
        OfflineInspectionBudget::bounded(4 * 1024, scenario.total_bytes() + 64 * 1024)
            .expect("budget"),
    );
    match result {
        Err(BackupStructuralVerificationDenial::Defects(report)) => {
            assert!(report
                .defects()
                .contains(&BackupVerificationDefect::WalCoverageGapOrOverlap));
        }
        other => panic!("broken cross-component WAL closure must fail: {other:?}"),
    }
}

#[test]
fn structurally_valid_bundle_omission_cannot_release_a_larger_admitted_cut() {
    let scenario = BackupScenario::new("omitted-reachable-page");
    let extra_path = scenario.source.join("extra.page");
    let canonical_page = scenario
        .references()
        .iter()
        .find(|artifact| artifact.family() == BackupArtifactFamily::Page)
        .expect("canonical physical page");
    let extra_owner = reclaim_reference(BackupArtifactFamily::Page, 55);
    crate::certification_scenario::backup_artifacts::copy_page_to_owner(
        canonical_page.source_path(),
        &extra_path,
        extra_owner,
    );
    let extra = BackupArtifactReference::declare_untrusted_physical_observation(
        UntrustedBackupArtifactClaim {
            family: BackupArtifactFamily::Page,
            format: artifact_format(BackupArtifactFamily::Page),
            identity: "extra-page".to_owned(),
            generation: 1,
            coverage: BackupArtifactCoverage::physical_reachability(),
        },
        observe_physical_backup_artifact(extra_path, 4 * 1024).expect("observation"),
        extra_owner,
    )
    .expect("extra reachable page");
    let mut complete = scenario.references().to_vec();
    complete.push(extra);

    let authority = scenario.authority();
    let control = scenario.control_store();
    let operation =
        OperationalOperationId::new("backup-omitted-reachable-page").expect("operation");
    let admitted = OnlineBackupIntent::new(
        operation.clone(),
        scenario.coordinates(),
        BackupCutManifest::canonical(complete).expect("complete source closure"),
        backup_custody(&authority),
    )
    .admit_cut(&authority, &control, &scenario.leases)
    .expect("complete cut");
    let completion = admitted
        .materialize(&scenario.target, 7, &control)
        .expect("session")
        .finish()
        .expect("materialization");
    let (materialized, cut) = completion.into_parts();
    let manifest = materialized.manifest();
    let omitted = manifest
        .artifacts()
        .iter()
        .find(|row| row.identity() == "extra-page")
        .expect("extra row")
        .output_name()
        .to_owned();
    let reduced = BackupBundleManifest::canonical(
        worth_store_physical_format::BackupBundleManifestDeclaration::from_manifest_with_artifacts(
            manifest,
            manifest
                .artifacts()
                .iter()
                .filter(|row| row.identity() != "extra-page")
                .cloned()
                .collect(),
        ),
    )
    .expect("internally valid reduced bundle");
    std::fs::remove_file(materialized.root().join(omitted)).expect("omit component");
    std::fs::write(
        materialized.root().join("backup.manifest"),
        BackupBundleFormatAuthority::canonical()
            .encode_manifest(&reduced)
            .expect("reduced manifest"),
    )
    .expect("publish controlled producer defect");
    let independently_opened = BackupBundleFormatAuthority::canonical()
        .admit_materialized(materialized.root())
        .expect("fresh process admits reduced format");
    let structural = verify_materialized_backup(
        independently_opened,
        OfflineInspectionBudget::bounded(4 * 1024, u64::MAX).expect("budget"),
    )
    .expect("reduced bundle is internally structural");
    assert!(matches!(
        record_independent_backup_verification(
            &operation,
            structural,
            cut,
            &control,
            &scenario.leases,
        ),
        Err(crate::BackupVerificationJoinDenial::WrongCut(_))
    ));
    assert_eq!(
        scenario
            .leases
            .live_index_snapshot()
            .expect("lease remains live")
            .active_leases(),
        1
    );
}

#[test]
fn independently_decodable_files_cannot_claim_one_physical_owner_twice() {
    let scenario = BackupScenario::new("duplicate-physical-owner");
    let authority = scenario.authority();
    let control = scenario.control_store();
    let admitted = OnlineBackupIntent::new(
        OperationalOperationId::new("backup-duplicate-physical-owner").expect("operation"),
        scenario.coordinates(),
        scenario.cut_manifest(),
        backup_custody(&authority),
    )
    .admit_cut(&authority, &control, &scenario.leases)
    .expect("cut");
    let completion = admitted
        .materialize(&scenario.target, 19, &control)
        .expect("session")
        .finish()
        .expect("materialize");
    let (materialized, _cut) = completion.into_parts();
    let original = materialized.manifest();
    let page = original
        .artifacts()
        .iter()
        .find(|row| row.family() == worth_store_physical_format::BackupBundleArtifactFamily::Page)
        .expect("page row");
    let duplicate_name = "duplicate-owner.page";
    std::fs::copy(
        materialized.root().join(page.output_name()),
        materialized.root().join(duplicate_name),
    )
    .expect("duplicate physical claim bytes");
    let duplicate = worth_store_physical_format::BackupBundleArtifactManifestRow::new(
        page.family(),
        page.format(),
        "duplicate-page-identity",
        duplicate_name,
        page.generation(),
        page.bytes(),
        page.content_digest(),
        page.coverage().clone(),
        page.reclaim_owner(),
    )
    .expect("individually valid duplicate owner row");
    let mut rows = original.artifacts().to_vec();
    rows.push(duplicate);
    let defective = BackupBundleManifest::canonical(
        worth_store_physical_format::BackupBundleManifestDeclaration::from_manifest_with_artifacts(
            original, rows,
        ),
    )
    .expect("duplicate physical ownership remains representable for hostile verification");
    std::fs::write(
        materialized.root().join("backup.manifest"),
        BackupBundleFormatAuthority::canonical()
            .encode_manifest(&defective)
            .expect("defective manifest"),
    )
    .expect("write controlled producer defect");
    let independently_opened = BackupBundleFormatAuthority::canonical()
        .admit_materialized(materialized.root())
        .expect("fresh process admits the individually valid duplicate rows");

    let result = verify_materialized_backup(
        independently_opened,
        OfflineInspectionBudget::bounded(4 * 1024, scenario.total_bytes() + 128 * 1024)
            .expect("budget"),
    );
    match result {
        Err(BackupStructuralVerificationDenial::PhysicalOwnershipOverlap) => {}
        other => panic!("duplicate physical ownership must fail at the ownership seam: {other:?}"),
    }
}
