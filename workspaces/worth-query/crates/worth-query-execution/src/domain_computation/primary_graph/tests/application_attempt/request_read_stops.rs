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
fn cancelled_retained_activation_native_read_keeps_the_stop() {
    with_admitted_facts(|admission, runtime, execution, snapshot, source| {
        admission.assert_cancelled_retained_activation_read(runtime, execution, snapshot, source)
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
    world
        .application
        .with_host_advancement(|phase| {
            let principal = authenticated_principal(&phase.test_host_request(), &world, &request);
            let account = resolved_account(&phase.test_host_request(), &world, "open", &request);
            let admission = admitted_operation(&phase, &world, &principal, &account, &request);
            let selected = world.selected_product(&phase.test_host_request());
            let source = CancellationSource::new();
            let serial =
                SerialRequest::from_memory(SerialMemoryBudget::new(64 << 20), source.token(), None);
            let execution = ExecutionRequest::serial(&serial);
            world
                .application
                .primary_provider
                .graph
                .with_runtime(&execution, |runtime| {
                    body(
                        &admission,
                        runtime,
                        &execution,
                        selected.application_basis().snapshot_handle(),
                        &source,
                    )
                });
        })
        .unwrap();
}
