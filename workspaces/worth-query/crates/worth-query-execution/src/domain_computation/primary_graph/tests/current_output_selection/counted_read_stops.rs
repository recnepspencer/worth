use super::*;
use crate::domain_computation::primary_graph::WorthQueryInvariantProjectionDenialKind as Invariant;
use worth_relational::facade::transactions::RelationalExecutionDenialCause as Cause;
#[test]
fn current_output_door_preserves_stops_at_every_consult() {
    let world = installed_authorization_world(true);
    let admission = world
        .application
        .with_host_advancement(|phase| {
            let (request, principal, scope, operation) =
                admitted_touch_parts(&phase, &phase.borrowed_request_for_test(), &world);
            world
                .selected_product(&phase.test_host_request())
                .authorize_operation(
                    &phase,
                    &principal,
                    &scope,
                    &operation,
                    Default::default(),
                    &request,
                )
                .unwrap()
        })
        .unwrap();
    world
        .application
        .runtime
        .primary_graph()
        .unwrap()
        .integration_handle()
        .output_lineage
        .lock()
        .unwrap()
        .install_output_families(std::collections::BTreeMap::from([(
            TestCurrentOutputFamily::IDENTITY.to_owned().into(),
            Vec::new(),
        )]));
    crate::test_read_stops::every_consult(
        |execution| {
            world
                .invariant
                .project_admitted_operation_with_request(
                    execution,
                    &admission,
                    |reader, root| {
                        reader
                            .current_output::<TestCurrentOutputFamily, Account>(root)
                            .map(|value| matches!(value, WorthQueryCurrentOutputSelection::Missing))
                    },
                    worth_execution::ExecutionAllocationPolicy::SystemAllocation,
                )
                .map(|completed| completed.into_parts().0.expect("live output query"))
                .map_err(|denial| denial.kind())
        },
        WorthQueryOperationProjectionDenialKind::InvariantAdmission(Invariant::ExecutionRequest(
            Cause::Cancelled,
        )),
        WorthQueryOperationProjectionDenialKind::InvariantAdmission(Invariant::ExecutionRequest(
            Cause::WorkExhausted,
        )),
    );
}
