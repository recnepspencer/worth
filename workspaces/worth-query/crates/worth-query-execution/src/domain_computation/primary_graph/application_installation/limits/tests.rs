//! Overrides replace exactly their own limit; presets keep their documented policy.
use super::*;
use std::num::NonZeroUsize;
use worth_query_admission::facade::authenticated_principal::WorthQueryAdmissionClock;

#[test]
fn query_profile_and_signal_budget_overrides_replace_only_their_own_limit() {
    let preset = WorthQueryApplicationLimits::ordinary_serial(WorthQueryAdmissionClock::system());
    assert_eq!(
        preset.queries,
        WorthQueryApplicationQueryResourceProfile::default()
    );
    assert_eq!(
        preset.conditionals,
        SignalConditionalEvaluationBudget::development()
    );
    assert_eq!(
        preset.world.execution_policy(),
        WorthQueryApplicationLimits::SERIAL_EXECUTION_POLICY
    );
    let profile = WorthQueryApplicationQueryResourceProfile::new(NonZeroUsize::new(3).unwrap());
    let budget = SignalConditionalEvaluationBudget {
        maximum_retained_slots: Some(7),
        maximum_retained_bytes: Some(4096),
        maximum_attempt_visits: Some(11),
    };
    let overridden = preset
        .with_query_profile(profile)
        .with_signal_budget(budget);
    assert_eq!(overridden.queries, profile);
    assert_eq!(overridden.conditionals, budget);
    assert_eq!(
        overridden.world.execution_policy(),
        WorthQueryApplicationLimits::SERIAL_EXECUTION_POLICY
    );
    assert_eq!(overridden.maximum_publication_records, None);
}
