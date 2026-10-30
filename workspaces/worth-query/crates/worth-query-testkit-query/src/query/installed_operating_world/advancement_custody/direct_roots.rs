//! Direct execution and each direct projection root keep custody ahead of source reads.
pub use super::super::installed_operation_fixture::{
    configured_runtime, ReadExecutionInput, ReadVertex,
};
pub use super::*;
pub type Workspace = worth_query::facade::runtime::WorthQueryWorkspace;
pub type Current = domain::WorthQueryCurrentDomainProjection<
    GeometryDomain,
    ReadVertex,
    ReadFamily,
    foundation::ObservationLaneWitness,
>;
pub type Live = domain::WorthQueryLiveBoundDomainProjection<
    GeometryDomain,
    ReadVertex,
    ReadFamily,
    foundation::ObservationLaneWitness,
>;

pub fn workspace(request_execution: &worth_execution::ExecutionRequest<'_, '_>) -> Workspace {
    configured_runtime()
        .workspace(request_execution, "direct-root-custody")
        .unwrap()
}
pub fn admitted(
    workspace: &Workspace,
) -> worth_query::facade::installed::operation::WorthQueryAdmittedDirectOperation<
    GeometryDomain,
    ReadVertex,
    ReadFamily,
    foundation::ObservationLaneWitness,
> {
    let installed = workspace.domain(GeometryDomain).unwrap();
    workspace
        .observe_operating_world(workspace.current_world())
        .unwrap()
        .family(ReadFamily)
        .bind(&installed, ReadVertex)
        .unwrap()
        .admit_execution_resources(
            ReadExecutionInput::default(),
            execution_resource_request(),
            workspace,
        )
        .unwrap()
}
pub fn current(workspace: &mut Workspace) -> Current {
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
            execution_resource_request(),
            workspace,
        )
        .unwrap()
        .execute(workspace)
        .unwrap()
        .publish()
        .unwrap()
        .consume(
            consumer,
            worth_query::facade::read::project_facts().entity_identities(),
        )
        .unwrap()
        .settle()
        .unwrap()
        .into_lifecycle()
}
pub fn promoted(
    request_execution: &worth_execution::ExecutionRequest<'_, '_>,
    current: Current,
    workspace: &mut Workspace,
) -> Live {
    match current.promote(request_execution, workspace) {
        domain::WorthQueryProjectionPromotionOutcome::Promoted(live) => live,
        _ => panic!("honest direct projection promotes"),
    }
}

pub mod foreign_phase_tests;
