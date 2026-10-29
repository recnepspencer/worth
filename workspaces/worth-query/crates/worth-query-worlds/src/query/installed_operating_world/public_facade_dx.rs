pub use worth_query::facade::{domain, installed};

pub mod operational_lifecycle;
pub mod single_root;

pub use super::installed_operation_fixture::{
    conditional_workspace, lineage_workflow_workspace, workspace, GeometryDomain,
    LineageEvidenceScenario, ReadExecutionInput, ReadFamily, ReadVertex,
};
pub use super::operation_native_access_matrix::{
    fixture::{insert_matrix_value, matrix_workspace, NativeMatrixRead},
    samples::matrix_value_with_order,
};

pub fn assert_threshold_family_executes_through_facade(
    request_execution: &worth_execution::ExecutionRequest<'_, '_>,
    declaration: domain::WorthQueryPortableConditionalNodeDeclaration,
) {
    let mut workspace = conditional_workspace(
        request_execution,
        "installed-public-facade-conditional-threshold",
        declaration,
    )
    .unwrap();
    let domain = workspace.domain(GeometryDomain).unwrap();
    let baseline = workspace
        .observe_operating_world(workspace.current_world())
        .unwrap()
        .family(ReadFamily)
        .bind(&domain, ReadVertex)
        .unwrap();
    let installed::transition::WorthQueryExecutionTransition::Deferred(baseline) =
        installed::transition::execution(
            baseline
                .admit_execution_resources(
                    ReadExecutionInput::default(),
                    crate::facade::query::suite::installed_operation_fixture::execution_resource_request(),
                    &workspace,
                )
                .unwrap()
                .execute(&mut workspace),
        )
    else {
        panic!("the typed threshold must establish its baseline before it can compare a delta")
    };
    assert_eq!(
        baseline.conditional_provenance()[0].class(),
        installed::conditional::WorthQueryConditionalOutcomeClass::Suppressed
    );

    assert_eq!(baseline.counters().conditional_compute_contacts, 0);
}
