use super::support::public_bridge_runtime::PublicBridgeRuntimeHarness;
use worth_foundational::facade::{AspectKey, CanonicalFieldPath, FieldKey};
use worth_query::facade::foundation::{basis_lifecycle, ScopedInspectionBasis};
use worth_query::facade::mutation::{
    WorthQueryAspectTouch, WorthQueryAuthoredAspectValue, WorthQueryMutationDeclaration,
};
use worth_query::facade::read::{
    AspectFieldSelector, AspectName, AuthoredResultShapeField, FieldName, QuerySchemaView,
    ScalarAspectType, SchemaFieldView, WorthQueryReadDenial,
};
use worth_query::facade::runtime::{WorthQueryReadBuilder, WorthQueryWorkspace};

pub fn workspace(
    request_execution: &worth_execution::ExecutionRequest<'_, '_>,
    name: &str,
) -> WorthQueryWorkspace {
    PublicBridgeRuntimeHarness::new()
        .bridge_backed_runtime(request_execution)
        .workspace(name)
        .expect("product boundary workspace should open")
}

pub fn identity_detail<Output>(
    read: WorthQueryReadBuilder<Output>,
) -> Result<Output, WorthQueryReadDenial> {
    read.local_detail(
        "Task",
        identity_schema("product-boundary-detail"),
        |query| query.project(identity_selector()),
        |shape| shape.field(identity_field()),
    )
}

pub fn identity_collection<Output>(
    read: WorthQueryReadBuilder<Output>,
) -> Result<Output, WorthQueryReadDenial> {
    read.local_collection(
        "Task",
        identity_schema("product-boundary-collection"),
        |query| query.project(identity_selector()),
        |shape| shape.field(identity_field()),
    )
}

pub fn mutation(value: &str) -> WorthQueryMutationDeclaration {
    worth_query::facade::mutation::declare(|mutation| {
        mutation
            .set_aspect(
                WorthQueryAspectTouch::from_authoring_ingress_text("identity.id")?,
                WorthQueryAuthoredAspectValue::string(value),
            )
            .build_insert("Task")
    })
    .expect("product boundary mutation should declare")
}

pub fn write_task(
    request_execution: &worth_execution::ExecutionRequest<'_, '_>,
    workspace: &mut WorthQueryWorkspace,
    id: &str,
) {
    workspace
        .insert(request_execution, "Task", |task| {
            task.set_aspect(
                WorthQueryAspectTouch::aspect_field_path(
                    AspectKey::new("identity").expect("identity aspect should build"),
                    CanonicalFieldPath::new([
                        FieldKey::new("id").expect("identity field should build")
                    ])
                    .expect("identity path should build"),
                ),
                WorthQueryAuthoredAspectValue::string(id),
            )
        })
        .expect("task should be written through the runtime");
}

pub fn inspection_basis(label: &str) -> ScopedInspectionBasis {
    basis_lifecycle()
        .historical_snapshot(format!("product-boundary-{label}"), true)
        .inspect()
        .expect("inspection basis should admit")
}

fn identity_schema(label: &str) -> QuerySchemaView {
    QuerySchemaView::new(
        label,
        [SchemaFieldView::new(
            AspectName::new("identity").expect("identity aspect should build"),
            FieldName::new("id").expect("identity field should build"),
            ScalarAspectType::String,
        )],
        [],
    )
}

fn identity_selector() -> AspectFieldSelector {
    AspectFieldSelector::new("identity", "id").expect("identity selector should build")
}

fn identity_field() -> AuthoredResultShapeField {
    AuthoredResultShapeField::new("identity", "id", "identity.id")
        .expect("identity field should build")
}
