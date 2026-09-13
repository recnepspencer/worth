use super::current_root_fixture::*;
use super::support::*;

#[test]
fn production_current_root_source_protects_one_page_allocation_for_multiple_slots() {
    let scenario = BackupScenario::new("runtime-root-closure");
    let fixture = current_root_fixture_with_shared_page();
    let source = fixture.source();
    assert_eq!(source.manifest().page_slots().len(), 2);
    assert_eq!(source.page_cells().len(), 1);

    let mut artifacts = scenario
        .references()
        .iter()
        .filter(|artifact| {
            !matches!(
                artifact.family(),
                BackupArtifactFamily::RootManifest
                    | BackupArtifactFamily::Page
                    | BackupArtifactFamily::Extent
            )
        })
        .cloned()
        .collect::<Vec<_>>();
    artifacts.extend(core_references_for_source(&fixture, &scenario.source));
    let manifest = BackupCutManifest::from_current_root_source(source, artifacts.clone())
        .expect("runtime-issued reachability must admit");
    assert_eq!(
        manifest
            .artifacts()
            .iter()
            .filter(|artifact| artifact.family() == BackupArtifactFamily::Page)
            .count(),
        1
    );

    let coordinates = BackupCutCoordinates::new(
        "lineage-a",
        source.manifest().root_publication().generation().get(),
        1,
        scenario.checkpoint_identity(),
        10,
        10,
        12,
        12,
        "worth-physical-format-v1",
        "posix-file-fsync-dir-sync",
    )
    .expect("runtime cut coordinates");
    let authority = crate::backup::export::current_authority("store.physical.default_instance");
    let admitted = OnlineBackupIntent::new(
        OperationalOperationId::new("runtime-root-source-admission").expect("operation"),
        coordinates,
        manifest,
        backup_custody(&authority),
    )
    .admit_cut(&authority, &scenario.control_store(), &scenario.leases)
    .expect("current runtime authority must admit its own physical source");
    assert_eq!(
        admitted.cut().authority_identity(),
        source.store_authority_identity()
    );

    let page = artifacts
        .iter()
        .position(|artifact| artifact.family() == BackupArtifactFamily::Page)
        .expect("page artifact");
    artifacts.remove(page);
    assert!(matches!(
        BackupCutManifest::from_current_root_source(source, artifacts),
        Err(BackupCutManifestDenial::MissingOwnerReachability)
    ));
}

#[test]
fn cut_identity_binds_acknowledged_frontier_and_rejects_unproven_profiles() {
    let scenario = BackupScenario::new("cut-coordinate-identity");
    let authority = scenario.authority();
    let control = scenario.control_store();
    let admit = |operation: &str, acknowledged_frontier, format, backend| {
        OnlineBackupIntent::new(
            OperationalOperationId::new(operation).expect("operation"),
            BackupCutCoordinates::new(
                "lineage-a",
                1,
                1,
                scenario.checkpoint_identity(),
                10,
                10,
                12,
                acknowledged_frontier,
                format,
                backend,
            )
            .expect("coordinates"),
            scenario.cut_manifest(),
            backup_custody(&authority),
        )
        .admit_cut(&authority, &control, &scenario.leases)
        .expect("cut")
        .cut()
        .identity()
    };

    let baseline = admit(
        "cut-coordinate-baseline",
        12,
        "worth-physical-format-v1",
        "posix-file-fsync-dir-sync",
    );
    let later_ack = admit(
        "cut-coordinate-ack",
        13,
        "worth-physical-format-v1",
        "posix-file-fsync-dir-sync",
    );

    assert_ne!(baseline, later_ack);

    for (operation, format, backend, expected) in [
        (
            "cut-coordinate-format",
            "invented-format",
            "posix-file-fsync-dir-sync",
            BackupCutAdmissionDenial::FormatProfileMismatch,
        ),
        (
            "cut-coordinate-backend",
            "worth-physical-format-v1",
            "invented-backend",
            BackupCutAdmissionDenial::BackendProfileMismatch,
        ),
    ] {
        let coordinates = BackupCutCoordinates::new(
            "lineage-a",
            1,
            1,
            scenario.checkpoint_identity(),
            10,
            10,
            12,
            12,
            format,
            backend,
        )
        .expect("individually representable coordinates");
        let denial = OnlineBackupIntent::new(
            OperationalOperationId::new(operation).expect("operation"),
            coordinates,
            scenario.cut_manifest(),
            backup_custody(&authority),
        )
        .admit_cut(&authority, &control, &scenario.leases)
        .expect_err("unproven profile must fail before source admission");
        assert!(matches!(denial, OnlineBackupAdmissionDenial::Cut(actual) if actual == expected));
    }
}

#[test]
fn source_replacement_after_cut_admission_fails_before_output_allocation() {
    let scenario = BackupScenario::new("source-replacement");
    let authority = scenario.authority();
    let control = scenario.control_store();
    let admitted = OnlineBackupIntent::new(
        OperationalOperationId::new("backup-source-replacement").expect("operation"),
        scenario.coordinates(),
        scenario.cut_manifest(),
        backup_custody(&authority),
    )
    .admit_cut(&authority, &control, &scenario.leases)
    .expect("cut");
    let source = scenario.references()[0].source_path().to_path_buf();
    let identical_bytes = std::fs::read(&source).expect("source bytes");
    std::fs::remove_file(&source).expect("remove admitted allocation");
    std::fs::write(&source, identical_bytes).expect("replace with identical bytes");
    assert!(matches!(
        admitted.materialize(&scenario.target, 7, &control),
        Err(BackupMaterializationDenial::Physical(
            worth_store_physical_backend::PhysicalBackupMaterializationDenial::SourceIdentityMismatch { .. }
        ))
    ));
    assert_eq!(
        std::fs::read_dir(&scenario.target).expect("target").count(),
        0
    );
}

