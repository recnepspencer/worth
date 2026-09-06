#[path = "runtime_world_certification/basis_history.rs"]
mod basis_history;
#[path = "runtime_world_certification/bridge.rs"]
mod bridge;
#[path = "runtime_world_certification/court/mod.rs"]
mod court;
#[path = "runtime_world_certification/reference.rs"]
mod reference;
#[path = "runtime_world_certification/retention.rs"]
mod retention;

use worth_relational::facade::mvcc::RelationalTransactionIntent;
use worth_runtime_world::facade::{
    CompositeComponentIntent, CompositePublicationIntent, ProductBranchCreationIntent,
    ProductBranchCreationPlans, ProductBranchName, RelationalBranchCreationPlan,
    RuntimeWorldBranchBudgetInstallation, RuntimeWorldBudgetDenial, RuntimeWorldBudgetInstallation,
    RuntimeWorldBudgets, RuntimeWorldCancellationSource, RuntimeWorldClock,
    RuntimeWorldClockSource, RuntimeWorldCustodyBudgetInstallation,
    RuntimeWorldHistoryBudgetInstallation, RuntimeWorldInstant,
    RuntimeWorldObservationBudgetInstallation, RuntimeWorldPublicationBudgetInstallation,
    RuntimeWorldRecoveryBudgetInstallation, RuntimeWorldRetentionBudgetInstallation,
    SignalBranchCreationPlan,
};

#[test]
fn installed_budgets_are_nonzero_and_cover_every_runtime_world_population() {
    let budgets = RuntimeWorldBudgets::install(RuntimeWorldBudgetInstallation {
        branches: RuntimeWorldBranchBudgetInstallation {
            live_product_branches: 1,
        },
        history: RuntimeWorldHistoryBudgetInstallation {
            retained_composite_commits: 2,
            history_metadata_bytes: 3,
        },
        observations: RuntimeWorldObservationBudgetInstallation {
            active_observations: 4,
        },
        publication: RuntimeWorldPublicationBudgetInstallation {
            active_publication_attempts: 5,
        },
        recovery: RuntimeWorldRecoveryBudgetInstallation {
            retained_product_unpublished_records: 6,
            retained_partial_metadata_bytes: 7,
        },
        retention: RuntimeWorldRetentionBudgetInstallation {
            unique_exact_component_pins: 8,
            in_flight_pin_acquisition_reservations: 9,
        },
        custody: RuntimeWorldCustodyBudgetInstallation {
            owner_created_component_custody_records: 10,
        },
    })
    .expect("all installed limits are nonzero");

    assert_eq!(budgets.live_product_branches().get(), 1);
    assert_eq!(budgets.retained_composite_commits().get(), 2);
    assert_eq!(budgets.history_metadata_bytes().get(), 3);
    assert_eq!(budgets.active_observations().get(), 4);
    assert_eq!(budgets.active_publication_attempts().get(), 5);
    assert_eq!(budgets.retained_product_unpublished_records().get(), 6);
    assert_eq!(budgets.retained_partial_metadata_bytes().get(), 7);
    assert_eq!(budgets.unique_exact_component_pins().get(), 8);
    assert_eq!(budgets.in_flight_pin_acquisition_reservations().get(), 9);
    assert_eq!(budgets.owner_created_component_custody_records().get(), 10);

    let denial = RuntimeWorldBudgets::install(RuntimeWorldBudgetInstallation {
        branches: RuntimeWorldBranchBudgetInstallation {
            live_product_branches: 0,
        },
        history: RuntimeWorldHistoryBudgetInstallation {
            retained_composite_commits: 1,
            history_metadata_bytes: 1,
        },
        observations: RuntimeWorldObservationBudgetInstallation {
            active_observations: 1,
        },
        publication: RuntimeWorldPublicationBudgetInstallation {
            active_publication_attempts: 1,
        },
        recovery: RuntimeWorldRecoveryBudgetInstallation {
            retained_product_unpublished_records: 1,
            retained_partial_metadata_bytes: 1,
        },
        retention: RuntimeWorldRetentionBudgetInstallation {
            unique_exact_component_pins: 1,
            in_flight_pin_acquisition_reservations: 1,
        },
        custody: RuntimeWorldCustodyBudgetInstallation {
            owner_created_component_custody_records: 1,
        },
    })
    .expect_err("zero capacity is not an installed bound");
    assert!(matches!(denial, RuntimeWorldBudgetDenial::ZeroLimit { .. }));
}

