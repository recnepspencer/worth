//! Conformance trace emitter for `StoreM4.ExtentGenerationSafety`, driven by
//! the real compiled `ExtentArenaRange`, `ExtentAllocationGeneration`, and
//! `PhysicalResidencyPool` (`drain_writeback_fence`, `seed_arena_generation_floor`,
//! `active_arena_generation`, `require_arena_frame_identity`).

use super::Trace;
use std::num::{NonZeroU32, NonZeroU64, NonZeroUsize};
use worth_store_buffer_pool::{
    ArenaReuseBarrier, PhysicalFrameKey, PhysicalOperationAllocationScope, PhysicalResidencyDenial,
    PhysicalResidencyLimits, PhysicalResidencyPool, PhysicalResidencyPoolOwner,
    PhysicalSpeculativeWorkKind, WritebackFenceDrainDenial,
};
use worth_store_physical_format::{
    store_namespace::{
        ProposedStoreIdentity, StableStoreIdentity, StoreNamespaceIdentityRecord,
        StoreNamespaceVersion,
    },
    ArenaExtentFrameIdentity, ExtentAllocationGeneration, ExtentArenaId, ExtentArenaRange,
    RecordArtifactFile, RecordFrameCoordinate,
};

const ARENA: u64 = 9;

fn range(arena: u64, offset: u64, length: u64) -> ExtentArenaRange {
    ExtentArenaRange::new(ExtentArenaId::new(arena).unwrap(), offset, length).unwrap()
}

fn generation(root: u64, seq: u64) -> ExtentAllocationGeneration {
    ExtentAllocationGeneration::at_durable_barrier(root, seq).unwrap()
}

fn range_json(r: ExtentArenaRange) -> String {
    format!("[{},{},{}]", r.arena().get(), r.offset(), r.length())
}

fn gen_json(g: ExtentAllocationGeneration) -> String {
    format!(
        "[{},{}]",
        g.root_publication_generation(),
        g.extent_fence_seq()
    )
}

fn ident_json(i: ArenaExtentFrameIdentity) -> String {
    format!(
        r#"{{"range":{},"gen":{}}}"#,
        range_json(i.range()),
        gen_json(i.generation())
    )
}

fn barrier_str(b: ArenaReuseBarrier) -> &'static str {
    match b {
        ArenaReuseBarrier::RetirementIncomplete => "retirement_incomplete",
        ArenaReuseBarrier::CompletedRetirement => "completed_retirement",
        ArenaReuseBarrier::CancelUncovered => "cancel_uncovered",
        ArenaReuseBarrier::CheckpointCoveredCancel => "checkpoint_covered_cancel",
    }
}

fn store_id(byte: u8) -> StableStoreIdentity {
    StoreNamespaceIdentityRecord::new(
        StoreNamespaceVersion::CURRENT,
        ProposedStoreIdentity::from_nonzero_bytes([byte; 16]).unwrap(),
    )
    .published_identity()
}

fn arena_key(identity: StableStoreIdentity, offset: u64) -> PhysicalFrameKey {
    PhysicalFrameKey::new(
        identity,
        RecordFrameCoordinate::new(RecordArtifactFile::ExtentArena { arena: ARENA }, offset, 32)
            .unwrap(),
    )
}

fn pool_limits(candidate_count: usize) -> PhysicalResidencyLimits {
    use PhysicalOperationAllocationScope as Scope;
    use PhysicalSpeculativeWorkKind as Speculation;
    let op_bytes = PhysicalResidencyPool::candidate_batch_operation_bytes(
        NonZeroUsize::new(candidate_count).unwrap(),
    )
    .unwrap()
    .get();
    let resident_bytes = 256_u64;
    let meta_bytes = 8192_u64;
    let total_bytes = resident_bytes * 2 + op_bytes + meta_bytes;
    let nz = |v| NonZeroU64::new(v).unwrap();
    let nzc = |v| NonZeroU32::new(v).unwrap();
    PhysicalResidencyLimits::builder()
        .total_bytes(nz(total_bytes))
        .resident_bytes(nz(resident_bytes))
        .metadata_bytes(nz(meta_bytes))
        .frame_entries(nzc(8))
        .pinned_frames(nzc(4))
        .pin_leases(nzc(4))
        .dirty_frames(nzc(2))
        .dirty_replacement_bytes(nz(resident_bytes))
        .operation_bytes(nz(op_bytes))
        .scope_bytes(Scope::ForegroundRead, nz(op_bytes))
        .scope_bytes(Scope::ForegroundWrite, nz(op_bytes))
        .scope_bytes(Scope::Recovery, nz(op_bytes))
        .scope_bytes(Scope::Scrub, nz(op_bytes))
        .scope_bytes(Scope::Maintenance, nz(op_bytes))
        .scope_bytes(Scope::Verification, nz(op_bytes))
        .scope_bytes(Scope::Blob, nz(op_bytes))
        .speculative_frames(Speculation::Prefetch, nzc(4))
        .speculative_frames(Speculation::ReadAhead, nzc(4))
        .speculative_frames(Speculation::WriteBehind, nzc(2))
        .admit(NonZeroU64::MIN)
        .unwrap()
}

