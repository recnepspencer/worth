//! A free worker dispatches later work while an earlier item is still blocked.
use super::*;
use std::sync::{Condvar, Mutex};

#[test]
fn next_map_item_starts_before_a_blocked_item_finishes() {
    let _serial = TEST_LOCK.lock().unwrap();
    let lease = authority().request_lease(request(2, 2_000, 100)).unwrap();
    let latch = (Mutex::new((false, false, false, 0)), Condvar::new());
    let partitions = (1..=3_u64)
        .map(|value| crate::MapPartition {
            identity: PartitionIdentity::new(value),
            value,
            read_keys: Vec::<u64>::new(),
            write_keys: Vec::<u64>::new(),
            kernel_scratch_bytes: 0,
            max_result_bytes: 0,
        })
        .collect();
    let map = crate::ExecutionMap::try_from_declared_partitions(
        (1..=3).map(PartitionIdentity::new).collect(),
        partitions,
    )
    .unwrap();
    let outcome = map.run_owned(Some(&lease), |value, context| {
        context.checkpoint(1)?;
        let (lock, changed) = &latch;
        let mut state = lock.lock().unwrap();
        state.3 += 1;
        match value {
            1 => {
                state.0 = true;
                changed.notify_all();
                while !state.1 {
                    state = changed.wait(state).unwrap();
                }
                state.2 = true;
            }
            2 => {
                while !state.0 {
                    state = changed.wait(state).unwrap();
                }
            }
            3 => {
                assert!(state.0, "the first item entered its blocked interval");
                assert!(
                    !state.2,
                    "the next item starts before the blocked item finishes"
                );
                state.1 = true;
                changed.notify_all();
            }
            _ => unreachable!(),
        }
        Ok::<_, crate::MapKernelFailure<()>>(value)
    });
    let crate::MapOutcome::Complete { values, .. } = outcome else {
        panic!("all work completes")
    };
    assert_eq!(values, [1, 2, 3]);
    assert_eq!(latch.0.lock().unwrap().3, 3);
}
