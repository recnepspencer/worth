use worth_proof::TransitionOutcome;
use worth_query::facade::domain;

use crate::suite::installed_operation_fixture::{
    foreign_material_workspace, workspace, GeometryDomain, ReadExecutionInput, ReadFamily,
    ReadVertex,
};

#[test]
fn ordinary_executor_failures_are_typed_and_must_be_declared() {
    let declared = operation_failure(
        "ordinary-declared-failure",
        domain::WorthQueryOperationFailureClass::Dependency,
    );
    assert_eq!(
        declared.kind(),
        &domain::WorthQueryBoundExecutionDenialKind::Executor(
            domain::WorthQueryOperationFailureClass::Dependency,
        )
    );
    let undeclared = operation_failure(
        "ordinary-undeclared-failure",
        domain::WorthQueryOperationFailureClass::Unsupported,
    );
    assert_eq!(
        undeclared.kind(),
        &domain::WorthQueryBoundExecutionDenialKind::UndeclaredFailureClass(
            domain::WorthQueryOperationFailureClass::Unsupported,
        )
    );
}

fn operation_failure(
    name: &str,
    class: domain::WorthQueryOperationFailureClass,
) -> domain::WorthQueryBoundExecutionDenial {
    let mut workspace = workspace(name, false).unwrap();
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
            crate::suite::installed_operation_fixture::execution_resource_request(),
            &workspace,
        )
        .unwrap()
        .execute(&mut workspace)
    {
        TransitionOutcome::Failed(denial) => denial,
        _ => panic!("deliberately failing executor did not produce an execution failure"),
    }
}

#[test]
fn unsupported_primary_read_basis_denies_before_execution_work() {
    let workspace = workspace("installed-foreign-basis-material", false).unwrap();
    let installed_domain = workspace.domain(GeometryDomain).unwrap();
    let branch = workspace
        .branches()
        .fork(workspace.current_world())
        .components(|components| components.reuse_exact_relational_basis().fork_signal())
        .create()
        .unwrap();
    let denial = match workspace
        .observe_operating_world(branch)
        .unwrap()
        .family(ReadFamily)
        .bind(&installed_domain, ReadVertex)
    {
        Ok(_) => panic!("unsupported primary-read basis unexpectedly bound"),
        Err(denial) => denial,
    };

    assert_eq!(
        denial.kind(),
        domain::WorthQueryOperationBindingDenialKind::BasisExecutionUnsupported
    );
    assert_eq!(denial.counters().graph_provider_contacts, 0);
    assert_eq!(denial.counters().planning_steps, 0);
}

#[test]
fn same_shaped_read_from_a_foreign_runtime_cannot_publish() {
    let mut workspace = foreign_material_workspace("foreign-runtime-material-owner").unwrap();
    let installed_domain = workspace.domain(GeometryDomain).unwrap();
    let executed = workspace
        .observe_operating_world(workspace.current_world())
        .unwrap()
        .family(ReadFamily)
        .bind(&installed_domain, ReadVertex)
        .unwrap()
        .admit_execution_resources(
            ReadExecutionInput::default(),
            crate::suite::installed_operation_fixture::execution_resource_request(),
            &workspace,
        )
        .unwrap()
        .execute(&mut workspace)
        .unwrap();
    assert_eq!(executed.counters().executor_contacts, 1);
    assert!(matches!(
        executed.publish(),
        TransitionOutcome::Denied(domain::WorthQueryPublicationDenial::ExecutionMaterialMismatch)
    ));
}
