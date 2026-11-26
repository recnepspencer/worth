use super::*;
use worth_store_physical_format::{
    ArenaExtentFrameIdentity, ExtentAllocationGeneration, ExtentArenaId, ExtentArenaRange,
};

const ARENA: u64 = 9;

fn range(offset: u64, length: u64) -> ExtentArenaRange {
    ExtentArenaRange::new(ExtentArenaId::new(ARENA).unwrap(), offset, length).unwrap()
}

fn generation(root: u64, seq: u64) -> ExtentAllocationGeneration {
    ExtentAllocationGeneration::at_durable_barrier(root, seq).unwrap()
}

fn arena_key(identity: StableStoreIdentity, offset: u64) -> PhysicalFrameKey {
    let artifact = RecordArtifactFile::ExtentArena { arena: ARENA };
    PhysicalFrameKey::new(
        identity,
        RecordFrameCoordinate::new(artifact, offset, 32).unwrap(),
    )
}

#[test]
fn paused_writeback_blocks_exact_split_and_coalesced_reuse_until_drained() {
    let identity = store(114);
    let (pool, _, writeback_clean) =
        PhysicalResidencyPoolOwner::open(identity, limits(256, 4, 2, candidate_batch_bytes(2), 8))
            .unwrap()
            .into_parts();
    let allocation = candidate_allocation(&pool, 2);
    let paused_key = arena_key(identity, 64);
    let neighbour_key = arena_key(identity, 0);
    let paused_dirty = pool
        .materialize_dirty_candidate(&allocation, paused_key, |bytes| bytes.fill(4))
        .unwrap();
    let neighbour_dirty = pool
        .materialize_dirty_candidate(&allocation, neighbour_key, |bytes| bytes.fill(5))
        .unwrap();
    drop(allocation);
    let paused = writeback_claim(&pool, &[paused_key]);
    let stale = paused.arena_identity(0).unwrap();
    assert_eq!(
        stale,
        ArenaExtentFrameIdentity::new(range(64, 32), ExtentAllocationGeneration::OPENED)
    );

    let retired = ArenaReuseBarrier::CompletedRetirement;
    let busy = Err(WritebackFenceDrainDenial::WritebackInFlight { overlapping: 1 });
    for (offset, length) in [(64, 32), (80, 16), (0, 256)] {
        assert_eq!(
            pool.drain_writeback_fence(range(offset, length), retired, generation(5, 10)),
            busy
        );
    }
    assert_eq!(
        pool.drain_writeback_fence(
            range(64, 32),
            ArenaReuseBarrier::RetirementIncomplete,
            generation(5, 10)
        ),
        Err(WritebackFenceDrainDenial::BarrierIncomplete(
            ArenaReuseBarrier::RetirementIncomplete
        ))
    );
    // A late writeback inside the coalesced fence is refused before any I/O.
    let late = pool
        .begin_foreground_write_operation(nonzero_bytes(32))
        .unwrap();
    assert_eq!(
        pool.claim_writeback(late, &[neighbour_key]).unwrap_err(),
        PhysicalResidencyDenial::WriteBackRangeFenced
    );
    neighbour_dirty.discard_candidate().unwrap();

    paused.complete_writeback(&writeback_clean).unwrap();
    drop(paused_dirty);
    let entries = pool.counters().frame_entries();
    let admitted = pool
        .drain_writeback_fence(range(0, 256), retired, generation(5, 10))
        .unwrap();
    assert_eq!(admitted.generation(), generation(5, 10));
    assert_eq!(pool.counters().frame_entries(), entries - 1);

    let cancelled = ArenaReuseBarrier::CheckpointCoveredCancel;
    assert_eq!(
        pool.drain_writeback_fence(range(64, 32), cancelled, generation(5, 10)),
        Err(WritebackFenceDrainDenial::GenerationNotAdvanced {
            proposed: generation(5, 10),
            active: generation(5, 10),
        })
    );
    let reused = pool
        .drain_writeback_fence(range(64, 32), cancelled, generation(5, 11))
        .unwrap();
    assert!(reused.generation() > stale.generation());

    let allocation = candidate_allocation(&pool, 1);
    let fresh = pool
        .materialize_dirty_candidate(&allocation, paused_key, |bytes| bytes.fill(6))
        .unwrap();
    assert_eq!(pool.arena_frame_identity(paused_key), Some(reused));
    assert_eq!(
        pool.require_arena_frame_identity(paused_key, stale),
        Err(PhysicalResidencyDenial::StaleArenaGeneration)
    );
    pool.require_arena_frame_identity(paused_key, reused)
        .unwrap();
    drop(allocation);
    let claim = writeback_claim(&pool, &[paused_key]);
    assert_eq!(claim.arena_identity(0), Some(reused));
    drop(claim);
    fresh.discard_candidate().unwrap();
}

