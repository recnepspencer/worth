use worth_runtime_world::facade::*;
fn illegal_skip(
    port: RuntimeWorldPublicationPort<(), (), (), (), ()>,
    execution: worth_execution::ExecutionRequest<'_, '_>,
    intent: CompositePublicationIntent<WithoutSignal>,
    token: &RuntimeWorldCancellationToken,
) {
    port.execute_without_signal(execution, intent, token);
}
fn main() {}
