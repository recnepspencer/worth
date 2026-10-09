use super::*;

#[path = "dirty_generation_capture/allocation_window.rs"]
mod allocation_window;

fn capture_bytes(frame_count: u64) -> u64 {
    (std::mem::size_of::<PhysicalDirtyFrameBasis>() as u64)
        .checked_mul(frame_count)
        .unwrap()
}

fn maintenance(pool: &PhysicalResidencyPool, frame_count: u64) -> MaintenanceAllocationGrant {
    pool.begin_maintenance_operation(nonzero_bytes(capture_bytes(frame_count)))
        .unwrap()
}

fn materialize(
    pool: &PhysicalResidencyPool,
    allocation: &ForegroundWriteAllocationGrant,
    key: PhysicalFrameKey,
    value: u8,
) -> DirtyPhysicalFrame {
    pool.materialize_dirty_candidate(allocation, key, |bytes| bytes.fill(value))
        .unwrap()
}

fn capture_all(
    pool: &PhysicalResidencyPool,
    frames_per_slice: u64,
) -> (
    PhysicalDirtyGenerationCaptureCompletion,
    Vec<PhysicalDirtyFrameBasis>,
) {
    let mut session = pool.begin_dirty_generation_capture().unwrap();
    let mut frames = Vec::new();
    loop {
        match pool
            .capture_next_dirty_generation_slice(session, maintenance(pool, frames_per_slice))
            .unwrap()
        {
            PhysicalDirtyGenerationCaptureStep::More {
                session: next,
                slice,
            } => {
                assert!(slice.frames().len() <= frames_per_slice as usize);
                frames.extend_from_slice(slice.frames());
                session = next;
            }
            PhysicalDirtyGenerationCaptureStep::Complete { completion, slice } => {
                assert!(slice.frames().len() <= frames_per_slice as usize);
                frames.extend_from_slice(slice.frames());
                return (completion, frames);
            }
        }
    }
}

#[test]
fn session_freezes_frontier_and_slices_bound_metadata() {
    let identity = store(121);
    let pool = PhysicalResidencyPool::open(identity, limits(256, 4, 4, 2048, 4)).unwrap();
    let write = candidate_batches_allocation(&pool, &[1, 1, 1]);
    let first = PhysicalFrameKey::new(identity, coordinate(1, 16));
    let second = PhysicalFrameKey::new(identity, coordinate(2, 16));
    let later = PhysicalFrameKey::new(identity, coordinate(3, 16));
    drop(materialize(&pool, &write, second, 2));
    drop(materialize(&pool, &write, first, 1));

    let session = pool.begin_dirty_generation_capture().unwrap();
    assert_eq!(session.frontier().get(), 2);
    drop(materialize(&pool, &write, later, 3));
    let mut session = session;
    let mut captured = Vec::new();
    let completion = loop {
        match pool
            .capture_next_dirty_generation_slice(session, maintenance(&pool, 1))
            .unwrap()
        {
            PhysicalDirtyGenerationCaptureStep::More {
                session: next,
                slice,
            } => {
                assert!(slice.metadata_bytes() <= slice.admitted_bytes());
                assert!(slice.frames().len() <= 1);
                captured.extend_from_slice(slice.frames());
                session = next;
            }
            PhysicalDirtyGenerationCaptureStep::Complete { completion, slice } => {
                captured.extend_from_slice(slice.frames());
                break completion;
            }
        }
    };
    captured.sort_unstable_by_key(|basis| basis.frame().coordinate());
    assert_eq!(completion.store_identity(), identity);
    assert_eq!(completion.pool_incarnation(), pool.incarnation());
    assert_eq!(completion.frontier().get(), 2);
    assert_eq!(
        captured
            .iter()
            .map(|basis| (basis.frame(), basis.generation().get()))
            .collect::<Vec<_>>(),
        vec![(first, 2), (second, 1)]
    );
}

#[test]
fn unresolved_candidate_is_not_dirty_source() {
    let identity = store(122);
    let pool = PhysicalResidencyPool::open(identity, limits(128, 2, 2, 2048, 3)).unwrap();
    let allocation = candidate_allocation(&pool, 1);
    let candidate =
        PhysicalCandidateFrameKey::fragment(PhysicalFrameKey::new(identity, coordinate(1, 16)));
    let mut batch = pool
        .reserve_candidate_frames(&allocation, &[candidate])
        .unwrap();
    let reservation = batch.reserve_next(candidate).unwrap();

    let (completion, captured) = capture_all(&pool, 1);
    assert_eq!(completion.frontier().get(), 0);
    assert!(captured.is_empty());
    drop(reservation);
    drop(batch);
}