#[test]
fn drain_refuses_pinned_and_loading_frames_and_evicts_only_overlap() {
    let identity = store(115);
    let pool = PhysicalResidencyPool::open(identity, limits(256, 4, 2, 256, 8)).unwrap();
    let allocation = allocation(&pool, READ_SCOPE);
    for (offset, value) in [(0, 1), (32, 2), (64, 3), (96, 4)] {
        drop(
            expect_fault(&pool, &allocation, arena_key(identity, offset))
                .load(|bytes| fill(bytes, value))
                .unwrap(),
        );
    }
    let retired = ArenaReuseBarrier::CompletedRetirement;
    let pinned = expect_hit(&pool, &allocation, arena_key(identity, 64));
    let before = pool.counters();
    assert_eq!(
        pool.drain_writeback_fence(range(32, 64), retired, generation(1, 1)),
        Err(WritebackFenceDrainDenial::FrameBusy(
            PhysicalResidencyDenial::FramePinned
        ))
    );
    assert_eq!(pool.counters().frame_entries(), before.frame_entries());
    drop(pinned);
    pool.drain_writeback_fence(range(32, 64), retired, generation(1, 1))
        .unwrap();
    assert_eq!(pool.counters().frame_entries(), 2);
    assert_eq!(
        &*expect_hit(&pool, &allocation, arena_key(identity, 0)),
        &[1; 32]
    );
    assert_eq!(
        &*expect_hit(&pool, &allocation, arena_key(identity, 96)),
        &[4; 32]
    );
    let loading = expect_fault(&pool, &allocation, arena_key(identity, 32));
    assert_eq!(
        pool.drain_writeback_fence(range(32, 32), retired, generation(1, 2)),
        Err(WritebackFenceDrainDenial::FrameBusy(
            PhysicalResidencyDenial::FrameIdentityOccupied
        ))
    );
    let reloaded = loading.load(|bytes| fill(bytes, 9)).unwrap();
    assert_eq!(&*reloaded, &[9; 32]);
    assert!(reloaded.integrity_validation().is_none());
    assert_eq!(
        pool.arena_frame_identity(arena_key(identity, 32))
            .map(ArenaExtentFrameIdentity::generation),
        Some(generation(1, 1))
    );
}

#[test]
fn cancelled_writeback_claim_discards_unpinned_dirty_frame_and_stale_gen_never_poisons_fence() {
    let identity = store(116);
    let pool =
        PhysicalResidencyPool::open(identity, limits(256, 4, 2, candidate_batch_bytes(2), 8))
            .unwrap();
    pool.seed_arena_generation_floor(generation(4, 20));
    assert_eq!(
        pool.active_arena_generation(range(0, 64)),
        generation(4, 20)
    );
    // A terminal GenerationNotAdvanced refusal must NOT install a fence.
    assert_eq!(
        pool.drain_writeback_fence(
            range(0, 64),
            ArenaReuseBarrier::CompletedRetirement,
            generation(4, 20)
        ),
        Err(WritebackFenceDrainDenial::GenerationNotAdvanced {
            proposed: generation(4, 20),
            active: generation(4, 20),
        })
    );
    let allocation = candidate_allocation(&pool, 1);
    let key = arena_key(identity, 0);
    let dirty = pool
        .materialize_dirty_candidate(&allocation, key, |bytes| bytes.fill(7))
        .unwrap();
    assert_eq!(
        pool.arena_frame_identity(key)
            .map(ArenaExtentFrameIdentity::generation),
        Some(generation(4, 20))
    );
    drop(allocation);
    let paused = writeback_claim(&pool, &[key]);
    assert_eq!(
        pool.drain_writeback_fence(
            range(0, 64),
            ArenaReuseBarrier::CompletedRetirement,
            generation(4, 21)
        ),
        Err(WritebackFenceDrainDenial::WritebackInFlight { overlapping: 1 })
    );
    // New candidate admission overlapping the fenced range is refused.
    let blocked_alloc = candidate_allocation(&pool, 1);
    assert_eq!(
        pool.materialize_dirty_candidate(&blocked_alloc, arena_key(identity, 32), |b| b.fill(8))
            .unwrap_err(),
        PhysicalResidencyDenial::WriteBackRangeFenced
    );
    drop(blocked_alloc);
    // Cancelling (dropping) the paused writeback claim and dropping the dirty
    // handle leaves an unpinned, unclaimed dirty frame; drain must discard it
    // cleanly rather than livelocking.
    drop(paused);
    drop(dirty);
    let admitted = pool
        .drain_writeback_fence(
            range(0, 64),
            ArenaReuseBarrier::CompletedRetirement,
            generation(4, 21),
        )
        .unwrap();
    assert_eq!(admitted.generation(), generation(4, 21));
    assert_eq!(pool.counters().frame_entries(), 0);
    assert_eq!(pool.counters().dirty_frames(), 0);
}
