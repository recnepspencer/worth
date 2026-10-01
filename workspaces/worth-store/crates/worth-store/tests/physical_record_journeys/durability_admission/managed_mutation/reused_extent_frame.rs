use super::*;
use worth_store_physical_format::{
    DurableExtentRecordPlacement, ExtentArenaId, ExtentArenaRange, PhysicalExtentId,
    PhysicalGeneration, PhysicalGenerationAuthority,
};
use worth_store_physical_integrity::{
    validate_extent_arena_frame, ExtentArenaFrameExpectation, ExtentArenaFrameIntegrityValidation,
    PhysicalArtifactScope, PhysicalByteRange, UntrustedPhysicalArtifact,
};

#[test]
fn held_reader_prevents_reuse_and_reused_frame_rejects_forged_old_route() {
    let parent = tempfile::tempdir().unwrap();
    let root = parent.path().join("store");
    let serving = serving_from_initialization(&root);
    let (format, placement, _) = configuration();
    let store = serving.store_identity();
    let old_bytes = extent_payload();
    let old = completed(prepare(&serving, placement, [151; 32], &old_bytes).execute());
    let old_identity = old.persisted_records()[0];
    let old_record = old.into_acknowledgment().record_ids().next().unwrap();
    let old_route = current_extent_route(&root, old_identity);
    let held_reader = serving.records().unwrap();

    completed(prepare_extent_rewrite(&serving, placement, [152; 32], old_record).execute());
    assert_eq!(
        serving.retire_displaced_segment(),
        Err(PhysicalRetirementDenial::Protected)
    );
    let interim = completed(prepare(&serving, placement, [153; 32], &old_bytes).execute());
    let interim_route = current_extent_route(&root, interim.persisted_records()[0]);
    assert_ne!(
        (interim_route.arena, interim_route.offset),
        (old_route.arena, old_route.offset),
        "ordinary allocation cannot take a range routed by a held root"
    );
    assert_eq!(read_from_reader(&held_reader, old_record), old_bytes);

    drop(held_reader);
    serving.retire_displaced_segment().unwrap();
    let new_bytes = vec![201; EXTENT_PAYLOAD_BYTES];
    let new = completed(prepare(&serving, placement, [154; 32], &new_bytes).execute());
    let new_identity = new.persisted_records()[0];
    let new_record = new.into_acknowledgment().record_ids().next().unwrap();
    let new_route = current_extent_route(&root, new_identity);
    assert_eq!(
        (new_route.arena, new_route.offset, new_route.length),
        (old_route.arena, old_route.offset, old_route.length)
    );
    assert_ne!(new_route.extent, old_route.extent);
    assert_eq!(read_record(&serving, new_record), new_bytes);

    let frame = arena_range_bytes(&root, new_route);
    let arena = ExtentArenaRange::new(
        ExtentArenaId::new(new_route.arena).unwrap(),
        new_route.offset,
        new_route.length,
    )
    .unwrap();
    let scoped_placement = |record, route: IndependentExtentRoute| {
        let cell = PhysicalGenerationAuthority::for_canonical_physical_format()
            .record_extent_cell(PhysicalExtentId::from_raw(route.extent).unwrap())
            .with_extent_generation(PhysicalGeneration::from_raw(route.generation).unwrap());
        DurableExtentRecordPlacement::legacy_unknown(record, cell, EXTENT_PAYLOAD_BYTES as u64, arena).unwrap()
    };
    let new_scope = PhysicalArtifactScope::extent_manifest(
        store,
        format.declaration(),
        scoped_placement(new_identity, new_route),
        PhysicalByteRange::new(new_route.offset, 104).unwrap(),
    );
    assert!(matches!(
        validate_extent_arena_frame(
            UntrustedPhysicalArtifact::from_bounded_bytes(&frame[..104]),
            ExtentArenaFrameExpectation::Manifest(new_scope),
        )
        .0,
        ExtentArenaFrameIntegrityValidation::Manifest(_)
    ));
    let old_scope = PhysicalArtifactScope::extent_manifest(
        store,
        format.declaration(),
        scoped_placement(old_identity, old_route),
        PhysicalByteRange::new(old_route.offset, 104).unwrap(),
    );
    assert!(
        matches!(
            validate_extent_arena_frame(
                UntrustedPhysicalArtifact::from_bounded_bytes(&frame[..104]),
                ExtentArenaFrameExpectation::Manifest(old_scope),
            )
            .0,
            ExtentArenaFrameIntegrityValidation::Rejected(_)
        ),
        "a forged old route must not accept the new extent frame as stale bytes"
    );
    serving.close();
}
