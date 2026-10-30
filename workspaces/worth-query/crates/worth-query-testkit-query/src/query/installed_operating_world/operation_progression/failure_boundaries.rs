pub use worth_proof::TransitionOutcome;
pub use worth_query::facade::domain;

pub use worth_query_testkit_installed_operation_fixture::facade::query::suite::installed_operation_fixture::{
    foreign_material_workspace, workspace, GeometryDomain, ReadExecutionInput, ReadFamily,
    ReadVertex,
};

pub fn operation_failure(
    request_execution: &worth_execution::ExecutionRequest<'_, '_>,
    name: &str,
    class: domain::WorthQueryOperationFailureClass,
) -> domain::WorthQueryBoundExecutionDenial {
    let mut workspace = workspace(request_execution, name, false).unwrap();
    let installed_domain = workspace.domain(GeometryDomain).unwrap();
    let bound = workspace
        .observe_operating_world(workspace.current_world())
        .unwrap()
        .family(ReadFamily)
        .bind(&installed_domain, ReadVertex)
        .unwrap();
    match bound
        .admit_execution_resources(
            ReadExecutionInput {
                failure: Some(class),
                ..Default::default()
            },
            worth_query_testkit_installed_operation_fixture::facade::query::suite::installed_operation_fixture::execution_resource_request(),
            &workspace,
        )
        .unwrap()
        .execute(&mut workspace)
    {
        TransitionOutcome::Failed(denial) => denial,
        _ => panic!("deliberately failing executor did not produce an execution failure"),
    }
}
