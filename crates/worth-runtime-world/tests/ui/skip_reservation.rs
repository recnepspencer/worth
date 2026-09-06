use worth_runtime_world::facade::*;
fn illegal_skip(port: RuntimeWorldPublicationPort<(),(),(),(),()>,
    intent: CompositePublicationIntent<WithoutSignal>, token: &RuntimeWorldCancellationToken) {
    port.execute_without_signal(intent,token);
}
fn main() {}
