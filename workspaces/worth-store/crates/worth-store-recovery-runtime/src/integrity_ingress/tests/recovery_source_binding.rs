use worth_proof::TransitionOutcome;
use worth_store::physical_runtime::{
    FilesystemAccessPosture, FilesystemMediaAdmission, PhysicalRuntimeAdmission, PhysicalStore,
    QualifiedRecoveryFilesystemMedia,
};
use worth_store_physical_format::{
    BootstrapCatalog, CurrentRootCatalogEntry, CurrentRootCatalogGeneration, DurableRootSelector,
    PhysicalRecordFormatDeclaration, RootSelectorIdentity, RootSelectorRole, ROOT_SELECTOR_BYTES,
};
use worth_store_physical_integrity::{
    validate_bootstrap_catalog, validate_current_root_selector,
    CurrentRootSelectorIntegrityValidation, PhysicalArtifactScope, PhysicalByteRange,
};

use super::super::admitted_artifact::IntegrityAdmittedRecoveryArtifact;
use super::super::{
    ObservedRecoverySource, RecoveryIntegrityIngressCounters,
    RecoveryIntegrityIngressObservationOutcome, RecoveryIntegrityIngressRejection,
};

#[test]
fn exact_c4_incarnation_and_scope_gate_typed_projection() {
    let parent = tempfile::tempdir().expect("test parent");
    let root = parent.path().join("recovery-source-binding");
    let runtime =
        PhysicalStore::admit(PhysicalRuntimeAdmission::new(root.clone()).expect("declared root"))
            .expect("ordinary runtime admission");
    let admission =
        FilesystemMediaAdmission::production(FilesystemAccessPosture::CoordinatedServiceAccount);
    let media = match runtime.try_admit_filesystem_media(admission).into_raw() {
        TransitionOutcome::Success(media) => media,
        _ => panic!("ordinary media initialization failed"),
    };
    let store = media.store_identity();
    let _ = media.close();
    let format = PhysicalRecordFormatDeclaration::builder().admit().unwrap();
    let selector = DurableRootSelector::new(
        store,
        format,
        RootSelectorIdentity::new(1).unwrap(),
        RootSelectorRole::Current,
        1,
        None,
        None,
    )
    .unwrap();
    let catalog = BootstrapCatalog::new(
        store,
        format,
        CurrentRootCatalogEntry::new(CurrentRootCatalogGeneration::new(1).unwrap()),
    );
    let records = root.join("families").join("records");
    std::fs::create_dir_all(&records).unwrap();
    std::fs::write(records.join("root-current.selector"), selector.encode()).unwrap();
    std::fs::write(records.join("bootstrap.catalog"), catalog.encode()).unwrap();

    let media = QualifiedRecoveryFilesystemMedia::qualify_existing(&root)
        .unwrap()
        .admit_persisted_store()
        .unwrap();
    let mut discovery = media.bounded_discovery(4, 4096).unwrap();
    let source_a = discovery
        .read_current_selector(ROOT_SELECTOR_BYTES as u64)
        .unwrap();
    let source_b = discovery
        .read_current_selector(ROOT_SELECTOR_BYTES as u64)
        .unwrap();
    let bootstrap_source = discovery.read_bootstrap_catalog(256).unwrap();
    let bootstrap_source_b = discovery.read_bootstrap_catalog(256).unwrap();
    let selector_scope = PhysicalArtifactScope::current_root_selector(
        store,
        format,
        PhysicalByteRange::new(0, ROOT_SELECTOR_BYTES as u64).unwrap(),
    );

    let validation = validate_selector(&source_a, selector_scope);
    let mut counters = RecoveryIntegrityIngressCounters::default();
    let admitted = IntegrityAdmittedRecoveryArtifact::bind_current_selector(
        &source_a,
        selector_scope,
        validation,
        &mut counters,
    );
    assert_eq!(
        admitted.observation().outcome(),
        RecoveryIntegrityIngressObservationOutcome::Admitted
    );
    let IntegrityAdmittedRecoveryArtifact::CurrentSelector(admitted) =
        admitted.into_outcome().unwrap()
    else {
        panic!("current selector admission routed to the wrong family")
    };
    assert_eq!(
        admitted.project_for_recovery(&mut counters).role(),
        RootSelectorRole::Current
    );

    let validation = validate_selector(&source_a, selector_scope);
    let substituted = IntegrityAdmittedRecoveryArtifact::bind_current_selector(
        &source_b,
        selector_scope,
        validation,
        &mut counters,
    );
    assert_eq!(
        substituted.observation().outcome(),
        RecoveryIntegrityIngressObservationOutcome::Rejected(
            RecoveryIntegrityIngressRejection::SourceIncarnationMismatch
        )
    );

    let bootstrap_scope = PhysicalArtifactScope::bootstrap_catalog(
        store,
        format,
        PhysicalByteRange::new(0, catalog.encode().len() as u64).unwrap(),
    );
    let input = ObservedRecoverySource::complete(&bootstrap_source, bootstrap_scope)
        .input()
        .unwrap();
    let (validation, _) = validate_bootstrap_catalog(input, bootstrap_scope);
    let admitted = IntegrityAdmittedRecoveryArtifact::bind_bootstrap_catalog(
        &bootstrap_source,
        bootstrap_scope,
        validation,
        &mut counters,
    );
    let IntegrityAdmittedRecoveryArtifact::BootstrapCatalog(admitted) =
        admitted.into_outcome().unwrap()
    else {
        panic!("bootstrap admission routed to the wrong family")
    };
    let projection = admitted.project(&mut counters);
    assert_eq!(projection.record_format, format);
    assert_eq!(projection.current_root_generation.get(), 1);
    let bootstrap_input = ObservedRecoverySource::complete(&bootstrap_source, bootstrap_scope)
        .input()
        .unwrap();
    let (bootstrap_validation, _) = validate_bootstrap_catalog(bootstrap_input, bootstrap_scope);
    let substituted_bootstrap = IntegrityAdmittedRecoveryArtifact::bind_bootstrap_catalog(
        &bootstrap_source_b,
        bootstrap_scope,
        bootstrap_validation,
        &mut counters,
    );
    assert_eq!(
        substituted_bootstrap.observation().outcome(),
        RecoveryIntegrityIngressObservationOutcome::Rejected(
            RecoveryIntegrityIngressRejection::SourceIncarnationMismatch
        )
    );
    assert_eq!((counters.attempted, counters.admitted), (4, 2));
    assert_eq!(counters.rejected_source_binding, 2);
    assert_eq!(counters.owner_projection_entries, 2);
    assert_eq!(counters.owner_decoder_entries, 0);

    let validation = validate_selector(&source_a, selector_scope);
    let wrong_scope =
        PhysicalArtifactScope::previous_root_selector(store, format, selector_scope.byte_range());
    assert!(matches!(
        IntegrityAdmittedRecoveryArtifact::bind_current_selector(
            &source_a,
            wrong_scope,
            validation,
            &mut counters,
        )
        .into_outcome(),
        Err(RecoveryIntegrityIngressRejection::ScopeMismatch)
    ));
    drop(discovery.finish());
}

fn validate_selector<'media>(
    source: &'media worth_store::physical_runtime::ObservedRecoveryArtifact,
    scope: PhysicalArtifactScope,
) -> CurrentRootSelectorIntegrityValidation<'media> {
    let input = ObservedRecoverySource::complete(source, scope)
        .input()
        .expect("present selector");
    validate_current_root_selector(input, scope).0
}
