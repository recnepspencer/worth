use worth_runtime_world::facade::*;
fn illegal_stage(
    port: RuntimeWorldPublicationPort<(), (), (), (), ()>,
    execution: worth_execution::ExecutionRequest<'_, '_>,
    prepared: PreparedCompositePublicationWithSignal,
    token: &RuntimeWorldCancellationToken,
) {
    port.execute_without_signal(execution, prepared, token);
}
fn main() {}
