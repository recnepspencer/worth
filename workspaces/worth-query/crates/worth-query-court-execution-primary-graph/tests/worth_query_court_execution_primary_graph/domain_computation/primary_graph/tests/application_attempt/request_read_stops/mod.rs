//! Real authorization observations retain cancellation through native reads.
use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::Account;
use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::IdentityExecutionSchema;
use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::schema_types::TouchAccountInput;
use crate::process::domain_computation::primary_graph::tests::application_attempt::*;
use worth_query_execution::facade::domain_computation::WorthQueryAdmittedApplicationOperation;
use worth_execution::CancellationSource;
use worth_execution::ExecutionRequest;
use worth_execution::SerialMemoryBudget;
use worth_execution::SerialRequest;
use worth_relational::facade::runtime::RelationalRuntime;
use worth_relational::facade::snapshots::SnapshotHandle;
pub type Admission = WorthQueryAdmittedApplicationOperation<
    IdentityExecutionSchema,
    TouchAccountOperation,
    TouchAccountInput,
    Account,
>;
#[test]
fn authorization_fact_keeps_the_stop_at_every_consult() {
    with_admitted_facts(|admission, runtime, execution, snapshot, source| {
        admission.assert_cancelled_fact_dispatch_for_test(runtime, execution, snapshot, source)
    });
}
#[test]
fn retained_activation_keeps_the_stop_at_every_consult() {
    with_admitted_facts(|admission, runtime, execution, snapshot, source| {
        admission.assert_retained_activation_read_at_every_consult_for_test(
            runtime, execution, snapshot, source,
        )
    });
}
#[test]
fn retained_equality_dispatch_stop_oracle() {
    with_admitted_facts(|admission, runtime, execution, snapshot, _| {
        admission.assert_fact_dispatch_stop_oracle_for_test(runtime, execution, snapshot)
    });
}
pub fn with_admitted_facts(
    body: impl FnOnce(
        &Admission,
        &RelationalRuntime,
        &ExecutionRequest<'_, '_>,
        &SnapshotHandle,
        &CancellationSource,
    ),
) {
    let _observation =
        worth_query_execution::facade::domain_computation::WorthQueryExecutionObservation::install_on_this_thread();
    let world = installed_authorization_world(
        true,
        worth_query_admission::facade::authenticated_principal::WorthQueryAdmissionClock::system(),
    );
    let request = live_scope();
    let (admission, selected) = world
        .application
        .with_host_advancement(|phase| {
            let principal =
                authenticated_principal(&phase.test_host_request_for_test(), &world, &request);
            let account = resolved_account(
                &phase.test_host_request_for_test(),
                &world,
                "open",
                &request,
            );
            let admission = admitted_operation(&phase, &world, &principal, &account, &request);
            let selected = world.selected_product(&phase.test_host_request_for_test());
            (admission, selected)
        })
        .unwrap();
    let source = CancellationSource::new();
    let serial = SerialRequest::from_memory(SerialMemoryBudget::new(None), source.token(), None);
    let execution = ExecutionRequest::serial(&serial);
    let outcome = world
        .application
        .runtime
        .primary_provider
        .graph
        .with_runtime(&execution, |runtime| {
            std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                body(
                    &admission,
                    runtime,
                    &execution,
                    selected.application_basis_for_test().snapshot_handle(),
                    &source,
                )
            }))
        });
    if let Err(payload) = outcome {
        std::panic::resume_unwind(payload);
    }
}

#[test]
fn cancelled_retained_activation_native_read_keeps_the_stop() {
    with_admitted_facts(|admission, runtime, execution, snapshot, source| {
        admission.assert_cancelled_retained_activation_read_for_test(
            runtime, execution, snapshot, source,
        )
    });
}
