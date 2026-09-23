use super::*;
use crate::facade::BridgeExecutionBasisLifecycleSignalStatus;
use crate::source::runtime_storage_for_test;

#[test]
fn terminal_observer_retains_its_request_runtime_and_releases_it_on_drop() {
    let installed = runtime(BridgeRuntimePolicy::development());
    let lane = installed.fork_managed_request_lane();
    let key = lane.signal_runtime_key;
    let basis = admit(&lane);
    let observer = basis.lifecycle_observer();
    drop(lane);

    basis
        .finalize(BridgeExecutionBasisTerminalDisposition::Completed)
        .unwrap();
    assert_eq!(
        observer.observe().unwrap().signal_status(),
        Some(BridgeExecutionBasisLifecycleSignalStatus::Fulfilled)
    );
    assert_eq!(runtime_storage_for_test(key), (true, true, false, true));

    drop(observer);
    assert_eq!(runtime_storage_for_test(key), (false, false, false, false));
}

#[test]
fn managed_request_runtime_lives_until_its_last_request_holder() {
    let installed = runtime(BridgeRuntimePolicy::development());
    for _ in 0..64 {
        let lane = installed.fork_managed_request_lane();
        let key = lane.signal_runtime_key;
        let basis = admit(&lane);
        let request = basis.request().clone();
        drop(lane);
        assert_eq!(runtime_storage_for_test(key), (true, true, false, true));
        basis
            .finalize(BridgeExecutionBasisTerminalDisposition::Completed)
            .unwrap();
        assert_eq!(runtime_storage_for_test(key), (true, true, false, true));
        drop(request);
        assert_eq!(runtime_storage_for_test(key), (false, false, false, false));
    }
}

#[test]
fn yielded_and_abandoned_requests_release_their_runtime_after_custody_ends() {
    let installed = runtime(BridgeRuntimePolicy::development());
    for yielded in [false, true] {
        let lane = installed.fork_managed_request_lane();
        let key = lane.signal_runtime_key;
        let basis = admit(&lane);
        drop(lane);
        if yielded {
            let yielded = basis.yield_execution_basis().unwrap();
            assert_eq!(runtime_storage_for_test(key), (true, true, false, true));
            yielded.release();
        } else {
            drop(basis);
        }
        assert_eq!(runtime_storage_for_test(key), (false, false, false, false));
    }
}

#[test]
fn foreign_last_drop_queues_affine_destruction_for_the_next_owner_access() {
    let installed = runtime(BridgeRuntimePolicy::development());
    let lane = installed.fork_managed_request_lane();
    let key = lane.signal_runtime_key;
    admit(&lane)
        .finalize(BridgeExecutionBasisTerminalDisposition::Completed)
        .unwrap();
    let foreign = lane.clone();
    std::thread::spawn(move || {
        assert!(with_async_request_signal_runtime(foreign.signal_runtime_key, |_| ()).is_err());
        drop(foreign);
    })
    .join()
    .unwrap();
    assert_eq!(runtime_storage_for_test(key), (true, true, false, true));
    std::thread::spawn(move || drop(lane)).join().unwrap();
    assert_eq!(runtime_storage_for_test(key), (true, true, false, false));
    with_async_request_signal_runtime(installed.signal_runtime_key, |_| ()).unwrap();
    assert_eq!(runtime_storage_for_test(key), (false, false, false, false));
}

fn admit(lane: &RuntimeBridge) -> crate::facade::BridgeBoundExecutionBasis {
    lane.admit_managed_execution_basis(
        managed_intent("custody-attempt"),
        step_contract(),
        truth_basis("snapshot-a"),
        planned_truth_view(lane),
    )
    .unwrap()
}
