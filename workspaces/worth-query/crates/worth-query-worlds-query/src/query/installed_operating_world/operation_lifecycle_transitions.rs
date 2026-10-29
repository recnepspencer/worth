pub use worth_query::facade::{domain, foundation, read};

pub use super::installed_operation_fixture::{
    configured_runtime, required_domain_runtime, AuxiliaryDomain, GeometryDomain,
    ReadExecutionInput, ReadFamily, ReadVertex,
};

pub type SettledProjection = domain::WorthQuerySettledDomainProjection<
    GeometryDomain,
    ReadVertex,
    ReadFamily,
    foundation::ObservationLaneWitness,
>;

pub fn promote(
    request_execution: &worth_execution::ExecutionRequest<'_, '_>,
    settled: SettledProjection,
    workspace: &mut worth_query::facade::runtime::WorthQueryWorkspace,
) -> domain::WorthQueryLiveBoundDomainProjection<
    GeometryDomain,
    ReadVertex,
    ReadFamily,
    foundation::ObservationLaneWitness,
> {
    match settled
        .into_lifecycle()
        .promote(request_execution, workspace)
    {
        domain::WorthQueryProjectionPromotionOutcome::Promoted(live) => live,
        _ => panic!("settled projection did not promote"),
    }
}

pub fn settle(
    workspace: &mut worth_query::facade::runtime::WorthQueryWorkspace,
) -> SettledProjection {
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
