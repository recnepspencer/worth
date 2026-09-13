use super::{
    BackupBundleArtifactCoverage, BackupBundleArtifactFamily, BackupBundleArtifactFormat,
    BackupBundleArtifactManifestRow, BackupBundleManifest, BackupBundleManifestDeclaration,
    BackupBundleManifestIdentity, BackupBundleRecoveryCoordinates,
};
use crate::{
    BackupBundleFormatAuthority, BackupBundleFormatDenial, BackupBundleManifestReadLimits,
    BackupBundlePhysicalOwner, PhysicalGeneration, PhysicalGenerationOwner, PhysicalPageId,
    PhysicalRecordSlot, PhysicalRootReference, PhysicalSegmentId,
};
use sha2::{Digest, Sha256};

#[test]
fn output_names_are_unique_across_nonadjacent_families() {
    let row = |family, identity: &str, output: &str| {
        BackupBundleArtifactManifestRow::new(
            family,
            format(family),
            identity,
            output,
            1,
            1,
            [1; 32],
            match family {
                BackupBundleArtifactFamily::RootManifest => {
                    BackupBundleArtifactCoverage::RootManifest { root_generation: 1 }
                }
                BackupBundleArtifactFamily::CheckpointManifest => {
                    BackupBundleArtifactCoverage::CheckpointManifest {
                        checkpoint_identity: "checkpoint".into(),
                        manifest_generation: 1,
                        durable_checkpoint_lsn: 1,
                        authority_fingerprint: [7; 32],
                        frontier_digest: [8; 32],
                    }
                }
                _ => BackupBundleArtifactCoverage::PhysicalReachability,
            },
            owner(family),
        )
        .expect("row")
    };
    let rows = vec![
        row(BackupBundleArtifactFamily::RootManifest, "root", "same.bin"),
        row(
            BackupBundleArtifactFamily::CheckpointManifest,
            "checkpoint",
            "middle.bin",
        ),
        row(BackupBundleArtifactFamily::Page, "page", "same.bin"),
    ];
    assert!(BackupBundleManifest::canonical(manifest_declaration(7, rows)).is_none());
}

#[test]
fn decoded_manifest_cannot_bypass_row_constructor_invariants() {
    let row = BackupBundleArtifactManifestRow::new(
        BackupBundleArtifactFamily::Page,
        BackupBundleArtifactFormat::PhysicalDataPageV1,
        "page-a",
        "page-a.bin",
        1,
        4,
        [7; 32],
        BackupBundleArtifactCoverage::PhysicalReachability,
        owner(BackupBundleArtifactFamily::Page),
    )
    .expect("canonical row");
    let manifest = BackupBundleManifest::canonical(manifest_declaration(9, vec![row]))
        .expect("canonical manifest");
    let authority = BackupBundleFormatAuthority::canonical();
    let mut encoded = authority
        .encode_manifest(&manifest)
        .expect("binary manifest");
    let output_offset = encoded
        .windows(b"page-a.bin".len())
        .position(|window| window == b"page-a.bin")
        .expect("encoded output name");
    encoded[output_offset] = b'/';
    assert!(matches!(
        authority.decode_manifest(&encoded),
        Err(BackupBundleFormatDenial::InvalidManifest)
    ));
}

#[test]
fn hostile_binary_manifests_fail_closed_without_length_driven_allocation() {
    let authority = BackupBundleFormatAuthority::canonical();
    let encoded = authority
        .encode_manifest(&page_manifest(1))
        .expect("canonical binary manifest");

    for boundary in 0..encoded.len() {
        assert!(authority.decode_manifest(&encoded[..boundary]).is_err());
    }

    let mut trailing = encoded.clone();
    trailing.push(0);
    assert!(authority.decode_manifest(&trailing).is_err());

    let mut unknown_version = encoded.clone();
    unknown_version[7] = b'2';
    assert!(authority.decode_manifest(&unknown_version).is_err());

    let mut impossible_lineage_length = encoded.clone();
    impossible_lineage_length[40..44].copy_from_slice(&u32::MAX.to_le_bytes());
    assert!(authority
        .decode_manifest(&impossible_lineage_length)
        .is_err());

    let mut invalid_utf8 = encoded;
    invalid_utf8[44] = u8::MAX;
    assert!(authority.decode_manifest(&invalid_utf8).is_err());
}

