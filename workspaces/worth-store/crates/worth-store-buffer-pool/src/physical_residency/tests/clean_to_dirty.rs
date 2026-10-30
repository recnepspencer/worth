use super::*;
use std::{
    sync::{mpsc, Arc},
    time::Duration,
};

#[test]
fn exact_clean_lease_becomes_one_dirty_candidate_atomically() {
    let identity = store(91);
    let pool = PhysicalResidencyPool::open(identity, limits(128, 3, 2, 64, 3)).unwrap();
    let allocation = pool
        .begin_foreground_write_operation(nonzero_bytes(8))
        .unwrap();
    let key = PhysicalFrameKey::new(identity, coordinate(1, 8));
    let clean = expect_fault(&pool, &allocation, key)
        .load(|target| {
            target.copy_from_slice(&[1; 8]);
            Ok::<_, ()>(())
        })
        .unwrap();

    let dirty = clean
        .begin_dirty_replacement(&allocation)
        .unwrap()
        .replace(|source, target| {
            assert_eq!(source, &[1; 8]);
            target.copy_from_slice(&[2; 8]);
            Ok::<_, ()>(())
        })
        .unwrap();
    assert_eq!(dirty.bytes(), &[2; 8]);
    assert_eq!(pool.counters().dirty_frames(), 1);
    assert_eq!(pool.counters().candidate_frames(), 1);

    let claim = writeback_claim(&pool, &[key]);
    assert_eq!(claim.frame_bytes(0), Some([2; 8].as_slice()));
    drop(claim);
    drop(dirty);
    assert_eq!(pool.counters().dirty_frames(), 1);
}

#[test]
fn competing_pin_prevents_clean_to_dirty_transition() {
    let identity = store(92);
    let pool = PhysicalResidencyPool::open(identity, limits(128, 3, 2, 64, 3)).unwrap();
    let allocation = pool
        .begin_foreground_write_operation(nonzero_bytes(8))
        .unwrap();
    let key = PhysicalFrameKey::new(identity, coordinate(1, 8));
    let first = expect_fault(&pool, &allocation, key)
        .load(|target| {
            target.copy_from_slice(&[1; 8]);
            Ok::<_, ()>(())
        })
        .unwrap();
    let second = expect_hit(&pool, &allocation, key);

    assert_eq!(
        first.begin_dirty_replacement(&allocation).unwrap_err(),
        PhysicalResidencyDenial::FramePinned
    );
    assert_eq!(second.as_ref(), &[1; 8]);
    assert_eq!(pool.counters().dirty_frames(), 0);
}

#[expect(
    clippy::disallowed_methods,
    reason = "category: coordinated storage-owner unit-test harness; purpose: pin racing after replacement admission preserves the original allocation; source: worth-store-buffer-pool::physical_residency::tests::clean_to_dirty::pin_racing_after_replacement_admission_preserves_the_original_allocation; lexical counts: std::thread::spawn=1, std::thread::scope=0, std::thread::Scope::spawn=0, available_parallelism=0; barriers, joins and original bounds coordinate the storage-owner claim"
)]
#[test]
fn pin_racing_after_replacement_admission_preserves_the_original_allocation() {
    let identity = store(93);
    let pool = Arc::new(PhysicalResidencyPool::open(identity, limits(128, 3, 2, 64, 3)).unwrap());
    let observer = pool.allocation_events();
    let write = pool
        .begin_foreground_write_operation(nonzero_bytes(8))
        .unwrap();
    let key = PhysicalFrameKey::new(identity, coordinate(1, 8));
    let clean = expect_fault(&pool, &write, key)
        .load(|target| {
            target.copy_from_slice(&[1; 8]);
            Ok::<_, ()>(())
        })
        .unwrap();
    let original_pointer = clean.as_ref().as_ptr() as usize;
    let replacement = clean.begin_dirty_replacement(&write).unwrap();
    let timeout = Duration::from_secs(5);
    let (enter_race, await_race) = mpsc::channel();
    let (pin_acquired, await_pin) = mpsc::channel();
    let (observe_after_denial, await_observation) = mpsc::channel();

    let worker_pool = Arc::clone(&pool);
    let worker = std::thread::spawn(move || {
        let read = allocation(&worker_pool, READ_SCOPE);
        await_race
            .recv_timeout(timeout)
            .expect("replacement fill must open the pin race");
        let pinned = expect_hit(&worker_pool, &read, key);
        let before = (pinned.as_ref().as_ptr() as usize, pinned.as_ref().to_vec());
        pin_acquired
            .send(before)
            .expect("replacement thread must await the competing pin");
        await_observation
            .recv_timeout(timeout)
            .expect("failed replacement must release the competing view");
        (pinned.as_ref().as_ptr() as usize, pinned.as_ref().to_vec())
    });

    let mut raced_view = None;
    let denial = replacement
        .replace(|source, target| {
            assert_eq!(source.as_ptr() as usize, original_pointer);
            assert_eq!(source, &[1; 8]);
            target.copy_from_slice(&[2; 8]);
            enter_race
                .send(())
                .expect("competing pin thread must remain live");
            raced_view = Some(
                await_pin
                    .recv_timeout(timeout)
                    .expect("competing pin must arrive before replacement finish"),
            );
            Ok::<_, ()>(())
        })
        .unwrap_err();

    assert_eq!(
        denial,
        PhysicalDirtyReplacementError::Residency(PhysicalResidencyDenial::FramePinned)
    );
    assert_eq!(pool.counters().dirty_frames(), 0);
    assert_eq!(pool.counters().dirty_replacement_bytes(), 0);
    let replacement_events = observer
        .snapshot()
        .for_dimension(PhysicalResidencyDimension::DirtyReplacementBytes);
    assert_eq!(replacement_events.admissions(), 1);
    assert_eq!(replacement_events.releases(), 1);
    assert_eq!(replacement_events.active_units(), 0);

    observe_after_denial
        .send(())
        .expect("competing view must remain live through replacement denial");
    let before = raced_view.expect("fill observed the competing immutable view");
    let after = worker.join().unwrap();
    assert_eq!(before.0, original_pointer);
    assert_eq!(after.0, original_pointer);
    assert_eq!(before.1, vec![1; 8]);
    assert_eq!(after.1, vec![1; 8]);

    let clean = expect_hit(&pool, &write, key);
    assert_eq!(clean.as_ref().as_ptr() as usize, original_pointer);
    assert_eq!(clean.as_ref(), &[1; 8]);
}
