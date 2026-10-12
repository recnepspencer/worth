use crate::consumer_kit::test_backend::{
    in_memory_test_product_world_resources, in_memory_test_runtime, task_schema,
};
use worth_query_execution::facade::application_contribution::WorthQueryAdvancementDenial;

#[test]
fn mutable_advancement_refuses_nested_opening_and_releases_the_host_scope() {
    let host = crate::bridge_host_request::serial_request();
    let execution = worth_execution::ExecutionRequest::serial(&host);
    let mut workspace = in_memory_test_runtime()
        .with_schema(task_schema())
        .product_world_resources(in_memory_test_product_world_resources())
        .workspace(&execution, "mutable-phase")
        .unwrap();
    let mut contacts = 0;
    workspace
        .with_advancement_mut(|phase, workspace| {
            assert!(workspace.snapshot_identity(&phase).is_ok());
            assert!(matches!(
                workspace.with_advancement(|_| contacts += 1),
                Err(WorthQueryAdvancementDenial::NestedOpening)
            ));
            assert!(matches!(
                workspace.with_advancement_mut(|_, _| contacts += 1),
                Err(WorthQueryAdvancementDenial::NestedOpening)
            ));
        })
        .unwrap();
    assert_eq!(contacts, 0);
    workspace
        .with_advancement_mut(|phase, workspace| {
            assert!(workspace.snapshot_identity(&phase).is_ok());
            contacts += 1;
        })
        .unwrap();
    assert_eq!(contacts, 1);
}
