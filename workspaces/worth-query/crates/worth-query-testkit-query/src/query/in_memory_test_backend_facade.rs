pub use worth_foundational::facade::{AspectValue, CanonicalFieldPath, FieldKey};
pub use worth_query::facade::consumer_kit::{
    compare_test_backend_write_receipts, in_memory_test_runtime,
};
pub use worth_query::facade::foundation::WorthQueryWorkspaceErrorKind;
pub use worth_query::facade::runtime::{InvariantCatalog, InvariantRegistration, InvariantRule};
pub use worth_query::facade::runtime::{
    WorthQueryRuntimeError, WorthQueryUnrefinedLiveShape, WorthQueryWriteReceipt,
};

pub use worth_query_testkit_installed_operation_fixture::facade::query::support;

pub use support::aspect_touch as touch;
pub use support::public_bridge_runtime::PublicBridgeRuntimeHarness;

pub fn assert_test_backend_receipt_equivalence(
    in_memory: &WorthQueryWriteReceipt,
    bridge: &WorthQueryWriteReceipt,
) {
    let report = compare_test_backend_write_receipts(in_memory, bridge);
    assert!(report.matched(), "{:?}", report.rows());
    assert_eq!(report.rows().len(), 8);
    assert!(!report
        .report_identity()
        .terminal_projection_for_reporting()
        .is_empty());
}

pub fn task_schema() -> worth_query::facade::consumer_kit::WorthQueryTestBackendSchema {
    support::task_backend_schema::task_backend_schema()
}

pub fn selected_task_scalars(
    row: &worth_query::facade::foundation::WorthQueryEntity,
) -> Vec<(CanonicalFieldPath, Option<AspectValue>)> {
    ["identity.id", "title.value"]
        .into_iter()
        .map(|path| {
            let field_path = field_path(path);
            let value = row.scalar_value_at(&field_path).cloned();
            (field_path, value)
        })
        .collect()
}

pub fn field_path(path: &str) -> CanonicalFieldPath {
    CanonicalFieldPath::new(
        path.split('.').map(|segment| {
            FieldKey::new(segment).expect("test field path segment should be valid")
        }),
    )
    .expect("test field path should be non-empty")
}

pub fn authored_text(
    value: impl Into<String>,
) -> worth_query::facade::runtime::WorthQueryAuthoredAspectValue {
    worth_query::facade::runtime::WorthQueryAuthoredAspectValue::string(value)
}

pub fn assert_workspace_error_kind(
    error: WorthQueryRuntimeError,
    kind: WorthQueryWorkspaceErrorKind,
) {
    match error {
        WorthQueryRuntimeError::Workspace(workspace_error) => {
            assert_eq!(workspace_error.kind(), kind, "{workspace_error}");
        }
        other => panic!("expected workspace error `{kind:?}`, got {other:?}"),
    }
}
