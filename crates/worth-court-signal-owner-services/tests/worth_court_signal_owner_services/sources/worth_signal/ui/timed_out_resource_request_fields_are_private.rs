use worth_signal::facade::{
    ReadyTemporalWake, ResourceLifecycleTransition, ResourceRequestHandle, ResourceTimeoutOrdinal,
    TimedOutResourceRequest,
};

fn forged_handle() -> ResourceRequestHandle {
    loop {}
}

fn forged_ready_wake() -> ReadyTemporalWake {
    loop {}
}

fn forged_transition() -> ResourceLifecycleTransition {
    loop {}
}

fn main() {
    let _ = TimedOutResourceRequest {
        handle: forged_handle(),
        timeout_ordinal: ResourceTimeoutOrdinal::new(0),
        ready_wake: forged_ready_wake(),
        lifecycle_transition: forged_transition(),
    };
}
