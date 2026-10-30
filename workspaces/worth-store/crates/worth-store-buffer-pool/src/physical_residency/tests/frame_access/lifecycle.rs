use std::sync::{Arc, Barrier};

use super::*;

#[test]
fn panicking_source_abandons_loading_and_releases_pin_budget() {
    let identity = store(11);
    let pool = PhysicalResidencyPool::open(identity, limits(1024, 1, 1, 64, 4)).unwrap();
    let allocation = allocation(&pool, READ_SCOPE);
    let key = PhysicalFrameKey::new(identity, coordinate(1, 32));
    let panic = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        let owner = expect_fault(&pool, &allocation, key);
        let _ = owner.load::<(), _>(|_| panic!("source panic"));
    }));

    assert!(panic.is_err());
    assert_eq!(pool.counters().pinned_frames(), 0);
    assert_eq!(pool.counters().active_loading_frames(), 0);
    let recovered = expect_fault(&pool, &allocation, key)
        .load(|target| {
            target.fill(3);
            Ok::<_, ()>(())
        })
        .unwrap();
    assert_eq!(&*recovered, &[3; 32]);
}

#[expect(
    clippy::disallowed_methods,
    reason = "category: coordinated storage-owner unit-test harness; purpose: close during source execution cancels unpublished frame; source: worth-store-buffer-pool::physical_residency::tests::frame_access::lifecycle::close_during_source_execution_cancels_unpublished_frame; lexical counts: std::thread::spawn=1, std::thread::scope=0, std::thread::Scope::spawn=0, available_parallelism=0; barriers, joins and original bounds coordinate the storage-owner claim"
)]
#[test]
fn close_during_source_execution_cancels_unpublished_frame() {
    let identity = store(12);
    let pool = Arc::new(PhysicalResidencyPool::open(identity, limits(1024, 1, 1, 64, 4)).unwrap());
    let key = PhysicalFrameKey::new(identity, coordinate(1, 32));
    let entered = Arc::new(Barrier::new(2));
    let release = Arc::new(Barrier::new(2));
    let worker_pool = Arc::clone(&pool);
    let worker_entered = Arc::clone(&entered);
    let worker_release = Arc::clone(&release);
    let worker = std::thread::spawn(move || {
        let allocation = allocation(&worker_pool, READ_SCOPE);
        expect_fault(&worker_pool, &allocation, key).load(|target| {
            worker_entered.wait();
            worker_release.wait();
            target.fill(4);
            Ok::<_, ()>(())
        })
    });
    entered.wait();
    let shutdown = pool.close();
    release.wait();
    let failure = worker.join().unwrap().unwrap_err();
    assert!(matches!(
        failure,
        PhysicalFrameFaultError::Residency {
            denial: PhysicalResidencyDenial::PoolClosed,
            ..
        }
    ));
    assert!(shutdown.requires_inspection());
    assert!(shutdown.has_cancellable_work_residue());
    assert_eq!(shutdown.counters().active_loading_frames(), 1);
    assert_eq!(shutdown.counters().pinned_frames(), 1);
    assert_eq!(shutdown.counters().resident_bytes(), 32);
    assert_eq!(shutdown.counters().active_operation_bytes(), 1);
    assert_eq!(pool.counters().active_loading_frames(), 0);
    assert_eq!(pool.counters().pinned_frames(), 0);
    assert_eq!(pool.counters().resident_bytes(), 0);
    assert_eq!(pool.counters().active_operation_bytes(), 0);
}