#[test]
fn durable_writeback_may_remove_uncaptured_source_and_redirty_is_later() {
    let identity = store(123);
    let (pool, _candidate_clean, writeback_clean) =
        PhysicalResidencyPoolOwner::open(identity, limits(128, 2, 2, 2048, 3))
            .unwrap()
            .into_parts();
    let write = pool
        .begin_foreground_write_operation(nonzero_bytes(candidate_batch_bytes(1)))
        .unwrap();
    let key = PhysicalFrameKey::new(identity, coordinate(1, 16));
    drop(materialize(&pool, &write, key, 1));
    let session = pool.begin_dirty_generation_capture().unwrap();
    writeback_claim(&pool, &[key])
        .complete_writeback(&writeback_clean)
        .unwrap();
    let clean = expect_hit(&pool, &write, key);
    drop(
        clean
            .begin_dirty_replacement(&write)
            .unwrap()
            .replace(|source, target| {
                target.copy_from_slice(source);
                target[0] = 2;
                Ok::<_, ()>(())
            })
            .unwrap(),
    );

    let PhysicalDirtyGenerationCaptureStep::Complete { completion, slice } = pool
        .capture_next_dirty_generation_slice(session, maintenance(&pool, 1))
        .unwrap()
    else {
        panic!("one-slot pool must complete one capture advance");
    };
    assert_eq!(completion.frontier().get(), 1);
    assert!(slice.frames().is_empty());
    let (_, later) = capture_all(&pool, 1);
    assert_eq!(later[0].generation().get(), 2);
}

#[test]
fn capture_rejects_foreign_session_grant_and_too_small_slice() {
    let identity = store(124);
    let pool = PhysicalResidencyPool::open(identity, limits(128, 2, 2, 1024, 3)).unwrap();
    let foreign = PhysicalResidencyPool::open(store(125), limits(128, 2, 2, 1024, 3)).unwrap();
    assert_eq!(
        pool.capture_next_dirty_generation_slice(
            foreign.begin_dirty_generation_capture().unwrap(),
            maintenance(&pool, 1),
        )
        .unwrap_err(),
        PhysicalResidencyDenial::DirtyGenerationCaptureSessionMismatch
    );
    assert_eq!(
        pool.capture_next_dirty_generation_slice(
            pool.begin_dirty_generation_capture().unwrap(),
            maintenance(&foreign, 1),
        )
        .unwrap_err(),
        PhysicalResidencyDenial::AllocationGrantMismatch
    );
    let tiny = pool.begin_maintenance_operation(NonZeroU64::MIN).unwrap();
    assert_eq!(
        pool.capture_next_dirty_generation_slice(
            pool.begin_dirty_generation_capture().unwrap(),
            tiny,
        )
        .unwrap_err(),
        PhysicalResidencyDenial::DirtyGenerationCaptureBudgetExceeded {
            required: capture_bytes(1),
            admitted: 1,
        }
    );
}

