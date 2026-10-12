//! Real authorization observations retain cancellation through native reads.
use super::super::fixture::{Account, IdentityExecutionSchema, TouchAccountInput};
use super::*;
use crate::domain_computation::primary_graph::WorthQueryAdmittedApplicationOperation;
use worth_execution::{CancellationSource, ExecutionRequest, SerialMemoryBudget, SerialRequest};
use worth_relational::facade::{runtime::RelationalRuntime, snapshots::SnapshotHandle};
type Admission = WorthQueryAdmittedApplicationOperation<
    IdentityExecutionSchema,
    TouchAccountOperation,
    TouchAccountInput,
    Account,
>;
#[test]
fn authorization_fact_keeps_the_stop_at_every_consult() {
    with_admitted_facts(|admission, runtime, execution, snapshot, source| {
        admission.assert_cancelled_fact_dispatch(runtime, execution, snapshot, source)
    });
}
#[test]
fn retained_activation_keeps_the_stop_at_every_consult() {
    with_admitted_facts(|admission, runtime, execution, snapshot, source| {
        admission
            .assert_retained_activation_read_at_every_consult(runtime, execution, snapshot, source)
    });
}
#[test]
fn retained_equality_dispatch_stop_oracle() {
    with_admitted_facts(|admission, runtime, execution, snapshot, _| {
        admission.assert_fact_dispatch_stop_oracle(runtime, execution, snapshot)
    });
}
fn with_admitted_facts(
    body: impl FnOnce(
        &Admission,
        &RelationalRuntime,
        &ExecutionRequest<'_, '_>,
        &SnapshotHandle,
        &CancellationSource,
    ),
) {
    let _observation =
        crate::facade::primary_graph::WorthQueryExecutionObservation::install_on_this_thread();
    let world = installed_authorization_world(true);
    let request = live_scope();
    let (admission, selected) = world
        .application
        .with_host_advancement(|phase| {
            let principal = authenticated_principal(&phase.test_host_request(), &world, &request);
            let account = resolved_account(&phase.test_host_request(), &world, "open", &request);
            let admission = admitted_operation(&phase, &world, &principal, &account, &request);
            let selected = world.selected_product(&phase.test_host_request());
            (admission, selected)
        })
        .unwrap();
    let source = CancellationSource::new();
    let serial =
        SerialRequest::from_memory(SerialMemoryBudget::new(64 << 20), source.token(), None);
    let execution = ExecutionRequest::serial(&serial);
    let outcome = world
        .application
        .primary_provider
        .graph
        .with_runtime(&execution, |runtime| {
            std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                body(
                    &admission,
                    runtime,
                    &execution,
                    selected.application_basis().snapshot_handle(),
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
        admission.assert_cancelled_retained_activation_read(runtime, execution, snapshot, source)
    });
}
