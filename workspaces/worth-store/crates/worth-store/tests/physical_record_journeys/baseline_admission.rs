use worth_proof::TransitionOutcome;
use worth_store::physical_runtime::{
    AdmittedPhysicalRecordFormat, ManifestEntryCapacity, PhysicalManifestCapacityTransition,
    PhysicalMutationIdempotencyMaterial, PhysicalPageSizeClass, PhysicalRecordAccessPolicy,
    PhysicalRecordFormatDeclaration, PhysicalRecordInitialization, PhysicalRecordOpen,
    PhysicalRecordPlacementPolicy, PhysicalRootProtocolRoute, RecordAppendBatch,
    RecordBootstrapDenial, RecordByteLimit, RecordServingRebindReason,
    RecordStoreInitializationOutcome, RecordStoreOpenOutcome,
};
use worth_store_physical_backend::MediaOperationRole;

use super::{
    configuration, durability, media, serving_from_initialization, serving_from_open, success,
};

#[test]
fn foreign_durability_policy_rebinds_before_any_target_media_effect() {
    let parent = tempfile::tempdir().unwrap();
    let source = media(&parent.path().join("source"));
    let target = media(&parent.path().join("target"));
    let policy = durability(&source);
    let (format, _, access) = configuration();
    let before = target.media_counters();

    let outcome = target
        .open_record_store(PhysicalRecordOpen::new(format, access, policy))
        .into_raw();
    let TransitionOutcome::RebindRequired(rebind) = outcome else {
        panic!("a durability policy from another Store must require rebinding");
    };
    assert_eq!(
        rebind.reason(),
        RecordServingRebindReason::PhysicalDurabilityStoreMismatch,
    );
    let target = rebind.into_runtime();
    assert_eq!(target.media_counters(), before);
    target.close();
    source.close();
}

#[test]
fn empty_bootstrap_create_and_reopen_converge() {
    let parent = tempfile::tempdir().unwrap();
    let root = parent.path().join("store");
    let serving = serving_from_initialization(&root);
    let store = serving.store_identity();
    let initialization = serving.root_protocol_counters();
    assert_eq!(
        initialization.root_entries(PhysicalRootProtocolRoute::Initialization),
        1,
        "initialization must enter its root only after resident admission",
    );
    assert_eq!(
        initialization.selector_entries(PhysicalRootProtocolRoute::Initialization),
        0,
        "initialization publishes its selector without reading it",
    );
    assert!(root.join("families/records/bootstrap.catalog").is_file());
    assert!(root
        .join("families/records/roots/root-0000000000000001.manifest")
        .is_file());
    let expected_reopen_bytes = super::durable_frame_oracle::artifact_bytes(
        &root,
        &[
            "families/records/bootstrap.catalog",
            "families/records/roots/root-0000000000000001.manifest",
            "families/records/free-space/free-space-0000000000000001.manifest",
        ],
    );
    serving.close();

    let (format, _, access) = configuration();
    let open_media = media(&root);
    let before = open_media.media_counters();
    let reopened = success(open_record_store!(open_media, |durability| {
        PhysicalRecordOpen::new(format, access, durability)
    }));
    let after = reopened.media_counters();
    assert_eq!(reopened.store_identity(), store);
    assert_eq!(
        reopened
            .root_protocol_counters()
            .root_entries(PhysicalRootProtocolRoute::OrdinaryOpen),
        1,
        "ordinary open must enter root interpretation exactly once after admission",
    );
    assert_eq!(
        reopened
            .root_protocol_counters()
            .selector_entries(PhysicalRootProtocolRoute::OrdinaryOpen),
        0,
        "ordinary open has no Store-private selector decode route",
    );
    assert!(!reopened.observed_staging_residue());
    assert_eq!(
        after.attempts_for(MediaOperationRole::PositionedRead)
            - before.attempts_for(MediaOperationRole::PositionedRead),
        3
    );
    assert_eq!(
        after.completed_bytes_for(MediaOperationRole::PositionedRead)
            - before.completed_bytes_for(MediaOperationRole::PositionedRead),
        expected_reopen_bytes
    );
    assert_eq!(after.replacements(), before.replacements());
    reopened.close();
}

