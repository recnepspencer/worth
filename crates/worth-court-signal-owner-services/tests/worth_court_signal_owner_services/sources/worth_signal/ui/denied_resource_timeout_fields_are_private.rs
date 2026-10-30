use worth_signal::facade::{
    DeniedResourceTimeout, ResourceRequestId, ResourceTimeoutDenialClass,
};

fn main() {
    let _ = DeniedResourceTimeout {
        request_id: ResourceRequestId::new(0),
        class: ResourceTimeoutDenialClass::WakeMismatch,
    };
}
