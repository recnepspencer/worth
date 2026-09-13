use super::support::*;

#[test]
fn backup_cut_manifest_rejects_two_names_for_one_physical_allocation() {
    let directory = tempfile::tempdir().expect("temp directory");
    let first_path = directory.path().join("first.page");
    let alias_path = directory.path().join("alias.extent");
    std::fs::write(&first_path, b"shared-allocation").expect("media");
    std::fs::hard_link(&first_path, &alias_path).expect("hard-link alias");
    let first = BackupArtifactReference::declare_untrusted_physical_observation(
        UntrustedBackupArtifactClaim {
            family: BackupArtifactFamily::Page,
            format: artifact_format(BackupArtifactFamily::Page),
            identity: "page-a".to_owned(),
            generation: 1,
            coverage: artifact_coverage(BackupArtifactFamily::Page),
        },
        observe_physical_backup_artifact(first_path, 4).expect("first observation"),
        reclaim_reference(BackupArtifactFamily::Page, 20),
    )
    .expect("reference");
    let alias = BackupArtifactReference::declare_untrusted_physical_observation(
        UntrustedBackupArtifactClaim {
            family: BackupArtifactFamily::Extent,
            format: artifact_format(BackupArtifactFamily::Extent),
            identity: "extent-a".to_owned(),
            generation: 1,
            coverage: artifact_coverage(BackupArtifactFamily::Extent),
        },
        observe_physical_backup_artifact(alias_path, 4).expect("alias observation"),
        reclaim_reference(BackupArtifactFamily::Extent, 21),
    )
    .expect("reference");
    assert!(matches!(
        BackupCutManifest::canonical([first, alias]),
        Err(BackupCutManifestDenial::DuplicatePhysicalArtifact)
    ));
}

#[test]
fn artifact_family_cannot_be_paired_with_an_unrelated_reclaim_domain() {
    let directory = tempfile::tempdir().expect("temp directory");
    let path = directory.path().join("page.media");
    std::fs::write(&path, b"page").expect("media");
    assert!(
        BackupArtifactReference::declare_untrusted_physical_observation(
            UntrustedBackupArtifactClaim {
                family: BackupArtifactFamily::Page,
                format: artifact_format(BackupArtifactFamily::Page),
                identity: "page-a".to_owned(),
                generation: 1,
                coverage: BackupArtifactCoverage::physical_reachability(),
            },
            observe_physical_backup_artifact(path, 4).expect("observation"),
            reclaim_reference(BackupArtifactFamily::Extent, 8),
        )
        .is_none()
    );
}

#[test]
fn cut_identity_binds_the_exact_physical_lease_owners() {
    let scenario = BackupScenario::new("lease-owner-identity");
    let authority = scenario.authority();
    let control = scenario.control_store();
    let first = OnlineBackupIntent::new(
        OperationalOperationId::new("backup-owner-one").expect("operation"),
        scenario.coordinates(),
        scenario.cut_manifest(),
        backup_custody(&authority),
    )
    .admit_cut(&authority, &control, &scenario.leases)
    .expect("first cut");

    let mut changed = scenario.references().to_vec();
    let page_index = changed
        .iter()
        .position(|artifact| artifact.family() == BackupArtifactFamily::Page)
        .expect("page artifact");
    let page = &changed[page_index];
    let changed_owner = reclaim_reference(BackupArtifactFamily::Page, 99);
    let changed_path = scenario.source.join("changed-owner.page");
    crate::certification_scenario::backup_artifacts::copy_page_to_owner(
        page.source_path(),
        &changed_path,
        changed_owner,
    );
    changed[page_index] = BackupArtifactReference::declare_untrusted_physical_observation(
        UntrustedBackupArtifactClaim {
            family: BackupArtifactFamily::Page,
            format: page.format(),
            identity: page.identity().to_owned(),
            generation: page.generation(),
            coverage: page.coverage().clone(),
        },
        observe_physical_backup_artifact(changed_path, 5).expect("observation"),
        changed_owner,
    )
    .expect("changed physical owner");
    let second = OnlineBackupIntent::new(
        OperationalOperationId::new("backup-owner-two").expect("operation"),
        scenario.coordinates(),
        BackupCutManifest::canonical(changed).expect("changed manifest"),
        backup_custody(&authority),
    )
    .admit_cut(&authority, &control, &scenario.leases)
    .expect("second cut");
    assert_ne!(first.cut().identity(), second.cut().identity());
}
