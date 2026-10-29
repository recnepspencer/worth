pub use worth_foundational::facade::{AspectKey, CanonicalFieldPath, FieldKey};
pub use worth_query::facade::consumer_kit::in_memory_test_runtime;
pub use worth_query::facade::runtime::WorthQueryWorkspace;

pub use worth_query_testkit_installed_operation_fixture::facade::query::support;

pub mod live_journey {
    pub use worth_query::facade::live::{
        current, declare, AspectFieldSelector, AspectName, AuthoredResultShapeField, FieldName,
        QuerySchemaView, ScalarAspectType, SchemaFieldView, WorthQueryAuthorityLane,
        WorthQueryManagedLiveCloseOutcome, WorthQueryManagedLiveDeliveryCauseKind,
        WorthQueryManagedLiveLifecyclePosture, WorthQueryManagedLiveSubscriptionFamily,
        WorthQueryOrdinaryRuntimePostureKind,
    };
}

pub fn insert_task(
    request_execution: &worth_execution::ExecutionRequest<'_, '_>,
    workspace: &mut WorthQueryWorkspace,
) {
    workspace
        .insert(request_execution, "Task", |task| {
            task.set_aspect(
                worth_query::facade::runtime::WorthQueryAspectTouch::aspect_field_path(
                    AspectKey::new("identity").expect("identity aspect should build"),
                    CanonicalFieldPath::new([
                        FieldKey::new("id").expect("identity field should build")
                    ])
                    .expect("identity path should build"),
                ),
                worth_query::facade::runtime::WorthQueryAuthoredAspectValue::string("task-1"),
            )
        })
        .expect("public live delivery fixture should write through the real runtime");
}

pub fn live_workspace(
    request_execution: &worth_execution::ExecutionRequest<'_, '_>,
) -> WorthQueryWorkspace {
    in_memory_test_runtime()
        .with_schema(support::task_backend_schema::task_backend_schema())
        .workspace(request_execution, "declarative-live-public-dx")
        .expect("public live test workspace should build")
}
