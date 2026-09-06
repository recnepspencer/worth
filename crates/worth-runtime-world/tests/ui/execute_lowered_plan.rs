use worth_runtime_world::facade::*;
fn illegal_stage(port: RuntimeWorldPublicationPort<(),(),(),(),()>,
    prepared: PreparedCompositePublicationWithSignal, token:&RuntimeWorldCancellationToken) {
    port.execute_without_signal(prepared,token);
}
fn main() {}
