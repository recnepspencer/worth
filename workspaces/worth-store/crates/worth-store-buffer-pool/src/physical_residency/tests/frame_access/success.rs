use std::sync::{mpsc, Arc, Barrier};

use super::*;

#[test]
fn hot_access_has_no_source_execution_surface() {
    let identity = store(1);
    let pool = PhysicalResidencyPool::open(identity, limits(1024, 2, 1, 64, 4)).unwrap();
    let allocation = allocation(&pool, READ_SCOPE);
    let key = PhysicalFrameKey::new(identity, coordinate(1, 32));
    let first = expect_fault(&pool, &allocation, key)
        .load(|target| {
            target.fill(7);
            Ok::<_, ()>(())
        })
        .unwrap();
    let second = expect_hit(&pool, &allocation, key);

    assert_eq!(&*first, &[7; 32]);
    assert_eq!(&*second, &[7; 32]);
    assert_eq!(pool.counters().source_loads(), 1);
    assert_eq!(pool.counters().hits(), 1);
    assert_eq!(pool.counters().pin_leases(), 2);

    let denial = pool.access_frame(&allocation, key).unwrap_err();
    assert_eq!(
        denial,
        PhysicalResidencyDenial::Pressure(PhysicalResidencyPressureDenial::new(
            identity,
            pool.incarnation(),
            PhysicalResidencyPressureDemand {
                dimension: PhysicalResidencyDimension::PinLeases,
                scope: READ_SCOPE,
                requested: 1,
                current: 2,
                limit: 2,
            },
        ))
    );
}

#[expect(
    clippy::disallowed_methods,
    reason = "category: coordinated storage-owner unit-test harness; purpose: forced overlap has one fault owner and one waiter; source: worth-store-buffer-pool::physical_residency::tests::frame_access::success::forced_overlap_has_one_fault_owner_and_one_waiter; lexical counts: std::thread::spawn=1, std::thread::scope=0, std::thread::Scope::spawn=0, available_parallelism=0; barriers, joins and original bounds coordinate the storage-owner claim"
)]
#[test]
fn forced_overlap_has_one_fault_owner_and_one_waiter() {
    let identity = store(7);
    let pool = Arc::new(PhysicalResidencyPool::open(identity, limits(2048, 2, 1, 64, 4)).unwrap());
    let key = PhysicalFrameKey::new(identity, coordinate(1, 32));
    let owner_ready = Arc::new(Barrier::new(2));
    let owner_release = Arc::new(Barrier::new(2));
    let (identity_tx, identity_rx) = mpsc::sync_channel(1);
    let worker_pool = Arc::clone(&pool);
    let worker_ready = Arc::clone(&owner_ready);
    let worker_release = Arc::clone(&owner_release);
    let worker = std::thread::spawn(move || {
        let allocation = allocation(&worker_pool, READ_SCOPE);
        let owner = expect_fault(&worker_pool, &allocation, key);
        identity_tx.send(owner.loading_identity()).unwrap();
        worker_ready.wait();
        worker_release.wait();
        owner
            .load(|target| {
                target.fill(11);
                Ok::<_, ()>(())
            })
            .unwrap()
    });

    owner_ready.wait();
    let allocation = allocation(&pool, READ_SCOPE);
    let waiter = match pool.access_frame(&allocation, key).unwrap() {
        PhysicalFrameAccess::Coalesced(waiter) => waiter,
        PhysicalFrameAccess::Hit(_) => panic!("overlap was incorrectly classified as a hit"),
        PhysicalFrameAccess::Fault(_) => panic!("a second fault owner was minted"),
    };
    assert_eq!(waiter.loading_identity(), identity_rx.recv().unwrap());
    owner_release.wait();
    let owner_lease = worker.join().unwrap();
    let waiter_lease = waiter.wait().unwrap();

    assert_eq!(&*owner_lease, &[11; 32]);
    assert_eq!(&*waiter_lease, &*owner_lease);
    let counters = pool.counters();
    assert_eq!(counters.faults(), 1);
    assert_eq!(counters.coalesced_waiters(), 1);
    assert_eq!(counters.source_loads(), 1);
    assert_eq!(counters.hits(), 0);
}

#[test]
fn stable_store_identity_rejects_foreign_frame_keys() {
    let owner = store(2);
    let pool = PhysicalResidencyPool::open(owner, limits(1024, 2, 1, 64, 4)).unwrap();
    let allocation = allocation(&pool, READ_SCOPE);
    let foreign = PhysicalFrameKey::new(store(3), coordinate(1, 16));

    assert_eq!(
        pool.access_frame(&allocation, foreign).unwrap_err(),
        PhysicalResidencyDenial::WrongStore
    );
    assert_eq!(pool.counters().resident_bytes(), 0);
}

#[test]
fn oldest_clean_unpinned_frame_evicts_deterministically_and_refaults() {
    let identity = store(4);
    let pool = PhysicalResidencyPool::open(identity, limits(1000, 2, 1, 64, 2)).unwrap();
    let allocation = allocation(&pool, READ_SCOPE);
    let oldest = PhysicalFrameKey::new(identity, coordinate(1, 300));
    let newer = PhysicalFrameKey::new(identity, coordinate(2, 300));
    let incoming = PhysicalFrameKey::new(identity, coordinate(3, 300));
    for (key, byte) in [(oldest, 1), (newer, 2), (incoming, 3)] {
        let lease = expect_fault(&pool, &allocation, key)
            .load(|target| {
                target.fill(byte);
                Ok::<_, ()>(())
            })
            .unwrap();
        drop(lease);
    }

    let counters = pool.counters();
    assert_eq!(counters.evictions(), 1);
    assert_eq!(counters.eviction_candidate_inspections(), 1);
    assert!(counters.metadata_bytes() <= 4096);
    assert!(counters.resident_bytes() <= 1000);
    assert_eq!(counters.peak_resident_bytes(), 600);

    let source_loads_before_newer = counters.source_loads();
    let newer_lease = expect_hit(&pool, &allocation, newer);
    assert_eq!(&*newer_lease, &[2; 300]);
    assert_eq!(pool.counters().source_loads(), source_loads_before_newer);
    drop(newer_lease);

    let source_loads_before_oldest = pool.counters().source_loads();
    let oldest_lease = expect_fault(&pool, &allocation, oldest)
        .load(|target| {
            target.fill(1);
            Ok::<_, ()>(())
        })
        .unwrap();
    assert_eq!(&*oldest_lease, &[1; 300]);
    assert_eq!(
        pool.counters().source_loads(),
        source_loads_before_oldest + 1
    );
}