fn record_overlap_and_order(trace: &mut Trace) {
    let base = range(3, 4096, 8192);
    for other in [
        range(3, 4096, 8192),
        range(3, 4096, 4096),
        range(3, 8192, 4096),
        range(3, 0, 16384),
        range(3, 0, 4096),
        range(3, 12288, 4096),
        range(4, 4096, 8192),
    ] {
        let overlaps = base.overlaps(other);
        trace.push(
            "extent_overlap",
            format!(
                r#""a":{},"b":{},"overlaps":{overlaps}"#,
                range_json(base),
                range_json(other)
            ),
        );
    }

    let pairs = [
        (ExtentAllocationGeneration::OPENED, generation(7, 40)),
        (generation(7, 40), generation(7, 41)),
        (generation(7, 41), generation(8, 1)),
        (generation(7, 41), generation(7, 41)),
        (generation(8, 1), generation(7, 41)),
    ];
    for (a, b) in pairs {
        trace.push(
            "extent_gen_order",
            format!(r#""a":{},"b":{},"lt":{}"#, gen_json(a), gen_json(b), a < b),
        );
    }
}

#[allow(clippy::too_many_arguments)]
fn push_drain(
    trace: &mut Trace,
    pool: &PhysicalResidencyPool,
    r: ExtentArenaRange,
    barrier: ArenaReuseBarrier,
    next: ExtentAllocationGeneration,
    floor: ExtentAllocationGeneration,
    seen: &mut Vec<ArenaExtentFrameIdentity>,
    inflight: &[ExtentArenaRange],
) -> Result<ArenaExtentFrameIdentity, WritebackFenceDrainDenial> {
    let seen_json = format!(
        "[{}]",
        seen.iter()
            .map(|i| ident_json(*i))
            .collect::<Vec<_>>()
            .join(",")
    );
    let inflight_json = format!(
        "[{}]",
        inflight
            .iter()
            .map(|w| range_json(*w))
            .collect::<Vec<_>>()
            .join(",")
    );
    let res = pool.drain_writeback_fence(r, barrier, next);
    let (admitted, ret_json) = match res {
        Ok(id) => {
            seen.push(id);
            (true, gen_json(id.generation()))
        }
        Err(_) => (false, "null".to_owned()),
    };
    let active_after = pool.active_arena_generation(r);
    trace.push(
        "extent_drain",
        format!(
            r#""range":{},"barrier":"{}","next_gen":{},"floor_gen":{},"seen":{seen_json},"inflight":{inflight_json},"admitted":{admitted},"returned_gen":{ret_json},"active_after":{}"#,
            range_json(r),
            barrier_str(barrier),
            gen_json(next),
            gen_json(floor),
            gen_json(active_after),
        ),
    );
    res
}

pub(super) fn record(trace: &mut Trace) {
    record_overlap_and_order(trace);

    let identity = store_id(77);
    let (pool, _, writeback_clean) = PhysicalResidencyPoolOwner::open(identity, pool_limits(2))
        .unwrap()
        .into_parts();
    let op_bytes =
        PhysicalResidencyPool::candidate_batch_operation_bytes(NonZeroUsize::new(2).unwrap())
            .unwrap();
    let alloc = pool.begin_foreground_write_operation(op_bytes).unwrap();
    let key = arena_key(identity, 64);
    let dirty = pool
        .materialize_dirty_candidate(&alloc, key, |b| b.fill(4))
        .unwrap();
    drop(alloc);
    let wb_alloc = pool
        .begin_foreground_write_operation(NonZeroU64::new(32).unwrap())
        .unwrap();
    let paused = pool.claim_writeback(wb_alloc, &[key]).unwrap();
    let stale = paused.arena_identity(0).unwrap();

    let mut seen = Vec::new();
    let floor = ExtentAllocationGeneration::OPENED;
    let inflight = [range(ARENA, 64, 32)];

    // Incomplete barriers refused even while writeback is in flight.
    for barrier in [
        ArenaReuseBarrier::RetirementIncomplete,
        ArenaReuseBarrier::CancelUncovered,
    ] {
        assert_eq!(
            push_drain(
                trace,
                &pool,
                range(ARENA, 64, 32),
                barrier,
                generation(5, 10),
                floor,
                &mut seen,
                &inflight,
            ),
            Err(WritebackFenceDrainDenial::BarrierIncomplete(barrier))
        );
    }

    // Exact, split, and coalesced drains refused while writeback is in flight.
    for (offset, len) in [(64, 32), (80, 16), (0, 256)] {
        assert_eq!(
            push_drain(
                trace,
                &pool,
                range(ARENA, offset, len),
                ArenaReuseBarrier::CompletedRetirement,
                generation(5, 10),
                floor,
                &mut seen,
                &inflight,
            ),
            Err(WritebackFenceDrainDenial::WritebackInFlight { overlapping: 1 })
        );
    }

    // Complete writeback and release dirty frame.
    paused.complete_writeback(&writeback_clean).unwrap();
    drop(dirty);

    // Incomplete barriers MUST still be refused on a fully drained range with
    // no overlapping in-flight writeback and a fresh advancing generation.
    for barrier in [
        ArenaReuseBarrier::RetirementIncomplete,
        ArenaReuseBarrier::CancelUncovered,
    ] {
        assert_eq!(
            push_drain(
                trace,
                &pool,
                range(ARENA, 64, 32),
                barrier,
                generation(5, 10),
                floor,
                &mut seen,
                &[],
            ),
            Err(WritebackFenceDrainDenial::BarrierIncomplete(barrier))
        );
    }

    // Completed barrier on the drained range now admits coalesced [0, 256) at (5, 10).
    let admitted = push_drain(
        trace,
        &pool,
        range(ARENA, 0, 256),
        ArenaReuseBarrier::CompletedRetirement,
        generation(5, 10),
        floor,
        &mut seen,
        &[],
    )
    .unwrap();
    assert_eq!(admitted.generation(), generation(5, 10));

    // Stale generation (5, 10) refused on Path B cancel; advancing (5, 11) admitted.
    assert!(matches!(
        push_drain(
            trace,
            &pool,
            range(ARENA, 64, 32),
            ArenaReuseBarrier::CheckpointCoveredCancel,
            generation(5, 10),
            floor,
            &mut seen,
            &[],
        ),
        Err(WritebackFenceDrainDenial::GenerationNotAdvanced { .. })
    ));
    let reused = push_drain(
        trace,
        &pool,
        range(ARENA, 64, 32),
        ArenaReuseBarrier::CheckpointCoveredCancel,
        generation(5, 11),
        floor,
        &mut seen,
        &[],
    )
    .unwrap();

    // Reload frame at reused generation and check stale vs current lookups.
    let alloc = pool
        .begin_foreground_write_operation(
            PhysicalResidencyPool::candidate_batch_operation_bytes(NonZeroUsize::new(1).unwrap())
                .unwrap(),
        )
        .unwrap();
    let fresh = pool
        .materialize_dirty_candidate(&alloc, key, |b| b.fill(6))
        .unwrap();
    let frame = pool.arena_frame_identity(key).unwrap();
    for query in [
        stale,
        ArenaExtentFrameIdentity::new(range(ARENA, 64, 32), generation(5, 10)),
        reused,
    ] {
        let res = pool.require_arena_frame_identity(key, query);
        let ok = res.is_ok();
        if !ok {
            assert_eq!(res, Err(PhysicalResidencyDenial::StaleArenaGeneration));
        }
        trace.push(
            "extent_lookup",
            format!(
                r#""frame":{},"query_range":{},"query_gen":{},"ok":{ok}"#,
                ident_json(frame),
                range_json(query.range()),
                gen_json(query.generation()),
            ),
        );
    }
    drop(alloc);
    fresh.discard_candidate().unwrap();

    // Post-crash reopen with durable floor (5, 11): equal gen refused, (5, 12) admitted.
    let reopened = PhysicalResidencyPool::open(store_id(78), pool_limits(1)).unwrap();
    let reopen_floor = generation(5, 11);
    reopened.seed_arena_generation_floor(reopen_floor);
    let mut reopen_seen = Vec::new();
    assert!(matches!(
        push_drain(
            trace,
            &reopened,
            range(ARENA, 0, 64),
            ArenaReuseBarrier::CompletedRetirement,
            generation(5, 11),
            reopen_floor,
            &mut reopen_seen,
            &[],
        ),
        Err(WritebackFenceDrainDenial::GenerationNotAdvanced { .. })
    ));
    assert!(push_drain(
        trace,
        &reopened,
        range(ARENA, 0, 64),
        ArenaReuseBarrier::CompletedRetirement,
        generation(5, 12),
        reopen_floor,
        &mut reopen_seen,
        &[],
    )
    .is_ok());
}
