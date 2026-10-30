use worth_signal::facade::{
    ResourceAttemptId, ResourceRequestHandle, ValidatedCompletionEnvelope,
};

fn forged_handle() -> ResourceRequestHandle {
    loop {}
}

fn main() {
    let _ = ValidatedCompletionEnvelope {
        handle: forged_handle(),
        attempt: ResourceAttemptId::new(0),
        payload_byte_len: 0,
    };
}
