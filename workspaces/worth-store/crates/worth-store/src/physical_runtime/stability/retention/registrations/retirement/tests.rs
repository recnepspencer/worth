//! Original private C.10 gate, slot and lease ownership under retirement.
use super::*;
use crate::physical_runtime::lifecycle::LifecycleCoordinator;
use std::num::NonZeroU64;
use worth_execution::{CancellationSource, ExecutionRequest, SerialMemoryBudget, SerialRequest};

fn admitted_registry() -> (LifecycleCoordinator, Arc<RootProtectionRegistry>) {
    let lifecycle = LifecycleCoordinator::admitted();
    lifecycle.progress_to_media_owned();
    lifecycle.progress_to_record_serving();
    let registry = RootProtectionRegistry::admit(
        PhysicalReadProtectionPolicy::default(),
        RuntimeIdentity::from_reopened(NonZeroU64::MIN),
        lifecycle.observation_state(),
    )
    .unwrap();
    (lifecycle, Arc::new(registry))
}

#[test]
fn original_c10_gate_busy_preserves_shared_slot_then_retires_once() {
    let (_lifecycle, registry) = admitted_registry();
    let root = DurablePhysicalRootManifest::builder(1, 1, 2, 1)
        .admit()
        .unwrap();
    let mut lease = registry.capture(&root).unwrap();
    let mut shared = lease.clone();
    let cancellation = CancellationSource::new();
    let serial = SerialRequest::from_memory(
        SerialMemoryBudget::new(Some(1024)),
        cancellation.token(),
        None,
    );
    let request = ExecutionRequest::serial(&serial);
    let state = registry.state.lock().unwrap();
    assert!(matches!(lease.try_retire(&request), Err(Failure::Busy)));
    assert_eq!(state.live, 1);
    assert_eq!(
        state
            .slots
            .iter()
            .find(|slot| slot.root == Some(root.root_cell()))
            .unwrap()
            .owners,
        2
    );
    drop(state);
    lease.try_retire(&request).unwrap();
    assert_eq!(registry.observation().live_acquisitions(), 1);
    drop(lease);
    assert_eq!(
        registry.observation().live_acquisitions(),
        1,
        "disarmed original Drop cannot consume its remaining shared owner"
    );
    shared.try_retire(&request).unwrap();
    shared.try_retire(&request).unwrap();
    assert_eq!(registry.observation().live_acquisitions(), 0);
    assert_eq!(registry.observation().releases(), 1);
    drop(shared);
    assert_eq!(registry.observation().releases(), 1);
}

#[test]
fn original_c10_stop_preserves_armed_lease_for_unbudgeted_fallback() {
    let (_lifecycle, registry) = admitted_registry();
    let root = DurablePhysicalRootManifest::builder(1, 1, 2, 1)
        .admit()
        .unwrap();
    let mut lease = registry.capture(&root).unwrap();
    let cancellation = CancellationSource::new();
    let serial = SerialRequest::from_memory(
        SerialMemoryBudget::new(Some(1024)),
        cancellation.token(),
        None,
    );
    let request = ExecutionRequest::serial(&serial);
    cancellation.cancel();
    assert!(matches!(
        lease.try_retire(&request),
        Err(Failure::Execution(_))
    ));
    assert_eq!(registry.observation().live_acquisitions(), 1);
    assert_eq!(registry.observation().releases(), 0);
    drop(lease);
    assert_eq!(registry.observation().live_acquisitions(), 0);
    assert_eq!(registry.observation().releases(), 1);
}
