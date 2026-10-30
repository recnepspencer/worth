use worth_signal::facade::{
    CompletionDenialClass, ResourceRetainedDeniedCompletionAvailability,
    ResourceRetainedDeniedCompletionAvailabilityClass,
};

fn fake<T>() -> T {
    panic!("compile-fail fixture")
}

fn main() {
    let _availability = ResourceRetainedDeniedCompletionAvailability {
        denial_id: fake(),
        request_id: fake(),
        node: fake(),
        denial_class: CompletionDenialClass::UnknownRequest,
        class: ResourceRetainedDeniedCompletionAvailabilityClass::PrunedByRetainedDeniedCompletionLimit,
    };
}
