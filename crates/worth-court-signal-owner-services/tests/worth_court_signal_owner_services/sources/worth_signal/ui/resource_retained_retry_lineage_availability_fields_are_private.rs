use worth_signal::facade::{
    ResourceRetainedRetryLineageAvailability, ResourceRetainedRetryLineageAvailabilityClass,
    ResourceRetryReason,
};

fn fake<T>() -> T {
    panic!("compile-fail fixture")
}

fn main() {
    let _availability = ResourceRetainedRetryLineageAvailability {
        previous: fake(),
        retry_ordinal: fake(),
        node: fake(),
        reason: ResourceRetryReason::TimedOut,
        next_attempt: fake(),
        scheduled_delay: fake(),
        class: ResourceRetainedRetryLineageAvailabilityClass::PrunedByRetainedRetryLineageLimit,
        policy_decision_digest: fake(),
    };
}
