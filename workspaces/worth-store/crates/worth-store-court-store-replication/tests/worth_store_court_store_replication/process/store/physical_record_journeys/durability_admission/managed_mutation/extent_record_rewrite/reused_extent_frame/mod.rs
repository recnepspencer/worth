use {worth_store_testkit_store::physical_record_journeys::durability_admission::managed_mutation::extent_record_rewrite::EXTENT_PAYLOAD_BYTES, worth_store_testkit_store::physical_record_journeys::manifest_fixture::IndependentExtentRoute, worth_store::physical_runtime::PhysicalRetirementDenial, worth_store_testkit_store::physical_record_journeys::durability_admission::managed_mutation::extent_record_rewrite::extent_frame_fixture::arena_range_bytes, worth_store_testkit_store::physical_record_journeys::durability_admission::managed_mutation::completed, worth_store_testkit_store::physical_record_journeys::configuration, worth_store_testkit_store::physical_record_journeys::manifest_fixture::current_extent_route, worth_store_testkit_store::physical_record_journeys::durability_admission::managed_mutation::extent_record_rewrite::extent_frame_fixture::extent_payload, worth_store_testkit_store::physical_record_journeys::durability_admission::managed_mutation::prepare, worth_store_testkit_store::physical_record_journeys::durability_admission::managed_mutation::extent_record_rewrite::prepare_extent_rewrite, worth_store_testkit_store::physical_record_journeys::durability_admission::managed_mutation::extent_record_rewrite::read_from_reader, worth_store_testkit_store::physical_record_journeys::durability_admission::managed_mutation::extent_record_rewrite::read_record, worth_store_testkit_store::physical_record_journeys::serving_from_initialization};
use {worth_store_physical_format::DurableExtentRecordPlacement, worth_store_physical_format::ExtentArenaId, worth_store_physical_format::ExtentArenaRange, worth_store_physical_format::PhysicalExtentId, worth_store_physical_format::PhysicalGeneration, worth_store_physical_format::PhysicalGenerationAuthority};
use {worth_store_physical_integrity::validate_extent_arena_frame, worth_store_physical_integrity::ExtentArenaFrameExpectation, worth_store_physical_integrity::ExtentArenaFrameIntegrityValidation, worth_store_physical_integrity::PhysicalArtifactScope, worth_store_physical_integrity::PhysicalByteRange, worth_store_physical_integrity::UntrustedPhysicalArtifact};

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
        Err(worth_store::physical_runtime::PhysicalRetirementDenial::Protected)
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
    let arena = worth_store_physical_format::ExtentArenaRange::new(
        worth_store_physical_format::ExtentArenaId::new(new_route.arena).unwrap(),
        new_route.offset,
        new_route.length,
    )
    .unwrap();
    let scoped_placement = |record, route: IndependentExtentRoute| {
        let cell = worth_store_physical_format::PhysicalGenerationAuthority::for_canonical_physical_format()
            .record_extent_cell(worth_store_physical_format::PhysicalExtentId::from_raw(route.extent).unwrap())
            .with_extent_generation(worth_store_physical_format::PhysicalGeneration::from_raw(route.generation).unwrap());
        worth_store_physical_format::DurableExtentRecordPlacement::legacy_unknown(
            record,
            cell,
            EXTENT_PAYLOAD_BYTES as u64,
            arena,
        )
        .unwrap()
    };
    let new_scope = worth_store_physical_integrity::PhysicalArtifactScope::extent_manifest(
        store,
        format.declaration(),
        scoped_placement(new_identity, new_route),
        worth_store_physical_integrity::PhysicalByteRange::new(new_route.offset, 104).unwrap(),
    );
    assert!(matches!(
        validate_extent_arena_frame(
            worth_store_physical_integrity::UntrustedPhysicalArtifact::from_bounded_bytes(
                &frame[..104]
            ),
            worth_store_physical_integrity::ExtentArenaFrameExpectation::Manifest(new_scope),
        )
        .0,
        worth_store_physical_integrity::ExtentArenaFrameIntegrityValidation::Manifest(_)
    ));
    let old_scope = worth_store_physical_integrity::PhysicalArtifactScope::extent_manifest(
        store,
        format.declaration(),
        scoped_placement(old_identity, old_route),
        worth_store_physical_integrity::PhysicalByteRange::new(old_route.offset, 104).unwrap(),
    );
    assert!(
        matches!(
            validate_extent_arena_frame(
                worth_store_physical_integrity::UntrustedPhysicalArtifact::from_bounded_bytes(
                    &frame[..104]
                ),
                worth_store_physical_integrity::ExtentArenaFrameExpectation::Manifest(old_scope),
            )
            .0,
            worth_store_physical_integrity::ExtentArenaFrameIntegrityValidation::Rejected(_)
        ),
        "a forged old route must not accept the new extent frame as stale bytes"
    );
    serving.close();
}
