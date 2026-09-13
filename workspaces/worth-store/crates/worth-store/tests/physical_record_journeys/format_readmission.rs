use worth_proof::TransitionOutcome;
use worth_store::physical_runtime::{
    AdmittedPhysicalRecordFormat, PhysicalPageSizeClass, PhysicalRecordAccessPolicy,
    PhysicalRecordFormatDeclaration, PhysicalRecordOpen, PhysicalRootProtocolRoute,
    RecordBootstrapDenial, RecordServingRebindReason,
};
use worth_store_physical_backend::MediaOperationRole;
use worth_store_physical_format::PhysicalRecordFormatDenial;

use super::{configuration, media, serving_from_initialization};

#[test]
fn poisoned_root_is_rejected_before_ordinary_interpretation_entry() {
    let parent = tempfile::tempdir().unwrap();
    let root = parent.path().join("poisoned-root");
    serving_from_initialization(&root).close();
    let artifact = root.join("families/records/roots/root-0000000000000001.manifest");
    let mut bytes = std::fs::read(&artifact).unwrap();
    bytes[28..36].copy_from_slice(&2_u64.to_le_bytes());
    reseal(&mut bytes);
    std::fs::write(&artifact, bytes).unwrap();

    let (format, _, access) = configuration();
    let outcome = open_record_store!(media(&root), |durability| {
        PhysicalRecordOpen::new(format, access, durability)
    })
    .into_raw();
    let TransitionOutcome::Denied(denial) = outcome else {
        panic!("a checksum-valid wrong-generation root must be denied before interpretation")
    };
    assert_eq!(denial.reason(), RecordBootstrapDenial::CurrentRootDamaged);
    let runtime = denial.into_runtime();
    let counters = runtime.root_protocol_counters();
    assert_eq!(
        counters.root_entries(PhysicalRootProtocolRoute::OrdinaryOpen),
        0,
    );
    assert_eq!(
        counters.selector_entries(PhysicalRootProtocolRoute::OrdinaryOpen),
        0,
    );
    assert_eq!(
        counters.publications(PhysicalRootProtocolRoute::OrdinaryOpen),
        0,
    );
    runtime.close();
}

#[test]
fn current_version_reopens_and_every_unimplemented_version_fails_typed() {
    let parent = tempfile::tempdir().unwrap();
    let (format, _, access) = configuration();
    let current = parent.path().join("current");
    serving_from_initialization(&current).close();
    super::success(open_record_store!(media(&current), |durability| {
        PhysicalRecordOpen::new(format, access, durability)
    }))
    .close();

    for (name, relative_path, read_paths) in [
        (
            "catalog",
            "families/records/bootstrap.catalog",
            &["families/records/bootstrap.catalog"][..],
        ),
        (
            "root",
            "families/records/roots/root-0000000000000001.manifest",
            &[
                "families/records/bootstrap.catalog",
                "families/records/roots/root-0000000000000001.manifest",
            ][..],
        ),
        (
            "free-space",
            "families/records/free-space/free-space-0000000000000001.manifest",
            &[
                "families/records/bootstrap.catalog",
                "families/records/roots/root-0000000000000001.manifest",
                "families/records/free-space/free-space-0000000000000001.manifest",
            ][..],
        ),
    ] {
        let root = parent.path().join(name);
        serving_from_initialization(&root).close();
        let expected_read_bytes = super::durable_frame_oracle::artifact_bytes(&root, read_paths);
        let artifact = root.join(relative_path);
        let mut bytes = std::fs::read(&artifact).unwrap();
        bytes[10..12].copy_from_slice(&2_u16.to_le_bytes());
        reseal(&mut bytes);
        std::fs::write(&artifact, bytes).unwrap();

        let media = media(&root);
        let before = media.media_counters();
        let outcome = open_record_store!(media, |durability| PhysicalRecordOpen::new(
            format, access, durability
        ))
        .into_raw();
        let TransitionOutcome::Denied(denial) = outcome else {
            panic!("{name} with an unimplemented version cannot open")
        };
        assert!(matches!(
            denial.reason(),
            RecordBootstrapDenial::UnsupportedPhysicalRecordFormat(unsupported)
                if unsupported.reason() == PhysicalRecordFormatDenial::UnsupportedVersion(2)
        ));
        let media = denial.into_runtime();
        let after = media.media_counters();
        assert_eq!(
            after.completed_bytes_for(MediaOperationRole::PositionedRead)
                - before.completed_bytes_for(MediaOperationRole::PositionedRead),
            expected_read_bytes,
            "{name}",
        );
        media.close();
    }
}

