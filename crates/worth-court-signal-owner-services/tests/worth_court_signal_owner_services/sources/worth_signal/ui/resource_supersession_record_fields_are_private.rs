use worth_signal::facade::{
    ResourceLifecycleTransition, ResourceRequestHandle, ResourceSupersessionOrdinal,
    ResourceSupersessionRecord,
};

fn forged_handle() -> ResourceRequestHandle {
    loop {}
}

fn forged_transition() -> ResourceLifecycleTransition {
    loop {}
}

fn main() {
    let _ = ResourceSupersessionRecord {
        supersession_ordinal: ResourceSupersessionOrdinal::new(0),
        previous: forged_handle(),
        replacing: forged_handle(),
        lifecycle_transition: forged_transition(),
    };
}
