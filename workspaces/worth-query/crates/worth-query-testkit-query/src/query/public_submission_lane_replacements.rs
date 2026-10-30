pub use worth_foundational::facade::{AspectValue, CanonicalFieldPath, FieldKey, InternedString};
pub use worth_query::facade::runtime::{
    WorthQueryAspectMutationBuilder, WorthQueryLiveView, WorthQueryMutationFamily,
    WorthQueryUnrefinedLiveShape, WorthQueryWriteCommand,
};

pub use worth_query_testkit_installed_operation_fixture::facade::query::support;

pub use support::aspect_touch as touch;
pub use support::public_bridge_runtime::PublicBridgeRuntimeHarness;

pub fn task_insert_command(id: &str, title: &str) -> WorthQueryWriteCommand {
    WorthQueryAspectMutationBuilder::new()
        .set_aspect(touch("identity.id"), authored_text(id))
        .set_aspect(touch("title.value"), authored_text(title))
        .build_insert("Task")
        .expect("task insert command should build")
}

pub fn task_live_view(
    request_execution: &worth_execution::ExecutionRequest<'_, '_>,
    workspace: &mut worth_query::facade::runtime::WorthQueryWorkspace,
    name: &str,
) -> WorthQueryLiveView<WorthQueryUnrefinedLiveShape> {
    workspace
        .live_view(request_execution, name, |q| {
            q.from("Task")
                .select([
                    worth_query::facade::foundation::AspectFieldKey::from_authoring_parts(
                        "identity", "id",
                    )
                    .unwrap(),
                    worth_query::facade::foundation::AspectFieldKey::from_authoring_parts(
                        "title", "value",
                    )
                    .unwrap(),
                ])
                .order_by(
                    worth_query::facade::foundation::AspectFieldKey::from_authoring_parts(
                        "identity", "id",
                    )
                    .unwrap(),
                )
                .schema_basis(format!("{name}-schema"))
        })
        .expect("task live view should declare")
}

pub fn authored_text(
    value: impl Into<String>,
) -> worth_query::facade::runtime::WorthQueryAuthoredAspectValue {
    worth_query::facade::runtime::WorthQueryAuthoredAspectValue::string(value)
}

pub fn text(value: impl Into<String>) -> worth_foundational::facade::AspectValue {
    worth_foundational::facade::AspectValue::String(value.into().into())
}

pub fn field_path(path: &str) -> CanonicalFieldPath {
    CanonicalFieldPath::new(
        path.split('.').map(|segment| {
            FieldKey::new(segment).expect("test field path segment should be valid")
        }),
    )
    .expect("test field path should be non-empty")
}

pub fn scalar_text(value: Option<&AspectValue>) -> Option<&str> {
    match value? {
        AspectValue::String(InternedString::Raw(value)) => Some(value.as_str()),
        AspectValue::String(InternedString::Symbol(_)) => None,
        _ => None,
    }
}