#[test]
fn unsupported_catalog_format_dimensions_localize_before_root_traversal() {
    let parent = tempfile::tempdir().unwrap();
    let (format, _, access) = configuration();
    for (name, offset, encoded, expected) in [
        (
            "old-version",
            10,
            &[0, 0][..],
            PhysicalRecordFormatDenial::UnsupportedVersion(0),
        ),
        (
            "page-size",
            12,
            &[1, 0, 0, 0][..],
            PhysicalRecordFormatDenial::UnsupportedPageBytes(1),
        ),
        (
            "byte-order",
            16,
            &[2][..],
            PhysicalRecordFormatDenial::UnsupportedByteOrder(2),
        ),
        (
            "root-protocol",
            17,
            &[2][..],
            PhysicalRecordFormatDenial::UnsupportedRootProtocol(2),
        ),
        (
            "integrity",
            18,
            &[2][..],
            PhysicalRecordFormatDenial::UnsupportedIntegrity(2),
        ),
        (
            "record-identity-width",
            19,
            &[25][..],
            PhysicalRecordFormatDenial::UnsupportedRecordIdentityBytes(25),
        ),
    ] {
        let root = parent.path().join(name);
        serving_from_initialization(&root).close();
        let catalog = root.join("families/records/bootstrap.catalog");
        let expected_read_bytes = super::durable_frame_oracle::artifact_bytes(
            &root,
            &["families/records/bootstrap.catalog"],
        );
        let mut bytes = std::fs::read(&catalog).unwrap();
        bytes[offset..offset + encoded.len()].copy_from_slice(encoded);
        reseal(&mut bytes);
        std::fs::write(&catalog, bytes).unwrap();
        let media = media(&root);
        let before = media.media_counters();
        let outcome = open_record_store!(media, |durability| PhysicalRecordOpen::new(
            format, access, durability
        ))
        .into_raw();
        let TransitionOutcome::Denied(denial) = outcome else {
            panic!("{name} must be denied")
        };
        assert!(matches!(
            denial.reason(),
            RecordBootstrapDenial::UnsupportedPhysicalRecordFormat(unsupported)
                if unsupported.reason() == expected
        ));
        let media = denial.into_runtime();
        assert_eq!(
            media
                .media_counters()
                .completed_bytes_for(MediaOperationRole::PositionedRead)
                - before.completed_bytes_for(MediaOperationRole::PositionedRead),
            expected_read_bytes,
            "{name}",
        );
        media.close();
    }
}

#[test]
fn selected_wrong_generation_root_is_damaged_before_foreign_store_rebind() {
    let parent = tempfile::tempdir().unwrap();
    let root = parent.path().join("stale");
    serving_from_initialization(&root).close();
    let roots = root.join("families/records/roots");
    std::fs::copy(
        roots.join("root-0000000000000001.manifest"),
        roots.join("root-0000000000000002.manifest"),
    )
    .unwrap();
    let catalog = root.join("families/records/bootstrap.catalog");
    let mut bytes = std::fs::read(&catalog).unwrap();
    bytes[28..36].copy_from_slice(&2_u64.to_le_bytes());
    super::durable_frame_oracle::payload_mut(&mut bytes)[16..24]
        .copy_from_slice(&2_u64.to_le_bytes());
    reseal(&mut bytes);
    std::fs::write(&catalog, bytes).unwrap();
    let (format, _, access) = configuration();
    let outcome = open_record_store!(media(&root), |durability| PhysicalRecordOpen::new(
        format, access, durability
    ))
    .into_raw();
    let TransitionOutcome::Denied(denial) = outcome else {
        panic!("selected wrong-generation root must be denied before owner interpretation")
    };
    assert_eq!(denial.reason(), RecordBootstrapDenial::CurrentRootDamaged,);
    let denied_runtime = denial.into_runtime();
    assert_eq!(
        denied_runtime
            .root_protocol_counters()
            .root_entries(PhysicalRootProtocolRoute::OrdinaryOpen),
        0,
    );
    denied_runtime.close();

    let foreign_root = parent.path().join("foreign");
    serving_from_initialization(&foreign_root).close();
    let catalog = foreign_root.join("families/records/bootstrap.catalog");
    let mut bytes = std::fs::read(&catalog).unwrap();
    super::durable_frame_oracle::payload_mut(&mut bytes)[..16].copy_from_slice(&[0x77; 16]);
    reseal(&mut bytes);
    std::fs::write(&catalog, bytes).unwrap();
    let outcome = open_record_store!(media(&foreign_root), |durability| PhysicalRecordOpen::new(
        format, access, durability
    ))
    .into_raw();
    let TransitionOutcome::RebindRequired(rebind) = outcome else {
        panic!("foreign persisted Store identity must require rebind")
    };
    assert_eq!(
        rebind.reason(),
        RecordServingRebindReason::StoreIdentityMismatch
    );
    rebind.into_runtime().close();
}

#[test]
fn caller_format_narrows_acceptance_without_authorizing_migration() {
    let parent = tempfile::tempdir().unwrap();
    let root = parent.path().join("store");
    serving_from_initialization(&root).close();
    let expected = AdmittedPhysicalRecordFormat::admit(
        PhysicalRecordFormatDeclaration::builder()
            .page_size(PhysicalPageSizeClass::KiB64)
            .admit()
            .unwrap(),
    );
    let access = PhysicalRecordAccessPolicy::builder()
        .admit(expected)
        .unwrap();
    let outcome = open_record_store!(media(&root), |durability| PhysicalRecordOpen::new(
        expected, access, durability
    ))
    .into_raw();
    let TransitionOutcome::Denied(denial) = outcome else {
        panic!("format drift cannot migrate on open")
    };
    assert!(matches!(
        denial.reason(),
        RecordBootstrapDenial::PhysicalRecordFormatMismatch(_)
    ));
    denial.into_runtime().close();
}

fn reseal(bytes: &mut [u8]) {
    super::durable_frame_oracle::reseal(bytes);
}
