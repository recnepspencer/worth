#[allow(unused_imports)] use crate::phase_four_planning::page_extent_recovery::*;
use {worth_store::physical_runtime::PhysicalCheckpointDeadline, worth_store::physical_runtime::PhysicalCheckpointIdempotencyKey, worth_store::physical_runtime::PhysicalCheckpointOutcome, worth_store::physical_runtime::PhysicalCheckpointRequest, worth_store_recovery_physics::PhysicalRedoTargetIdentity, worth_store_test_support::harness::physical_residency::PhysicalResidencyStoreWorld, worth_proof::TransitionOutcome, worth_store_test_support::harness::physical_residency::canonical_durable_wal_attempt_without_execution, worth_store_test_support::harness::physical_residency::canonical_physical_mutation_acknowledgment, worth_store_test_support::harness::physical_residency::canonical_rooted_mutation_without_acknowledgment, worth_store_testkit_recovery_runtime::phase_three_support::expect_blocked, crate::phase_four_planning::selected_ordinary_recovery};
use {std::io::Read, std::io::Seek, std::io::SeekFrom, std::io::Write};
use {worth_store_physical_format::integrity_declarations::PhysicalIntegrityArtifactFamily};
use {worth_store_physical_format::DurableExtentManifest, worth_store_physical_format::ExtentArenaFrameLayout, worth_store_physical_format::EXTENT_ARENA_MANIFEST_FRAME_BYTES};
use {worth_store_recovery_runtime::PhysicalRecoveryIntegrityObservationOutcome};



#[test]
fn extent_recovery_planning_admits_manifest_and_chunks_before_redo() {
    let root = prepare_extent_recovery_root("c9-page-extent-recovery");
    let planned = selected_ordinary_recovery(root.path()).plan().unwrap();
    assert!(planned.redo_plan().resolved_decisions().any(|decision| {
        matches!(
            decision.target().identity(),
            worth_store_recovery_physics::PhysicalRedoTargetIdentity::ExtentChunk { .. }
        )
    }));
    let counters = planned.planning_counters();
    assert_eq!(counters.page_extent_integrity_attempts(), 4);
    assert_eq!(counters.page_extent_integrity_admissions(), 4);
    assert_eq!(counters.page_extent_integrity_rejections(), 0);
    assert_eq!(counters.page_extent_owner_projections(), 4);
    assert_eq!(counters.page_extent_owner_decoders(), 3);
    let observations: Vec<_> = planned
        .integrity_observations()
        .iter()
        .filter(|observation| {
            matches!(
                observation.scope().artifact_family(),
                worth_store_physical_format::integrity_declarations::PhysicalIntegrityArtifactFamily::ExtentManifest
                    | worth_store_physical_format::integrity_declarations::PhysicalIntegrityArtifactFamily::ExtentChunk
            )
        })
        .collect();
    assert_eq!(observations.len(), 4);
    assert_eq!(
        observations[0].scope().artifact_family(),
        worth_store_physical_format::integrity_declarations::PhysicalIntegrityArtifactFamily::ExtentManifest
    );
    assert!(observations.iter().all(|observation| observation.outcome()
        == worth_store_recovery_runtime::PhysicalRecoveryIntegrityObservationOutcome::Admitted));
}

#[test]
fn corrupt_extent_recovery_frame_stops_before_owner_projection() {
    let root = prepare_extent_recovery_root("c9-corrupt-extent-recovery");
    let routes = worth_store_testkit_recovery_runtime::arena_route::selected_routes(root.path());
    assert_eq!(
        routes.len(),
        2,
        "two rooted extent publications are selected"
    );
    for route in routes {
        let path = root.path().join(format!(
            "families/records/arenas/arena-{:016x}.data",
            route.range.arena().get()
        ));
        let mut file = std::fs::OpenOptions::new()
            .read(true)
            .write(true)
            .open(path)
            .unwrap();
        file.seek(std::io::SeekFrom::Start(route.range.offset())).unwrap();
        let mut manifest_bytes = [0_u8; EXTENT_ARENA_MANIFEST_FRAME_BYTES];
        file.read_exact(&mut manifest_bytes).unwrap();
        let (manifest, format) = worth_store_physical_format::DurableExtentManifest::decode(&manifest_bytes).unwrap();
        // The literal root-tree route and the addressed manifest bind one extent.
        assert_eq!(manifest.record(), route.record);
        assert_eq!(manifest.extent().get(), route.extent);
        assert_eq!(manifest.extent_cell().generation().get(), route.generation);
        assert_eq!(manifest.logical_bytes(), route.payload_bytes);
        let chunk_offset = worth_store_physical_format::ExtentArenaFrameLayout::new(format, manifest.alignment())
            .unwrap()
            .chunk_offset(1)
            .unwrap();
        let corruption = route.range.offset() + chunk_offset + 120;
        assert!(corruption < route.range.offset() + route.range.length());
        file.seek(std::io::SeekFrom::Start(corruption)).unwrap();
        let mut byte = [0_u8; 1];
        file.read_exact(&mut byte).unwrap();
        byte[0] ^= 1;
        file.seek(std::io::SeekFrom::Start(corruption)).unwrap();
        file.write_all(&byte).unwrap();
        file.sync_all().unwrap();
    }
    let blocked = match selected_ordinary_recovery(root.path()).plan() {
        Ok(_) => panic!("a corrupt clean extent frame cannot form a recovery plan"),
        Err(outcome) => expect_blocked(outcome),
    };
    let counters = blocked.evidence().planning_counters.unwrap();
    assert_eq!(counters.page_extent_integrity_attempts(), 2);
    assert_eq!(counters.page_extent_integrity_admissions(), 1);
    assert_eq!(counters.page_extent_integrity_rejections(), 1);
    assert_eq!(counters.page_extent_owner_projections(), 1);
    assert_eq!(counters.page_extent_owner_decoders(), 0);
    let observations = blocked.evidence().integrity_observations();
    let rejected = observations
        .last()
        .expect("the failed extent attempt survives the block");
    assert_eq!(
        rejected.scope().artifact_family(),
        worth_store_physical_format::integrity_declarations::PhysicalIntegrityArtifactFamily::ExtentChunk
    );
    assert!(matches!(
        rejected.outcome(),
        worth_store_recovery_runtime::PhysicalRecoveryIntegrityObservationOutcome::Rejected(_)
    ));
    assert_eq!(
        observations[observations.len() - 2]
            .scope()
            .artifact_family(),
        worth_store_physical_format::integrity_declarations::PhysicalIntegrityArtifactFamily::ExtentManifest
    );
    assert_eq!(blocked.recovery_effects(), 0);
}
