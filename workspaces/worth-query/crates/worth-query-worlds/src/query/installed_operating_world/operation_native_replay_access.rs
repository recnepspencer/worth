pub use worth_foundational::facade::{FieldKey, InternedString};
pub use worth_query::facade::{certification, domain, foundation};

pub use super::installed_operation_fixture::{
    workflow_workspace, GeometryDomain, ReadFamily, WorkflowRead,
};
pub use super::operation_reexecution::intent;

pub fn native_replay_request<D, O, F, L: foundation::BasisOperationLane>(
    consumer: domain::WorthQueryConsumerProjectionContract<D, O, F, L>,
) -> (
    domain::WorthQueryBoundProjectionRequest<D, O, F, L>,
    Vec<domain::WorthQueryNativeAccessKey>,
) {
    let mut builder = consumer.projection_request();
    let display = builder
        .select_display_native_field(FieldKey::new("id").unwrap())
        .unwrap();
    let derived = builder
        .select_derived_native_field(FieldKey::new("id").unwrap())
        .unwrap();
    let request = builder.build().unwrap();
    let keys = [display, derived]
        .iter()
        .map(|selection| request.resolve_native_key(selection).unwrap().into_key())
        .collect();
    (request, keys)
}

pub fn bind(
    workspace: &worth_query::facade::runtime::WorthQueryWorkspace,
) -> domain::WorthQueryBoundDomainOperation<
    GeometryDomain,
    WorkflowRead,
    ReadFamily,
    foundation::ObservationLaneWitness,
> {
    let installed = workspace.domain(GeometryDomain).unwrap();
    workspace
        .observe_operating_world(workspace.current_world())
        .unwrap()
        .family(ReadFamily)
        .bind(&installed, WorkflowRead)
        .unwrap()
}
