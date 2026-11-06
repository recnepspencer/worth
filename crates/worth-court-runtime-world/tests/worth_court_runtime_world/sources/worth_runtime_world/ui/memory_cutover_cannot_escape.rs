use worth_runtime_world::facade::*;

fn escape(
    port: &RuntimeWorldPublicationPort<(), (), (), (), ()>,
    execution: worth_execution::ExecutionRequest<'_, '_>,
    prepared: PreparedCompositePublicationWithoutSignal,
    cancellation: &RuntimeWorldCancellationToken,
) {
    let mut escaped = None;
    port.execute_without_signal_with_memory_cutover(execution, prepared, cancellation, |cutover| {
        escaped = Some(cutover);
    });
    escaped.unwrap().perform();
}

fn main() {}
