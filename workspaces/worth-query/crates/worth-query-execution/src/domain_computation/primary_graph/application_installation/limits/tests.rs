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

/// A finite caller policy distinct from the preset's in every ceiling.
fn bounded_policy() -> worth_foundational::ExecutionRequestPolicy {
    worth_foundational::ExecutionRequestPolicy::new(
        worth_foundational::ExecutionPosture::Serial,
        worth_foundational::DeterminismContract::CanonicalBitwise,
        worth_foundational::ExecutionBudget::new(
            NonZeroUsize::MIN,
            Some(128 * 1024 * 1024),
            Some(128_000_000),
        ),
    )
}

fn manual(
    budgets: worth_runtime_world::facade::RuntimeWorldBudgetInstallation,
    queries: WorthQueryApplicationQueryResourceProfile,
    conditionals: SignalConditionalEvaluationBudget,
) -> Result<WorthQueryApplicationLimits, worth_runtime_world::facade::RuntimeWorldBudgetDenial> {
    use crate::domain_computation::execution_runtime::{
        product_world::WorthQueryProductWorldClock, WorthQueryInvalidationResources,
    };
    WorthQueryProductWorldResources::install(
        budgets,
        WorthQueryProductWorldClock::start(WorthQueryAdmissionClock::system()),
        WorthQueryInvalidationResources::new(),
        bounded_policy(),
    )
    .map(|world| WorthQueryApplicationLimits::new(world, queries, conditionals))
}

#[test]
fn bounded_preset_overrides_install_exactly_what_manual_assembly_installs() {
    let (queries, conditionals) = (
        WorthQueryApplicationQueryResourceProfile::new(NonZeroUsize::new(5).unwrap()),
        SignalConditionalEvaluationBudget {
            maximum_retained_slots: Some(9),
            maximum_retained_bytes: Some(2048),
            maximum_attempt_visits: Some(13),
        },
    );
    let preset = WorthQueryApplicationLimits::bounded_serial(WorthQueryAdmissionClock::system())
        .with_execution_policy(bounded_policy())
        .with_query_profile(queries)
        .with_signal_budget(conditionals);
    assert_eq!(
        *preset.budgets(),
        WorthQueryBoundedApplicationLimits::DEFAULT_BUDGETS
    );
    let (preset, manual) = (
        preset.install().expect("the default budgets install"),
        manual(
            WorthQueryBoundedApplicationLimits::DEFAULT_BUDGETS,
            queries,
            conditionals,
        )
        .expect("the default budgets install"),
    );
    assert_eq!(preset.world.execution_policy(), bounded_policy());
    assert_eq!(
        preset.world.execution_policy(),
        manual.world.execution_policy()
    );
    assert_eq!(
        (preset.queries, preset.conditionals),
        (manual.queries, manual.conditionals)
    );
    assert_eq!(preset.output_demands, manual.output_demands);
    assert_eq!(preset.profile, manual.profile);
    assert_eq!(preset.maximum_publication_records, None);
}

#[test]
fn bounded_preset_overrides_refuse_an_invalid_budget_exactly_as_manual_assembly() {
    let mut zero = WorthQueryBoundedApplicationLimits::DEFAULT_BUDGETS;
    zero.observations.active_observations = 0;
    let refused = WorthQueryApplicationLimits::bounded_serial(WorthQueryAdmissionClock::system())
        .with_observations(zero.observations)
        .with_execution_policy(bounded_policy())
        .with_query_profile(WorthQueryApplicationQueryResourceProfile::default())
        .with_signal_budget(SignalConditionalEvaluationBudget::development())
        .install()
        .err()
        .expect("a zero observation ceiling refuses");
    let manual = manual(
        zero,
        WorthQueryApplicationQueryResourceProfile::default(),
        SignalConditionalEvaluationBudget::development(),
    )
    .err()
    .expect("a zero observation ceiling refuses");
    assert_eq!(refused, manual);
    assert!(matches!(
        refused,
        worth_runtime_world::facade::RuntimeWorldBudgetDenial::ZeroLimit { .. }
    ));
}
