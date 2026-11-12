use super::*;
use crate::tests::support::*;

#[test]
fn refused_close_preserves_the_original_admitted_operation_and_serving_owner() {
    let runtime = runtime_with_test_schema();
    let binding = runtime.owner_binding();
    let operation = binding.admit().expect("real admission before close");
    let identity = runtime.runtime_instance_id();
    let refused = runtime.prepare_close().unwrap_err();
    assert_eq!(
        refused.reason(),
        RelationalRuntimeCloseDenial::OperationsInFlight
    );
    let runtime = refused.into_runtime();
    assert_eq!(runtime.runtime_instance_id(), identity);
    let next = binding.admit().expect("refused close restores service");
    drop(next);
    drop(operation);
    runtime.prepare_close().unwrap().finish();
    assert!(binding.admit().is_none());
}

#[test]
fn prepared_close_stops_new_admission_and_resumes_the_same_owner_on_request() {
    let runtime = runtime_with_test_schema();
    let binding = runtime.owner_binding();
    let identity = runtime.runtime_instance_id();
    let closing = runtime.prepare_close().unwrap();
    assert!(binding.admit().is_none());
    let runtime = closing.resume();
    assert_eq!(runtime.runtime_instance_id(), identity);
    drop(binding.admit().expect("explicit resume restores admission"));
    runtime.prepare_close().unwrap().finish();
}
