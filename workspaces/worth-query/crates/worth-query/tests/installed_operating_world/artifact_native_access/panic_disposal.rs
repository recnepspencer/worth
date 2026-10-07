use super::*;

#[test]
fn native_provider_panic_unwinds_and_disposes_the_managed_artifact_once() {
    let (mut workspace, probe) = artifact_move_workspace("artifact-native-provider-panic").unwrap();
    let unwind = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        let _ = bind_artifact_workflow(&workspace)
            .admit_workflow_resources(
                crate::suite::installed_operation_fixture::execution_resource_request(),
                &workspace,
            )
            .unwrap()
            .reexecute(
                move_intent("native-provider-panic"),
                &mut workspace,
                ExecutionRequest::serial(&workflow_request()),
            );
    }));

    assert!(unwind.is_err());
    assert_eq!(probe.allocations(), 1);
    assert_eq!(probe.native_row_batches(), 1);
    assert_eq!(probe.borrow_observations(), 0);
    assert_eq!(probe.disposals(), 1);
}