/// Creation is a two-by-two matrix of independent per-owner plans, and every
/// fork carries the owner-issued destination it names.
#[test]
fn branch_name_and_per_owner_creation_plans_are_explicit() {
    assert!(ProductBranchName::try_new("main").is_ok());
    assert!(ProductBranchName::try_new("  ").is_err());

    let signal_target = worth_signal::facade::branch::validate_signal_branch_name("signal-child")
        .expect("Signal branch name validates");
    let plans = ProductBranchCreationPlans::new(
        RelationalBranchCreationPlan::ForkExact {
            target: worth_relational::facade::history::BranchId("relational-child".to_owned()),
        },
        SignalBranchCreationPlan::ReuseExact,
    );
    assert!(plans.requires_relational_owner_effect());
    assert!(!plans.requires_signal_owner_effect());
    assert!(!plans.is_exact_reuse());
    assert!(plans.relational().fork_target().is_some());
    assert!(plans.signal().fork_target().is_none());
    assert!(ProductBranchCreationPlans::new(
        RelationalBranchCreationPlan::ReuseExact,
        SignalBranchCreationPlan::ForkExact {
            target: signal_target,
        },
    )
    .requires_signal_owner_effect());

    let bootstrap = ProductBranchCreationIntent::named("root").expect("valid root name");
    assert!(bootstrap.plans().is_none());
    let from_source =
        ProductBranchCreationIntent::from_source("child", plans).expect("valid child branch name");
    assert!(from_source.plans().is_some());
}

#[test]
fn component_intent_carries_owner_meaning_without_ambient_currentness() {
    let relational =
        CompositeComponentIntent::relational_only(RelationalTransactionIntent::ordinary());
    assert!(relational.changes_relational());
    assert!(!relational.changes_signal());
    assert!(relational.relational_change().is_some());

    let signal = CompositeComponentIntent::signal_only();
    assert!(!signal.changes_relational());
    assert!(signal.changes_signal());
    assert!(signal.relational_change().is_none());
}

/// A publication declares at construction whether it contacts the Signal
/// owner. The two stages are separate types, so the decision is visible to the
/// compiler rather than re-read from a runtime posture.
#[test]
fn publication_stage_is_declared_at_intent_construction() {
    let without_signal =
        CompositePublicationIntent::without_signal(RelationalTransactionIntent::ordinary());
    assert!(without_signal.component_intent().changes_relational());
    assert!(!without_signal.component_intent().changes_signal());

    let signal_only = CompositePublicationIntent::with_signal(None);
    assert!(!signal_only.component_intent().changes_relational());
    assert!(signal_only.component_intent().changes_signal());

    let both =
        CompositePublicationIntent::with_signal(Some(RelationalTransactionIntent::ordinary()));
    assert!(both.component_intent().changes_relational());
    assert!(both.component_intent().changes_signal());
}

#[test]
fn cancellation_has_a_named_pre_effect_source_and_token() {
    let source = RuntimeWorldCancellationSource::new();
    let token = source.token();
    assert!(!token.is_cancelled());
    source.cancel();
    assert!(token.is_cancelled());
}

struct FixedClock;

impl RuntimeWorldClockSource for FixedClock {
    fn now(&self) -> RuntimeWorldInstant {
        RuntimeWorldInstant::from_ticks(42)
    }
}

#[test]
fn clock_is_explicit_and_only_reports_deadline_time() {
    let clock = RuntimeWorldClock::from_source(FixedClock);
    assert_eq!(clock.now().ticks(), 42);
}

#[test]
fn compile_failures_protect_the_public_contract() {
    let tests = trybuild::TestCases::new();
    tests.compile_fail("tests/ui/*.rs");
    tests.pass("tests/pass/*.rs");
}
