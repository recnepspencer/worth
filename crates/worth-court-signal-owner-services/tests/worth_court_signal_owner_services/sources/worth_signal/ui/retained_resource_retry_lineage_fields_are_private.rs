use worth_signal::facade::{ResourceRetryReason, RetainedResourceRetryLineage};

fn fake<T>() -> T {
    panic!("compile-fail fixture")
}

fn main() {
    let _retained = RetainedResourceRetryLineage {
        previous: fake(),
        retry_ordinal: fake(),
        node: fake(),
        reason: ResourceRetryReason::TimedOut,
        next_attempt: fake(),
        scheduled_delay: fake(),
        policy_decision_digest: fake(),
    };
}
