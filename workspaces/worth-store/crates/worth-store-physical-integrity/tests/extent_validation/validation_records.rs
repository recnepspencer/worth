use worth_store_physical_format::{encode_extent_chunk, PhysicalPageSizeClass};
use worth_store_physical_integrity::{
    validate_extent_chunk, ExtentChunkIntegrityValidation, PhysicalArtifactScope,
    PhysicalByteRange, PhysicalIntegrityValidationRecord, UntrustedPhysicalArtifact,
};

use super::support::{
    chunk_payload_capacity, chunk_scope, extent_cell, format, record, store, validated_manifest,
    ExtentFixture, CHUNK_OFFSET,
};

#[test]
fn intact_validation_records_bind_every_extent_identity_axis() {
    let baseline = ExtentFixture::new();
    let (baseline_manifest, baseline_chunk) = records(baseline, 2);
    let other_format = format(PhysicalPageSizeClass::KiB32);
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
    let fixture = ExtentFixture::new();
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
    let shifted_scope = PhysicalArtifactScope::extent_chunk(
        fixture.store,
        fixture.format,
        fixture.chunk_coordinate(2),
        PhysicalByteRange::new(CHUNK_OFFSET + 4096, chunk_bytes.len() as u64).unwrap(),
    );
    let shifted = chunk_record(&chunk_bytes, shifted_scope, &baseline_manifest);

    for changed in [other_store, shifted] {
        assert_eq!(baseline.byte_range_digest(), changed.byte_range_digest());
        assert_ne!(baseline.exact_scope_digest(), changed.exact_scope_digest());
    }
}

fn records(
    fixture: ExtentFixture,
    ordinal: u32,
) -> (
    PhysicalIntegrityValidationRecord,
    PhysicalIntegrityValidationRecord,
) {
    let manifest_bytes = fixture.manifest_bytes();
    let manifest = validated_manifest(&manifest_bytes, fixture.manifest_scope());
    let manifest_record =
        validated_manifest(&manifest_bytes, fixture.manifest_scope()).into_validation_record();
    let coordinate = fixture.chunk_coordinate(ordinal);
    let length = if ordinal == 1 {
        chunk_payload_capacity(fixture.format)
    } else {
        fixture.logical_bytes - chunk_payload_capacity(fixture.format)
    };
    let payload = vec![0xA5; usize::try_from(length).unwrap()];
    let chunk_bytes = encode_extent_chunk(fixture.format, coordinate, &payload).unwrap();
    let scope = chunk_scope(
        fixture.store,
        fixture.format,
        coordinate,
        chunk_bytes.len() as u64,
    );
    (
        manifest_record,
        chunk_record(&chunk_bytes, scope, &manifest),
    )
}

fn chunk_record(
    bytes: &[u8],
    scope: PhysicalArtifactScope,
    manifest: &worth_store_physical_integrity::IntegrityValidatedExtentManifest<'_>,
) -> PhysicalIntegrityValidationRecord {
    let (ExtentChunkIntegrityValidation::Intact(chunk), _) = validate_extent_chunk(
        UntrustedPhysicalArtifact::from_bounded_bytes(bytes),
        scope,
        manifest,
    ) else {
        panic!("clean extent chunk rejected")
    };
    chunk.into_validation_record()
}
