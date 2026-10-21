use worth_query_admission::facade::graph_obligation::WorthQueryGraphWorkAdmissionDenial;
use worth_query_admission::integration::{
    admit_prepared_application_operation_graph_work, prepare_application_operation_graph_work,
};

use super::super::fixture::{installed_authorization_world, TouchAccountOperation};
use crate::domain_computation::primary_graph::application_resource_request;

#[test]
fn installed_operation_template_keeps_exact_operation_and_support_authority() {
    let current = installed_authorization_world(true);
    let foreign = installed_authorization_world(true);
    let operation = current
        .application
        .installed_schema()
        .installed_operation(TouchAccountOperation::reference())
        .expect("actual installed operation");
    let other_operation = foreign
        .application
        .installed_schema()
        .installed_operation(TouchAccountOperation::reference())
        .expect("same declaration in a foreign installed runtime");
    let request = application_resource_request(operation.contracts())
        .expect("actual installed execution strategy has a request");
    let current_support = current.application.graph_work_resource_support_ref();
    let foreign_support = foreign.application.graph_work_resource_support_ref();
    let prepared = prepare_application_operation_graph_work(&operation, &request, current_support)
        .expect("the actual installed operation is supported");

    let plan = admit_prepared_application_operation_graph_work(
        &operation,
        &prepared,
        "actual-invocation-1",
        current_support,
    )
    .expect("matching installed operation and provider admit");
    drop(plan);

    let foreign_operation = admit_prepared_application_operation_graph_work(
        &other_operation,
        &prepared,
        "foreign-invocation",
        current_support,
    );
    assert!(matches!(
        foreign_operation,
        Err(WorthQueryGraphWorkAdmissionDenial::OperationAuthorityMismatch)
    ));

    let foreign_provider = admit_prepared_application_operation_graph_work(
        &operation,
        &prepared,
        "foreign-provider-invocation",
        foreign_support,
    );
    assert!(matches!(
        foreign_provider,
        Err(WorthQueryGraphWorkAdmissionDenial::ProviderSupportUnavailable)
    ));
}
