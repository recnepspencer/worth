use worth_signal::facade::{CancelledResourceRequest, SignalRuntime};

fn cancelled_request() -> CancelledResourceRequest {
    loop {}
}

fn stage_cancelled_request(mut runtime: SignalRuntime<(), (), (), (), ()>) {
    let _ = runtime.stage_admitted_resource_completion(cancelled_request());
}

fn main() {}
