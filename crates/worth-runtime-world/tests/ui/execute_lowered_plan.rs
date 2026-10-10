use worth_execution::ExecutionRequest;
use worth_runtime_world::facade::*;
fn illegal_stage(
    execution: ExecutionRequest<'_, '_>,
    port: RuntimeWorldPublicationPort<(), (), (), (), ()>,
    prepared: PreparedCompositePublicationWithSignal,
    token: &RuntimeWorldCancellationToken,
) {
    port.execute_without_signal(execution, prepared, token);
}
fn main() {}