#[test]
fn manifest_decode_denies_encoded_and_artifact_limits_before_admission() {
    let authority = BackupBundleFormatAuthority::canonical();
    let manifest = page_manifest(2);
    let encoded = authority
        .encode_manifest(&manifest)
        .expect("encode manifest");

    let byte_denial = authority
        .decode_manifest_with_limits(
            &encoded,
            BackupBundleManifestReadLimits::new(encoded.len() as u64 - 1, 2, 17, 4096)
                .expect("limits"),
        )
        .expect_err("encoded byte limit");
    assert!(matches!(
        byte_denial,
        BackupBundleFormatDenial::ManifestReadLimitExceeded { .. }
    ));

    let artifact_denial = authority
        .decode_manifest_with_limits(
            &encoded,
            BackupBundleManifestReadLimits::new(encoded.len() as u64, 1, 17, 4096).expect("limits"),
        )
        .expect_err("artifact count limit");
    assert!(matches!(
        artifact_denial,
        BackupBundleFormatDenial::ManifestArtifactLimitExceeded {
            artifacts: 2,
            maximum_artifacts: 1
        }
    ));
}

#[test]
fn manifest_decode_denies_owned_allocation_before_length_driven_materialization() {
    let authority = BackupBundleFormatAuthority::canonical();
    let encoded = authority
        .encode_manifest(&page_manifest(2))
        .expect("encode manifest");
    let denial = authority
        .decode_manifest_with_limits(
            &encoded,
            BackupBundleManifestReadLimits::new(encoded.len() as u64, 2, 17, 17).expect("limits"),
        )
        .expect_err("manifest structures exceed the owned allocation budget");
    assert!(matches!(
        denial,
        BackupBundleFormatDenial::ManifestOwnedAllocationLimitExceeded { .. }
    ));
}

#[test]
fn materialized_admission_streams_and_reports_manifest_cost_exactly() {
    let directory = tempfile::tempdir().expect("temp directory");
    let authority = BackupBundleFormatAuthority::canonical();
    let manifest = page_manifest(2);
    let encoded = authority
        .encode_manifest(&manifest)
        .expect("encode manifest");
    std::fs::write(directory.path().join("backup.manifest"), &encoded).expect("write manifest");
    let materialized = authority
        .admit_materialized_with_limits(
            directory.path(),
            BackupBundleManifestReadLimits::new(encoded.len() as u64, 2, 17, 4096).expect("limits"),
        )
        .expect("bounded admission");
    let observation = materialized.manifest_read_observation();

    assert_eq!(observation.encoded_bytes(), encoded.len() as u64);
    assert_eq!(observation.read_buffer_bytes(), 17);
    let expected_peak_owned_allocation = materialized
        .manifest()
        .owned_allocation_bytes()
        .expect("manifest allocation accounting")
        + (materialized.manifest().artifacts().len() * std::mem::size_of::<&str>()) as u64;
    assert_eq!(
        observation.owned_allocation_bytes(),
        expected_peak_owned_allocation
    );
    let expected_digest: [u8; 32] = Sha256::digest(encoded).into();
    assert_eq!(materialized.manifest_digest(), expected_digest);
}

#[test]
fn manifest_limits_reject_wasteful_or_unbounded_read_buffers() {
    assert!(BackupBundleManifestReadLimits::new(1024, 1, 1025, 2048).is_none());
    assert!(BackupBundleManifestReadLimits::new(
        16 * 1024 * 1024,
        1,
        8 * 1024 * 1024 + 1,
        16 * 1024 * 1024,
    )
    .is_none());
}

fn page_manifest(rows: usize) -> BackupBundleManifest {
    let artifacts = (0..rows)
        .map(|index| {
            BackupBundleArtifactManifestRow::new(
                BackupBundleArtifactFamily::Page,
                BackupBundleArtifactFormat::PhysicalDataPageV1,
                format!("page-{index}"),
                format!("page-{index}.bin"),
                1,
                4,
                [index as u8; 32],
                BackupBundleArtifactCoverage::PhysicalReachability,
                owner(BackupBundleArtifactFamily::Page),
            )
            .expect("page row")
        })
        .collect();
    BackupBundleManifest::canonical(manifest_declaration(9, artifacts)).expect("canonical manifest")
}

