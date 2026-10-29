#[allow(unused_imports)] use crate::extent_validation::validation_records::*;
use {worth_store_physical_format::encode_extent_chunk, worth_store_physical_format::DurableExtentRecordPlacement, worth_store_physical_format::ExtentArenaFrameLayout, worth_store_physical_format::ExtentArenaRange, worth_store_physical_format::PhysicalPageSizeClass};
use {worth_store_physical_integrity::validate_extent_chunk, worth_store_physical_integrity::ExtentChunkIntegrityValidation, worth_store_physical_integrity::PhysicalArtifactScope, worth_store_physical_integrity::PhysicalByteRange, worth_store_physical_integrity::PhysicalIntegrityValidationRecord, worth_store_physical_integrity::UntrustedPhysicalArtifact};

use {crate::support::extent_validation::chunk_payload_capacity, crate::support::extent_validation::chunk_scope, crate::support::extent_validation::extent_cell, crate::support::extent_validation::format, crate::support::extent_validation::record, crate::support::extent_validation::store, crate::support::extent_validation::validated_manifest, crate::support::extent_validation::ExtentFixture};

#[test]
fn intact_validation_records_bind_every_extent_identity_axis() {
    let baseline = crate::support::extent_validation::ExtentFixture::new();
    let (baseline_manifest, baseline_chunk) = records(baseline, 2);
    let other_format = format(worth_store_physical_format::PhysicalPageSizeClass::KiB32);
    let variants = [
        ExtentFixture {
            record: record(0x33, 8),
            ..baseline
        },
        ExtentFixture {
            extent: extent_cell(8, 5),
            ..baseline
        },
        ExtentFixture {
            extent: extent_cell(4, 6),
            ..baseline
        },
        ExtentFixture {
            logical_bytes: chunk_payload_capacity(baseline.format) + 7,
            ..baseline
        },
        ExtentFixture {
            format: other_format,
            logical_bytes: chunk_payload_capacity(other_format) + 5,
            ..baseline
        },
        ExtentFixture {
            store: store(8),
            ..baseline
        },
    ];

    for variant in variants {
        let (manifest, chunk) = records(variant, 2);
        assert_ne!(
            baseline_manifest.exact_scope_digest(),
            manifest.exact_scope_digest()
        );
        assert_ne!(
            baseline_chunk.exact_scope_digest(),
            chunk.exact_scope_digest()
        );
    }

    let (_, first_chunk) = records(baseline, 1);
    assert_ne!(
        baseline_chunk.exact_scope_digest(),
        first_chunk.exact_scope_digest()
    );
}

#[test]
fn identical_bytes_keep_byte_digest_but_bind_store_and_range() {
    let fixture = crate::support::extent_validation::ExtentFixture::new();
    let manifest_bytes = fixture.manifest_bytes();
    let chunk_bytes = fixture.tail_chunk_bytes();
    let baseline_manifest = validated_manifest(&manifest_bytes, fixture.manifest_scope());
    let baseline = chunk_record(&chunk_bytes, fixture.tail_chunk_scope(), &baseline_manifest);

    let other_store_fixture = ExtentFixture {
        store: store(8),
        ..fixture
    };
    let other_manifest = validated_manifest(&manifest_bytes, other_store_fixture.manifest_scope());
    let other_store = chunk_record(
        &chunk_bytes,
        other_store_fixture.tail_chunk_scope(),
        &other_manifest,
    );
    let arena = fixture.arena_range();
    let shifted_arena =
        worth_store_physical_format::ExtentArenaRange::new(arena.arena(), arena.offset() + 65_536, arena.length()).unwrap();
    let shifted_placement = worth_store_physical_format::DurableExtentRecordPlacement::legacy_unknown(
        fixture.record,
        fixture.extent,
        fixture.logical_bytes,
        shifted_arena,
    )
    .unwrap();
    let shifted_manifest_scope = worth_store_physical_integrity::PhysicalArtifactScope::extent_manifest(
        fixture.store,
        fixture.format,
        shifted_placement,
        worth_store_physical_integrity::PhysicalByteRange::new(shifted_arena.offset(), manifest_bytes.len() as u64).unwrap(),
    );
    let shifted_manifest = validated_manifest(&manifest_bytes, shifted_manifest_scope);
    let layout = worth_store_physical_format::ExtentArenaFrameLayout::new(
        fixture.format,
        u64::from(fixture.format.page_size().bytes()),
    )
    .unwrap();
    let shifted_scope = worth_store_physical_integrity::PhysicalArtifactScope::extent_chunk(
        fixture.store,
        fixture.format,
        fixture.chunk_coordinate(2),
        worth_store_physical_integrity::PhysicalByteRange::new(
            shifted_arena.offset() + layout.chunk_offset(2).unwrap(),
            chunk_bytes.len() as u64,
        )
        .unwrap(),
        shifted_arena,
    );
    let shifted = chunk_record(&chunk_bytes, shifted_scope, &shifted_manifest);

    for changed in [other_store, shifted] {
        assert_eq!(baseline.byte_range_digest(), changed.byte_range_digest());
        assert_ne!(baseline.exact_scope_digest(), changed.exact_scope_digest());
    }
}
