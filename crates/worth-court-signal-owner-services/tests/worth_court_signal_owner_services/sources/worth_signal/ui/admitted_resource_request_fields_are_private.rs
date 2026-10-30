use worth_signal::facade::{
    AdmittedResourceRequest, ResourceAttemptId, ResourceRequestHandle,
};

fn forged_handle() -> ResourceRequestHandle {
    loop {}
}

fn main() {
    let _ = AdmittedResourceRequest {
        handle: forged_handle(),
        attempt: ResourceAttemptId::new(0),
    };
}