fn manifest_declaration(
    security_scope_fingerprint: u64,
    artifacts: Vec<BackupBundleArtifactManifestRow>,
) -> BackupBundleManifestDeclaration {
    BackupBundleManifestDeclaration::new(
        BackupBundleManifestIdentity {
            cut_identity: [1; 32],
            store_lineage: "lineage".to_owned(),
            root_generation: 1,
            manifest_generation: 1,
        },
        BackupBundleRecoveryCoordinates {
            checkpoint_identity: "checkpoint".to_owned(),
            durable_checkpoint_lsn: 1,
            wal_half_open_interval: (1, 1),
            acknowledged_frontier: 1,
        },
        security_scope_fingerprint,
        artifacts,
    )
}

fn owner(family: BackupBundleArtifactFamily) -> BackupBundlePhysicalOwner {
    let generation = PhysicalGeneration::from_raw(1).expect("generation");
    let owner = match family {
        BackupBundleArtifactFamily::RootManifest | BackupBundleArtifactFamily::SecondaryRoot => {
            PhysicalGenerationOwner::for_root_publication(
                PhysicalRootReference::from_raw(1).expect("root"),
                generation,
            )
        }
        BackupBundleArtifactFamily::WalSegment => PhysicalGenerationOwner::for_segment(
            PhysicalSegmentId::from_raw(1).expect("segment"),
            generation,
        ),
        BackupBundleArtifactFamily::Extent => PhysicalGenerationOwner::for_record_extent(
            crate::PhysicalExtentId::from_raw(1).expect("extent"),
            generation,
        ),
        BackupBundleArtifactFamily::BlobChunk => PhysicalGenerationOwner::for_extent(
            PhysicalSegmentId::from_raw(1).expect("segment"),
            crate::PhysicalExtentId::from_raw(1).expect("extent"),
            generation,
        ),
        BackupBundleArtifactFamily::Page => {
            crate::PhysicalGenerationAuthority::for_canonical_physical_format()
                .page_cell(
                    PhysicalSegmentId::from_raw(1).expect("segment"),
                    PhysicalPageId::from_raw(1).expect("page"),
                )
                .with_page_generation(generation)
                .owner()
        }
        BackupBundleArtifactFamily::CheckpointManifest | BackupBundleArtifactFamily::Index => {
            PhysicalGenerationOwner::for_slot(
                PhysicalSegmentId::from_raw(1).expect("segment"),
                PhysicalPageId::from_raw(1).expect("page"),
                PhysicalRecordSlot::from_raw(1).expect("slot"),
                generation,
            )
        }
    };
    BackupBundlePhysicalOwner::from_generation_owner(owner)
}

#[test]
fn artifact_rows_bind_family_and_generation_to_the_physical_owner() {
    let page_owner = owner(BackupBundleArtifactFamily::Page);
    assert!(BackupBundleArtifactManifestRow::new(
        BackupBundleArtifactFamily::Index,
        BackupBundleArtifactFormat::LayoutBTreeLeafV1,
        "index",
        "index.bin",
        1,
        1,
        [1; 32],
        BackupBundleArtifactCoverage::PhysicalReachability,
        page_owner,
    )
    .is_none());
    assert!(BackupBundleArtifactManifestRow::new(
        BackupBundleArtifactFamily::Page,
        BackupBundleArtifactFormat::PhysicalDataPageV1,
        "page",
        "page.bin",
        2,
        1,
        [1; 32],
        BackupBundleArtifactCoverage::PhysicalReachability,
        page_owner,
    )
    .is_none());
}

const fn format(family: BackupBundleArtifactFamily) -> BackupBundleArtifactFormat {
    match family {
        BackupBundleArtifactFamily::RootManifest => {
            BackupBundleArtifactFormat::PhysicalRootManifestV1
        }
        BackupBundleArtifactFamily::CheckpointManifest => {
            BackupBundleArtifactFormat::RecoveryCheckpointManifestV1
        }
        BackupBundleArtifactFamily::WalSegment => BackupBundleArtifactFormat::WalSegmentV1,
        BackupBundleArtifactFamily::Page => BackupBundleArtifactFormat::PhysicalDataPageV1,
        BackupBundleArtifactFamily::Extent => BackupBundleArtifactFormat::PhysicalExtentRecordV1,
        BackupBundleArtifactFamily::Index => BackupBundleArtifactFormat::LayoutBTreeLeafV1,
        BackupBundleArtifactFamily::BlobChunk => BackupBundleArtifactFormat::BlobChunkV1,
        BackupBundleArtifactFamily::SecondaryRoot => {
            BackupBundleArtifactFormat::PhysicalSecondaryRootManifestV1
        }
    }
}
