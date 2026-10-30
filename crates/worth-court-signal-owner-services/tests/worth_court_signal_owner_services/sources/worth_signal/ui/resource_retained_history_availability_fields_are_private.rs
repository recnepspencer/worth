use worth_signal::facade::{
    ResourceRetainedHistoryAvailability, ResourceRetainedHistoryAvailabilityClass,
    ResourceRetentionDecisionClass,
};

fn fake<T>() -> T {
    panic!("compile-fail fixture")
}

fn main() {
    let _availability = ResourceRetainedHistoryAvailability {
        handle: fake(),
        attempt: fake(),
        node: fake(),
        lifecycle: fake(),
        class: ResourceRetainedHistoryAvailabilityClass::TerminalSummaryOnly,
        retention_descriptor_id: fake(),
        retention_decision_class: ResourceRetentionDecisionClass::TerminalSummariesOnly,
        retention_decision_digest: fake(),
    };
}
