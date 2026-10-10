use worth_execution::ExecutionRequest;
use worth_runtime_world::facade::*;
fn illegal_skip(
    execution: ExecutionRequest<'_, '_>,
    port: RuntimeWorldPublicationPort<(), (), (), (), ()>,
    intent: CompositePublicationIntent<WithoutSignal>,
    token: &RuntimeWorldCancellationToken,
) {
    port.execute_without_signal(execution, intent, token);
}
fn main() {}
