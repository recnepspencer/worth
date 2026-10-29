use worth_signal::facade::{
    DeniedResourceCancellation, ResourceCancellationDenialClass, ResourceRequestId,
};

fn main() {
    let _ = DeniedResourceCancellation {
        request_id: ResourceRequestId::new(0),
        class: ResourceCancellationDenialClass::UnknownOrStaleRequest,
    };
}
