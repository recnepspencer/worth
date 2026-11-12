//! Actual owner identity, local alias discharge and unchanged refused handles.
use super::*;
use crate::tests::support::{create_entity, runtime_with_test_schema};
use std::sync::Mutex;
use worth_execution::{CancellationSource, SerialMemoryBudget, SerialRequest};

#[test]
fn immutable_alias_retirement_keeps_owner_serving_and_original_slot_until_reclaim() {
    let stop = CancellationSource::new();
    let host = SerialRequest::from_memory(SerialMemoryBudget::new(None), stop.token(), None);
    let request = ExecutionRequest::serial(&host);
    let runtime = Arc::new(runtime_with_test_schema());
    let handle = RelationalRuntimeHandle::immutable(Arc::clone(&runtime));
    let original_slot = Arc::downgrade(&handle.lock_observation);
    let retired = handle
        .try_retire_immutable_alias(&request, &runtime)
        .unwrap();
    create_entity(&request, &runtime, "owner-still-serving-after-alias");
    Arc::try_unwrap(runtime)
        .unwrap()
        .prepare_close()
        .unwrap()
        .finish();
    assert!(original_slot.upgrade().is_some());
    retired.reclaim();
    assert!(original_slot.upgrade().is_none());
}

#[test]
fn immutable_alias_retirement_rejects_same_state_cached_identity_and_foreign_owner() {
    let stop = CancellationSource::new();
    let host = SerialRequest::from_memory(SerialMemoryBudget::new(None), stop.token(), None);
    let request = ExecutionRequest::serial(&host);
    let runtime = Arc::new(runtime_with_test_schema());
    let operation = runtime.owner_binding().admit().unwrap();
    let admitted = Arc::new(RelationalRuntime::admitted(
        runtime.state_binding().upgrade().unwrap(),
        operation,
    ));
    let handle = RelationalRuntimeHandle::immutable(Arc::clone(&admitted));
    assert_eq!(handle.runtime_instance_id(), runtime.runtime_instance_id());
    let (handle, cause) = handle
        .try_retire_immutable_alias(&request, &runtime)
        .unwrap_err();
    assert_eq!(
        cause,
        RelationalRuntimeHandleAliasRetirementDenial::ForeignOwner
    );
    assert!(
        matches!(&handle.ownership, RuntimeOwnership::Immutable(original) if Arc::ptr_eq(original, &admitted))
    );
    assert_eq!(
        handle.validate_immutable_owner_alias(&admitted),
        Err(RelationalRuntimeHandleAliasRetirementDenial::NotOwner)
    );
    drop(handle);
    drop(admitted);
    let other = runtime_with_test_schema();
    let handle = RelationalRuntimeHandle::immutable(Arc::clone(&runtime));
    let slot = Arc::downgrade(&handle.lock_observation);
    let (handle, cause) = handle
        .try_retire_immutable_alias(&request, &other)
        .unwrap_err();
    assert_eq!(
        cause,
        RelationalRuntimeHandleAliasRetirementDenial::ForeignOwner
    );
    assert!(Arc::ptr_eq(
        &handle.lock_observation,
        &slot.upgrade().unwrap()
    ));
    drop(handle);
    Arc::try_unwrap(runtime)
        .unwrap()
        .prepare_close()
        .unwrap()
        .finish();
    other.prepare_close().unwrap().finish();
}

#[test]
fn immutable_alias_retirement_shared_and_stopped_refusals_preserve_original_handle() {
    let stop = CancellationSource::new();
    let host = SerialRequest::from_memory(SerialMemoryBudget::new(None), stop.token(), None);
    let request = ExecutionRequest::serial(&host);
    let shared = Arc::new(Mutex::new(runtime_with_test_schema()));
    let handle = RelationalRuntimeHandle::shared(Arc::clone(&shared));
    let (handle, cause) = handle
        .try_retire_immutable_alias(&request, &shared.lock().unwrap())
        .unwrap_err();
    assert_eq!(
        cause,
        RelationalRuntimeHandleAliasRetirementDenial::SharedRepresentation
    );
    assert!(
        matches!(&handle.ownership, RuntimeOwnership::Shared(original) if Arc::ptr_eq(original, &shared))
    );
    drop(handle);
    Arc::try_unwrap(shared)
        .unwrap()
        .into_inner()
        .unwrap()
        .prepare_close()
        .unwrap()
        .finish();
    let runtime = Arc::new(runtime_with_test_schema());
    let handle = RelationalRuntimeHandle::immutable(Arc::clone(&runtime));
    let slot = Arc::downgrade(&handle.lock_observation);
    stop.cancel();
    let (handle, cause) = handle
        .try_retire_immutable_alias(&request, &runtime)
        .unwrap_err();
    assert!(matches!(
        cause,
        RelationalRuntimeHandleAliasRetirementDenial::Execution(_)
    ));
    assert!(
        matches!(&handle.ownership, RuntimeOwnership::Immutable(original) if Arc::ptr_eq(original, &runtime))
    );
    assert!(Arc::ptr_eq(
        &handle.lock_observation,
        &slot.upgrade().unwrap()
    ));
    drop(handle);
    Arc::try_unwrap(runtime)
        .unwrap()
        .prepare_close()
        .unwrap()
        .finish();
}
