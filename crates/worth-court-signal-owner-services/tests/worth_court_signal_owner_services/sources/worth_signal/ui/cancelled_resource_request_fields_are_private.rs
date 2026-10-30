use worth_signal::facade::{
    CancelledResourceRequest, ResourceCancellationOrdinal, ResourceCancellationReason,
    ResourceCancellationGraceWindow, ResourceHostCancellationAdvisory,
    ResourceLifecycleTransition, ResourcePolicyDigest, ResourceRequestHandle,
};

fn forged_handle() -> ResourceRequestHandle {
    loop {}
}

fn forged_transition() -> ResourceLifecycleTransition {
    loop {}
}

fn forged_host_advisory() -> ResourceHostCancellationAdvisory {
    loop {}
}

fn forged_grace_window() -> ResourceCancellationGraceWindow {
    loop {}
}

fn forged_digest() -> ResourcePolicyDigest {
    loop {}
}

fn main() {
    let _ = CancelledResourceRequest {
        handle: forged_handle(),
        cancellation_ordinal: ResourceCancellationOrdinal::new(0),
        reason: ResourceCancellationReason::HostRequested,
        policy_decision_digest: forged_digest(),
        host_advisory: Some(forged_host_advisory()),
        grace_window: Some(forged_grace_window()),
        lifecycle_transition: forged_transition(),
    };
}
