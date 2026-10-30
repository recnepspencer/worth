pub use worth_query::facade::domain::WorthQueryProjectionPromotionCounters;

pub fn assert_zero_lifecycle_work(counters: WorthQueryProjectionPromotionCounters) {
    assert_eq!(counters.lifecycle_attempts, 0);
    assert_eq!(counters.fresh_conditional_decisions, 0);
    assert_eq!(counters.planning_attempts, 0);
    assert_eq!(counters.lower_runtime_contacts, 0);
    assert_eq!(counters.managed_resource_registrations, 0);
}
