pub use worth_foundational::facade::{FieldKey, InternedString};
pub use worth_query::facade::{domain, foundation, read};

pub use super::installed_operation_fixture::{
    workflow_workspace, workspace, GeometryDomain, ReadExecutionInput, ReadFamily, ReadVertex,
    WorkflowRead,
};

pub fn native_id_request<D, O, F, L: foundation::BasisOperationLane>(
    consumer: domain::WorthQueryConsumerProjectionContract<D, O, F, L>,
) -> (
    domain::WorthQueryBoundProjectionRequest<D, O, F, L>,
    domain::WorthQueryNativeAccessKey,
) {
    let mut builder = consumer.projection_request();
    let selection = builder
        .select_display_native_field(FieldKey::new("id").unwrap())
        .unwrap();
    let request = builder.build().unwrap();
    let key = request.resolve_native_key(&selection).unwrap().into_key();
    (request, key)
}