#[test]
fn initialize_and_open_never_substitute_for_each_other() {
    let parent = tempfile::tempdir().unwrap();
    let root = parent.path().join("store");
    let (format, placement, access) = configuration();
    let absent = media(&root);
    let open_outcome: RecordStoreOpenOutcome =
        open_record_store!(absent, |durability| PhysicalRecordOpen::new(
            format, access, durability
        ));
    let absent = match open_outcome.into_raw() {
        TransitionOutcome::Denied(denial) => denial.into_runtime(),
        _ => panic!("C5_PREDICATE:lifecycle open must not initialize an absent record family"),
    };
    success(initialize_record_store!(absent, |durability| {
        PhysicalRecordInitialization::new(format, placement, access, durability)
    }))
    .close();

    let existing = media(&root);
    let existing: RecordStoreInitializationOutcome = initialize_record_store!(
        existing,
        |durability| PhysicalRecordInitialization::new(format, placement, access, durability)
    );
    assert!(matches!(existing.into_raw(), TransitionOutcome::Denied(_)));
}

#[test]
fn operational_policy_drift_reopens_but_format_drift_does_not() {
    let parent = tempfile::tempdir().unwrap();
    let root = parent.path().join("store");
    serving_from_initialization(&root).close();

    let (format, _, _) = configuration();
    let wider_access = PhysicalRecordAccessPolicy::builder()
        .transfer_limit(RecordByteLimit::new(32_768).unwrap())
        .scratch_limit(RecordByteLimit::new(32_768).unwrap())
        .admit(format)
        .unwrap();
    let reopened = success(open_record_store!(media(&root), |durability| {
        PhysicalRecordOpen::new(format, wider_access, durability)
    }));
    let changed_placement = PhysicalRecordPlacementPolicy::builder()
        .manifest_capacity(ManifestEntryCapacity::new(128).unwrap())
        .admit(format)
        .unwrap();
    let completed = super::durable_publication::publish_single_with_manifest_capacity_transition(
        &reopened,
        changed_placement,
        PhysicalManifestCapacityTransition::ReconstructToRequested,
        PhysicalMutationIdempotencyMaterial::new([165; 32]),
        RecordAppendBatch::try_from_iter([b"policy drift".as_slice()]).unwrap(),
    );
    assert_eq!(completed.current_root().generation(), 2);
    assert_eq!(completed.current_root().node_capacity(), 128);
    reopened.close();

    let changed_format = AdmittedPhysicalRecordFormat::admit(
        PhysicalRecordFormatDeclaration::builder()
            .page_size(PhysicalPageSizeClass::KiB32)
            .admit()
            .unwrap(),
    );
    let changed_access = PhysicalRecordAccessPolicy::builder()
        .admit(changed_format)
        .unwrap();
    let wrong_media = media(&root);
    let replacements = wrong_media.media_counters().replacements();
    let rejected = open_record_store!(wrong_media, |durability| PhysicalRecordOpen::new(
        changed_format,
        changed_access,
        durability
    ))
    .into_raw();
    let TransitionOutcome::Denied(denial) = rejected else {
        panic!("persisted format drift must be denied");
    };
    let RecordBootstrapDenial::PhysicalRecordFormatMismatch(mismatch) = denial.reason() else {
        panic!("format mismatch must retain both exact declarations")
    };
    assert_eq!(mismatch.expected(), changed_format.declaration());
    assert_eq!(mismatch.persisted(), format.declaration());
    let returned = denial.into_runtime();
    assert_eq!(returned.media_counters().replacements(), replacements);
    returned.close();
}

#[test]
fn namespace_residue_cannot_elect_current_truth() {
    let parent = tempfile::tempdir().unwrap();
    let root = parent.path().join("store");
    serving_from_initialization(&root).close();
    let catalog = root.join("families/records/bootstrap.catalog");
    std::fs::copy(
        &catalog,
        root.join("staging/records/bootstrap-0000000000000002.candidate"),
    )
    .unwrap();
    std::fs::write(
        root.join("staging/records/duplicate.candidate"),
        b"duplicate",
    )
    .unwrap();
    let selected = root.join("families/records/roots/root-0000000000000001.manifest");
    std::fs::copy(
        &selected,
        root.join("families/records/roots/root-0000000000000000.manifest"),
    )
    .unwrap();
    std::fs::copy(
        &selected,
        root.join("families/records/roots/root-0000000000000002.manifest"),
    )
    .unwrap();
    let foreign_root = parent.path().join("foreign-store");
    serving_from_initialization(&foreign_root).close();
    std::fs::copy(
        foreign_root.join("families/records/roots/root-0000000000000001.manifest"),
        root.join("families/records/roots/root-ffffffffffffffff.manifest"),
    )
    .unwrap();
    let reopened = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        serving_from_open(&root)
    }))
    .unwrap_or_else(|_| {
        panic!(
            "C5_PREDICATE:independent-decision-path: unselected root artifacts cannot elect current truth"
        )
    });
    assert!(reopened.observed_non_authoritative_residue());
    reopened.close();
}