#[test]
fn capture_progress_survives_32x_resident_interleaved_mutation() {
    let identity = store(126);
    let resident = 4_u64;
    let (pool, _candidate_clean, writeback_clean) =
        PhysicalResidencyPoolOwner::open(identity, limits(512, 4, 4, 8192, 4))
            .unwrap()
            .into_parts();
    let write = candidate_batches_allocation(&pool, &[1, 1, 1, 1]);
    let mut expected = Vec::new();
    for block in 1..=resident {
        let key = PhysicalFrameKey::new(identity, coordinate(block, 16));
        expected.push(key);
        drop(materialize(&pool, &write, key, block as u8));
    }
    let session = pool.begin_dirty_generation_capture().unwrap();
    let frontier = session.frontier();
    let PhysicalDirtyGenerationCaptureStep::More {
        session,
        slice: first_slice,
    } = pool
        .capture_next_dirty_generation_slice(session, maintenance(&pool, 1))
        .unwrap()
    else {
        panic!("four resident slots require another bounded advance");
    };
    let churn_key = first_slice.frames()[0].frame();
    for value in 0..(resident * 32) {
        writeback_claim(&pool, &[churn_key])
            .complete_writeback(&writeback_clean)
            .unwrap();
        let clean = expect_hit(&pool, &write, churn_key);
        drop(
            clean
                .begin_dirty_replacement(&write)
                .unwrap()
                .replace(|source, target| {
                    target.copy_from_slice(source);
                    target[0] = value as u8;
                    Ok::<_, ()>(())
                })
                .unwrap(),
        );
    }
    let mut captured = first_slice.frames().to_vec();
    let mut session = session;
    let completion = loop {
        match pool
            .capture_next_dirty_generation_slice(session, maintenance(&pool, 1))
            .unwrap()
        {
            PhysicalDirtyGenerationCaptureStep::More {
                session: next,
                slice,
            } => {
                captured.extend_from_slice(slice.frames());
                session = next;
            }
            PhysicalDirtyGenerationCaptureStep::Complete { completion, slice } => {
                captured.extend_from_slice(slice.frames());
                break completion;
            }
        }
    };
    assert_eq!(completion.frontier(), frontier);
    assert!(captured.iter().all(|basis| basis.generation() <= frontier));
    let mut captured_keys = captured
        .iter()
        .map(|basis| basis.frame())
        .collect::<Vec<_>>();
    captured_keys.sort_unstable_by_key(|key| key.coordinate());
    expected.sort_unstable_by_key(|key| key.coordinate());
    assert_eq!(captured_keys, expected);
}

#[test]
fn exhausted_generation_denies_before_dirty_bytes_become_resident() {
    let identity = store(127);
    let pool = PhysicalResidencyPool::open(identity, limits(64, 1, 1, 1024, 2)).unwrap();
    let write = candidate_allocation(&pool, 1);
    let key = PhysicalFrameKey::new(identity, coordinate(1, 16));
    pool.force_dirty_generation_frontier(u64::MAX);
    assert_eq!(
        pool.materialize_dirty_candidate(&write, key, |bytes| bytes.fill(1))
            .unwrap_err(),
        PhysicalResidencyDenial::DirtyGenerationExhausted
    );
    assert_eq!(pool.counters().dirty_frames(), 0);
}

#[test]
fn capture_session_obeys_pool_close_boundary() {
    let identity = store(128);
    let pool = PhysicalResidencyPool::open(identity, limits(64, 1, 1, 512, 2)).unwrap();
    let session = pool.begin_dirty_generation_capture().unwrap();
    let allocation = maintenance(&pool, 1);
    assert!(pool.close().requires_inspection());
    assert_eq!(
        pool.capture_next_dirty_generation_slice(session, allocation)
            .unwrap_err(),
        PhysicalResidencyDenial::PoolClosed
    );
    assert_eq!(
        pool.begin_dirty_generation_capture().unwrap_err(),
        PhysicalResidencyDenial::PoolClosed
    );
}

#[cfg(feature = "certification-test-authority")]
#[test]
fn slice_memory_has_independent_actualization_and_release_trace() {
    use crate::{
        PhysicalResidencyAllocationBoundaryKind as Kind, PhysicalResidencyDimension as Dimension,
    };

    let identity = store(129);
    let pool = PhysicalResidencyPool::open(identity, limits(64, 1, 1, 512, 2)).unwrap();
    let observer = pool.allocation_events();
    let step = pool
        .capture_next_dirty_generation_slice(
            pool.begin_dirty_generation_capture().unwrap(),
            maintenance(&pool, 1),
        )
        .unwrap();
    let slice = match step {
        PhysicalDirtyGenerationCaptureStep::Complete { slice, .. } => slice,
        PhysicalDirtyGenerationCaptureStep::More { .. } => panic!("one slot must complete"),
    };
    let requested = slice.admitted_bytes();
    let actual = slice.metadata_bytes();
    let dimensions = [
        Dimension::OperationBytes,
        Dimension::OperationScope(PhysicalOperationAllocationScope::Maintenance),
        Dimension::TotalBytes,
    ];
    for dimension in dimensions {
        assert!(observer.trace().events().iter().any(|event| {
            event.kind() == Kind::Actualization
                && event.dimension() == dimension
                && event.requested_units() == requested
                && event.actual_units() == actual
        }));
    }
    drop(slice);
    for dimension in dimensions {
        assert!(observer.trace().events().iter().any(|event| {
            event.kind() == Kind::Release
                && event.dimension() == dimension
                && event.actual_units() == requested
        }));
    }
}
