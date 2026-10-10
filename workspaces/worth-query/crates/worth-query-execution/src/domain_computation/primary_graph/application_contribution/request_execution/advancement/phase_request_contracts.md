A Bridge read seam requires the caller's request.

```compile_fail,E0061
use worth_runtime_bridge::facade::{TruthSnapshotReader, SnapshotReadPacket};
fn read(reader: &dyn TruthSnapshotReader, packet: &SnapshotReadPacket) {
    let _ = reader.read_packet(packet);
}
```
```
use worth_execution::ExecutionRequest;
use worth_runtime_bridge::facade::{TruthSnapshotReader, SnapshotReadPacket};
fn read(reader: &dyn TruthSnapshotReader, packet: &SnapshotReadPacket, request: ExecutionRequest<'_, '_>) {
    let _ = reader.read_packet(packet, request);
}
```

A wake slot cannot retain the request borrowed from an advancement.

```compile_fail,E0521
use worth_execution::ExecutionRequest;
use worth_query_execution::facade::product::WorthQueryPrimaryGraphApplicationRuntime;
use worth_query_installation::facade::ApplicationSchema;
use worth_query_admission::facade::authenticated_principal::WorthQueryRequestScope;
fn pause<'held, Schema: ApplicationSchema + 'static>(
    app: &WorthQueryPrimaryGraphApplicationRuntime<Schema>, scope: &WorthQueryRequestScope,
    runtime: &worth_query_execution::facade::integration::WorthQueryProductRuntime,
    wake: &mut Option<ExecutionRequest<'held, 'held>>,
) {
    let _ = app.with_application_advancement(scope, |phase| {
        *wake = Some(phase.execution_request_for(runtime).unwrap());
    });
}
```
```
use worth_query_execution::facade::product::WorthQueryPrimaryGraphApplicationRuntime;
use worth_query_installation::facade::ApplicationSchema;
use worth_query_admission::facade::authenticated_principal::WorthQueryRequestScope;
fn pause<Schema: ApplicationSchema + 'static>(
    app: &WorthQueryPrimaryGraphApplicationRuntime<Schema>, scope: &WorthQueryRequestScope,
    runtime: &worth_query_execution::facade::integration::WorthQueryProductRuntime,
    wake: &mut Option<()>,
) {
    let _ = app.with_application_advancement(scope, |phase| {
        let _ = phase.execution_request_for(runtime).unwrap();
        *wake = Some(phase.complete());
    });
}
```

A queued product delivery cannot outlive the advancement lending its phase.

```compile_fail,E0521
use std::sync::Arc;
use worth_query_execution::facade::{integration::WorthQueryProductSharedRoot, product::{WorthQueryPrimaryGraphApplicationRuntime, WorthQueryPerformedRelationalProductChange}};
use worth_query_installation::facade::ApplicationSchema;
use worth_query_admission::facade::authenticated_principal::WorthQueryRequestScope;
fn deliver<Schema: ApplicationSchema + 'static>(
    app: &WorthQueryPrimaryGraphApplicationRuntime<Schema>, scope: &WorthQueryRequestScope,
    root: &WorthQueryProductSharedRoot, lowering: &Arc<worth_runtime_bridge::facade::BridgeInstalledConditionalLowering>,
    change: WorthQueryPerformedRelationalProductChange,
) {
    let mut queued_phase = None;
    app.with_application_advancement(scope, |phase| {
        queued_phase = Some(&phase);
    }).unwrap();
    let _ = root.deliver_performed_relational_change(queued_phase.unwrap(), lowering, 0, change);
}
```
```
use std::sync::Arc;
use worth_query_execution::facade::{integration::WorthQueryProductSharedRoot, product::{WorthQueryPrimaryGraphApplicationRuntime, WorthQueryPerformedRelationalProductChange}};
use worth_query_installation::facade::ApplicationSchema;
use worth_query_admission::facade::authenticated_principal::WorthQueryRequestScope;
fn deliver<Schema: ApplicationSchema + 'static>(
    app: &WorthQueryPrimaryGraphApplicationRuntime<Schema>, scope: &WorthQueryRequestScope,
    root: &WorthQueryProductSharedRoot, lowering: &Arc<worth_runtime_bridge::facade::BridgeInstalledConditionalLowering>,
    change: WorthQueryPerformedRelationalProductChange,
) {
    let _ = app.with_application_advancement(scope, |phase| {
        root.deliver_performed_relational_change(&phase, lowering, 0, change)
    });
}
```
