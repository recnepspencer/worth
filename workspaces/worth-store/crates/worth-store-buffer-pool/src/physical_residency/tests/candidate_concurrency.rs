use std::sync::{Arc, Barrier};

use super::*;

#[test]
fn disjoint_candidate_sessions_and_reads_progress_independently() {
    let identity = store(16);
    let operation_bytes = candidate_batches_bytes(&[1, 1]) + 1;
    let pool =
        PhysicalResidencyPool::open(identity, limits(256, 3, 2, operation_bytes, 6)).unwrap();
    let read_allocation = allocation(&pool, READ_SCOPE);
    let write_allocation = candidate_batches_allocation(&pool, &[1, 1]);
    let stable_key = PhysicalFrameKey::new(identity, coordinate(1, 16));
    drop(
        expect_fault(&pool, &read_allocation, stable_key)
            .load(|bytes| fill(bytes, 3))
            .unwrap(),
    );
    let first_key = PhysicalFrameKey::new(identity, coordinate(2, 32));
    let second_key = PhysicalFrameKey::new(identity, coordinate(3, 32));
    let first_candidate = PhysicalCandidateFrameKey::fragment(first_key);
    let second_candidate = PhysicalCandidateFrameKey::fragment(second_key);
    let mut first = pool
        .reserve_candidate_frames(&write_allocation, &[first_candidate])
        .unwrap();
    let mut second = pool
        .reserve_candidate_frames(&write_allocation, &[second_candidate])
        .unwrap();
    let first_dirty = first
        .reserve_next(first_candidate)
        .unwrap()
        .materialize(|bytes| bytes.fill(4))
        .unwrap();
    let second_dirty = second
        .reserve_next(second_candidate)
        .unwrap()
        .materialize(|bytes| bytes.fill(5))
        .unwrap();
    let stable = expect_hit(&pool, &read_allocation, stable_key);
    assert_eq!(&*stable, &[3; 16]);
    drop(stable);
    first_dirty.discard_candidate().unwrap();
    second_dirty.discard_candidate().unwrap();
    drop(first);
    drop(second);
    drop(read_allocation);
    drop(write_allocation);
    assert!(!pool.close().requires_inspection());
}

#[expect(
    clippy::disallowed_methods,
    reason = "category: coordinated storage-owner unit-test harness; purpose: unrelated faults do not hold the metadata lock during io; source: worth-store-buffer-pool::physical_residency::tests::candidate_concurrency::unrelated_faults_do_not_hold_the_metadata_lock_during_io; lexical counts: std::thread::spawn=1, std::thread::scope=0, std::thread::Scope::spawn=0, available_parallelism=0; barriers, joins and original bounds coordinate the storage-owner claim"
)]
#[test]
fn unrelated_faults_do_not_hold_the_metadata_lock_during_io() {
    let identity = store(6);
    let pool = Arc::new(PhysicalResidencyPool::open(identity, limits(2048, 2, 1, 64, 4)).unwrap());
    let barrier = Arc::new(Barrier::new(2));
    let mut workers = Vec::new();
    for block in 1..=2 {
        let pool = Arc::clone(&pool);
        let barrier = Arc::clone(&barrier);
        workers.push(std::thread::spawn(move || {
            let allocation = allocation(&pool, READ_SCOPE);
            let key = PhysicalFrameKey::new(identity, coordinate(block, 32));
            expect_fault(&pool, &allocation, key)
                .load(|target| {
                    barrier.wait();
                    target.fill(block as u8);
                    Ok::<_, ()>(())
                })
                .unwrap()
        }));
    }
    let leases: Vec<_> = workers
        .into_iter()
        .map(|worker| worker.join().unwrap())
        .collect();
    assert_eq!(leases.len(), 2);
    assert_eq!(pool.counters().source_loads(), 2);
}
