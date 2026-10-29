use worth_signal::facade::adapters::{
    attach_foundational_invalidation_performance_receipt, InvalidationPlanningEstimate,
    SignalInvalidationRealizedCounters,
};

fn main() {
    let predicted = InvalidationPlanningEstimate::default();
    let expected = SignalInvalidationRealizedCounters::default();
    let _ = attach_foundational_invalidation_performance_receipt(predicted, expected);
}
