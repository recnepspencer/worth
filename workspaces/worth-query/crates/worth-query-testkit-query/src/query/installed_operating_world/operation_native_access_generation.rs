pub use worth_foundational::facade::FieldKey;
pub use worth_query::facade::{domain, foundation};

pub use super::installed_operation_fixture::{
    configured_runtime, GeometryDomain, ReadExecutionInput, ReadFamily, ReadVertex,
};

pub fn bind(
    workspace: &worth_query::facade::runtime::WorthQueryWorkspace,
    domain: &domain::WorthQueryInstalledDomainHandle<GeometryDomain>,
) -> domain::WorthQueryBoundDomainOperation<
    GeometryDomain,
    ReadVertex,
    ReadFamily,
    foundation::ObservationLaneWitness,
> {
    workspace
        .observe_operating_world(workspace.current_world())
        .unwrap()
        .family(ReadFamily)
        .bind(domain, ReadVertex)
        .unwrap()
}

pub fn native_id_request(
    bound: &domain::WorthQueryBoundDomainOperation<
        GeometryDomain,
        ReadVertex,
        ReadFamily,
        foundation::ObservationLaneWitness,
    >,
) -> (
    domain::WorthQueryBoundProjectionRequest<
        GeometryDomain,
        ReadVertex,
        ReadFamily,
        foundation::ObservationLaneWitness,
    >,
    domain::WorthQueryNativeAccessKey,
) {
    let mut builder = bound
        .consumer_projection_contract()
        .unwrap()
        .projection_request();
    let selection = builder
        .select_display_native_field(FieldKey::new("id").unwrap())
        .unwrap();
    let request = builder.build().unwrap();
    let key = request.resolve_native_key(&selection).unwrap().into_key();
    (request, key)
}
