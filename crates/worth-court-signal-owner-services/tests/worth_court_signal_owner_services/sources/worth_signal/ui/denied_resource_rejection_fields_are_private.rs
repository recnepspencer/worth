use worth_signal::facade::{
    DeniedResourceRejection, ResourceRejectionDenialClass, ResourceRequestId,
};

fn main() {
    let _ = DeniedResourceRejection {
        request_id: ResourceRequestId::new(0),
        class: ResourceRejectionDenialClass::UnknownOrStaleRequest,
    };
}