#[test]
fn backup_cut_admission_rejects_manifest_coordinates_that_name_another_root() {
    let scenario = BackupScenario::new("root-mismatch");
    let authority = scenario.authority();
    let control = scenario.control_store();
    let mismatched = BackupCutCoordinates::new(
        "lineage-a",
        2,
        1,
        scenario.checkpoint_identity(),
        10,
        10,
        12,
        12,
        "worth-physical-format-v1",
        "posix-file-fsync-dir-sync",
    )
    .expect("individually valid coordinates");

    let operation = OperationalOperationId::new("backup-root-mismatch").expect("operation");
    let denial = OnlineBackupIntent::new(
        operation.clone(),
        mismatched,
        scenario.cut_manifest(),
        backup_custody(&authority),
    )
    .admit_cut(&authority, &control, &scenario.leases)
    .expect_err("cross-generation cut must fail");
    assert!(matches!(
        denial,
        OnlineBackupAdmissionDenial::Cut(BackupCutAdmissionDenial::RootGenerationMismatch)
    ));
    assert_eq!(
        control
            .observe_selection_coordinates()
            .expect("control observation"),
        None,
        "a rejected cut must not durably open an unrecoverable workflow"
    );

    let admitted = OnlineBackupIntent::new(
        operation,
        scenario.coordinates(),
        scenario.cut_manifest(),
        backup_custody(&authority),
    )
    .admit_cut(&authority, &control, &scenario.leases)
    .expect("the same operation identity remains usable after preflight denial");
    admitted
        .abandon("test closeout", &control, &scenario.leases)
        .expect("durable closeout");
}

#[test]
fn checkpoint_frontier_cannot_claim_a_different_live_store_authority() {
    let scenario = BackupScenario::new("checkpoint-authority-mismatch");
    let authority = scenario.authority();
    let control = scenario.control_store();
    let checkpoint = scenario
        .references()
        .iter()
        .find(|artifact| artifact.family() == BackupArtifactFamily::CheckpointManifest)
        .expect("checkpoint artifact");
    let BackupArtifactCoverage::CheckpointManifest {
        checkpoint_identity,
        manifest_generation,
        durable_checkpoint_lsn,
        frontier_digest,
        ..
    } = checkpoint.coverage()
    else {
        panic!("checkpoint carries checkpoint coverage");
    };
    let forged = BackupArtifactReference::declare_untrusted_physical_observation(
        UntrustedBackupArtifactClaim {
            family: BackupArtifactFamily::CheckpointManifest,
            format: checkpoint.format(),
            identity: checkpoint.identity().to_owned(),
            generation: checkpoint.generation(),
            coverage: BackupArtifactCoverage::checkpoint_manifest(
                checkpoint_identity,
                *manifest_generation,
                *durable_checkpoint_lsn,
                [0xa5; 32],
                *frontier_digest,
            )
            .expect("forged checkpoint coverage remains structurally valid"),
        },
        observe_physical_backup_artifact(checkpoint.source_path().to_path_buf(), 4 * 1024)
            .expect("checkpoint observation"),
        checkpoint.reclaim_reference(),
    )
    .expect("forged checkpoint reference");
    let mut references = scenario.references().to_vec();
    let checkpoint_index = references
        .iter()
        .position(|artifact| artifact.family() == BackupArtifactFamily::CheckpointManifest)
        .expect("checkpoint index");
    references[checkpoint_index] = forged;
    let denial = OnlineBackupIntent::new(
        OperationalOperationId::new("checkpoint-authority-mismatch").expect("operation"),
        scenario.coordinates(),
        BackupCutManifest::canonical(references).expect("structurally valid cut"),
        backup_custody(&authority),
    )
    .admit_cut(&authority, &control, &scenario.leases)
    .expect_err("checkpoint authority substitution must fail closed");
    assert!(matches!(
        denial,
        OnlineBackupAdmissionDenial::Cut(BackupCutAdmissionDenial::CheckpointFrontierMismatch)
    ));
}

#[test]
fn custody_admitted_by_another_store_authority_cannot_open_a_backup_cut() {
    let scenario = BackupScenario::new("foreign-custody-authority");
    let current = scenario.authority();
    let foreign = crate::backup::export::current_authority("s10-foreign-custody-authority");
    let control = scenario.control_store();

    let denial = OnlineBackupIntent::new(
        OperationalOperationId::new("foreign-custody-authority").expect("operation"),
        scenario.coordinates(),
        scenario.cut_manifest(),
        backup_custody(&foreign),
    )
    .admit_cut(&current, &control, &scenario.leases)
    .expect_err("scope labels cannot launder another Store's custody authority");

    assert!(matches!(
        denial,
        OnlineBackupAdmissionDenial::Cut(BackupCutAdmissionDenial::SecurityScopeAuthorityMismatch)
    ));
    assert_eq!(
        control
            .observe_selection_coordinates()
            .expect("control observation"),
        None
    );
}
