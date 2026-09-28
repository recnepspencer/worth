use super::*;
use std::io::{Read, Seek, SeekFrom, Write};
use worth_store_physical_format::integrity_declarations::PhysicalIntegrityArtifactFamily;
use worth_store_physical_format::{
    DurableExtentManifest, ExtentArenaFrameLayout, EXTENT_ARENA_MANIFEST_FRAME_BYTES,
};
use worth_store_recovery_runtime::PhysicalRecoveryIntegrityObservationOutcome;

#[path = "../c10_extent_rewrite_crash/arena_route.rs"]
mod arena_route;

#[test]
fn extent_recovery_planning_admits_manifest_and_chunks_before_redo() {
    let root = prepare_extent_recovery_root("c9-page-extent-recovery");
    let planned = selected_ordinary_recovery(root.path()).plan().unwrap();
    assert!(planned.redo_plan().resolved_decisions().any(|decision| {
        matches!(
            decision.target().identity(),
            PhysicalRedoTargetIdentity::ExtentChunk { .. }
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
                PhysicalIntegrityArtifactFamily::ExtentManifest
                    | PhysicalIntegrityArtifactFamily::ExtentChunk
            )
        })
        .collect();
    assert_eq!(observations.len(), 4);
    assert_eq!(
        observations[0].scope().artifact_family(),
        PhysicalIntegrityArtifactFamily::ExtentManifest
    );
    assert!(observations.iter().all(|observation| observation.outcome()
        == PhysicalRecoveryIntegrityObservationOutcome::Admitted));
}

#[test]
fn corrupt_extent_recovery_frame_stops_before_owner_projection() {
    let root = prepare_extent_recovery_root("c9-corrupt-extent-recovery");
    let routes = arena_route::selected_routes(root.path());
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
        file.seek(SeekFrom::Start(route.range.offset())).unwrap();
        let mut manifest_bytes = [0_u8; EXTENT_ARENA_MANIFEST_FRAME_BYTES];
        file.read_exact(&mut manifest_bytes).unwrap();
        let (manifest, format) = DurableExtentManifest::decode(&manifest_bytes).unwrap();
        let chunk_offset = ExtentArenaFrameLayout::new(format, manifest.alignment())
            .unwrap()
            .chunk_offset(1)
            .unwrap();
        let corruption = route.range.offset() + chunk_offset + 120;
        assert!(corruption < route.range.offset() + route.range.length());
        file.seek(SeekFrom::Start(corruption)).unwrap();
        let mut byte = [0_u8; 1];
        file.read_exact(&mut byte).unwrap();
        byte[0] ^= 1;
        file.seek(SeekFrom::Start(corruption)).unwrap();
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
        PhysicalIntegrityArtifactFamily::ExtentChunk
    );
    assert!(matches!(
        rejected.outcome(),
        PhysicalRecoveryIntegrityObservationOutcome::Rejected(_)
    ));
    assert_eq!(
        observations[observations.len() - 2]
            .scope()
            .artifact_family(),
        PhysicalIntegrityArtifactFamily::ExtentManifest
    );
    assert_eq!(blocked.recovery_effects(), 0);
}

fn prepare_extent_recovery_root(name: &str) -> worth_store_test_support::TemporaryDirectory {
    let world = PhysicalResidencyStoreWorld::initialize_for_recovery(name).unwrap();
    let retained_root = world.retained_root();
    let payload = vec![0x61; 40_000];
    canonical_physical_mutation_acknowledgment(&world, [0x51; 32], &payload);
    let request = PhysicalCheckpointRequest::fuzzy(
        PhysicalCheckpointIdempotencyKey::new([0x52; 32]),
        PhysicalCheckpointDeadline::after_milliseconds(5_000).unwrap(),
    );
    let TransitionOutcome::Success(handle) =
        world.serving().checkpoints().start(request).into_raw()
    else {
        panic!("extent checkpoint admission must succeed")
    };
    assert!(matches!(
        handle.wait(),
        PhysicalCheckpointOutcome::Completed(_)
    ));
    canonical_rooted_mutation_without_acknowledgment(&world, [0x53; 32], &payload);
    canonical_durable_wal_attempt_without_execution(&world, [0x54; 32], &payload);
    drop(world);
    retained_root
}
