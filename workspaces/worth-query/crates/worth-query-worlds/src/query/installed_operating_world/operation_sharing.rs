pub use worth_query::facade::{domain, foundation, read, runtime};

pub use super::installed_operation_fixture::{
    configured_runtime, GeometryDomain, ReadExecutionInput, ReadFamily, ReadVertex,
};

pub type SettledProjection = domain::WorthQuerySettledDomainProjection<
    GeometryDomain,
    ReadVertex,
    ReadFamily,
    foundation::ObservationLaneWitness,
>;

pub fn settle(workspace: &mut runtime::WorthQueryWorkspace) -> SettledProjection {
    let installed = workspace.domain(GeometryDomain).unwrap();
    let bound = workspace
        .observe_operating_world(workspace.current_world())
        .unwrap()
        .family(ReadFamily)
        .bind(&installed, ReadVertex)
        .unwrap();
    let consumer = bound.consumer_projection_contract().unwrap();
    bound
        .admit_execution_resources(
            ReadExecutionInput::default(),
            crate::facade::query::suite::installed_operation_fixture::execution_resource_request(),
            &*workspace,
        )
        .unwrap()
        .execute(workspace)
        .unwrap()
        .publish()
        .unwrap()
        .consume(consumer, read::project_facts().entity_identities())
        .unwrap()
        .settle()
        .unwrap()
}
