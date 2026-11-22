//! Preset limits install through the same validation and enforce the same
//! ceilings as the equivalent explicitly assembled limits.
use super::*;
use std::num::NonZeroUsize;
use worth_query_host::facade::{
    application_entry::WorthQueryApplicationMutationOutcome,
    application_installation::WorthQueryBoundedApplicationLimits,
    primary_graph::SignalConditionalEvaluationBudget, runtime::*,
};

const DEFAULTS: RuntimeWorldBudgetInstallation =
    WorthQueryBoundedApplicationLimits::DEFAULT_BUDGETS;

fn serial_policy() -> ExecutionRequestPolicy {
    ExecutionRequestPolicy::new(
        ExecutionPosture::Serial,
        DeterminismContract::CanonicalBitwise,
        ExecutionBudget::new(NonZeroUsize::MIN, None, None),
    )
}

/// The hand-assembled equivalent of a bounded serial preset.
fn explicit(
    budgets: RuntimeWorldBudgetInstallation,
    clock: &WorthQueryAdmissionClock,
) -> Result<WorthQueryApplicationLimits, RuntimeWorldBudgetDenial> {
    WorthQueryProductWorldResources::install(
        budgets,
        WorthQueryProductWorldClock::start(clock.clone()),
        WorthQueryInvalidationResources::new(),
        serial_policy(),
    )
    .map(|world| {
        WorthQueryApplicationLimits::new(
            world,
            WorthQueryApplicationQueryResourceProfile::default(),
            SignalConditionalEvaluationBudget::development(),
        )
    })
}

/// Overrides every group the preset exposes with the given budgets.
fn preset(
    budgets: RuntimeWorldBudgetInstallation,
    clock: &WorthQueryAdmissionClock,
) -> WorthQueryBoundedApplicationLimits {
    WorthQueryApplicationLimits::bounded_serial(clock.clone())
        .with_branches(budgets.branches)
        .with_history(budgets.history)
        .with_observations(budgets.observations)
        .with_publication(budgets.publication)
        .with_recovery(budgets.recovery)
        .with_retention(budgets.retention)
        .with_custody(budgets.custody)
}

#[test]
fn bounded_preset_refuses_each_zero_or_oversized_budget_like_explicit_limits() {
    let clock = WorthQueryAdmissionClock::system();
    assert_eq!(
        WorthQueryApplicationLimits::SERIAL_EXECUTION_POLICY,
        serial_policy()
    );
    assert_eq!(
        WorthQueryApplicationLimits::bounded_serial(clock.clone()).budgets(),
        &DEFAULTS
    );
    assert_eq!(preset(DEFAULTS, &clock).budgets(), &DEFAULTS);
    let mut refused = Vec::new();
    for value in [0, u64::MAX] {
        let mut variants = [DEFAULTS; 10];
        variants[0].branches.live_product_branches = value;
        variants[1].history.retained_composite_commits = value;
        variants[2].history.history_metadata_bytes = value;
        variants[3].observations.active_observations = value;
        variants[4].publication.active_publication_attempts = value;
        variants[5].recovery.retained_product_unpublished_records = value;
        variants[6].recovery.retained_partial_metadata_bytes = value;
        variants[7].retention.unique_exact_component_pins = value;
        variants[8].retention.in_flight_pin_acquisition_reservations = value;
        variants[9].custody.owner_created_component_custody_records = value;
        for budgets in variants {
            let preset = preset(budgets, &clock).install().err();
            let explicit = explicit(budgets, &clock).err();
            assert_eq!(preset, explicit);
            refused.extend(preset);
        }
    }
    let usize_holds_u64 = usize::try_from(u64::MAX).is_ok();
    assert_eq!(refused.len(), if usize_holds_u64 { 10 } else { 20 });
    assert!(preset(DEFAULTS, &clock).install().is_ok());
}

/// Opens a Home under `limits`, then reads, reads while the first result is
/// held, commits and reads again, recording every outcome.
fn journey(limits: WorthQueryApplicationLimits) -> Vec<String> {
    let lease = checkpoint_capture::lease(128 * 1024 * 1024);
    let execution = worth_execution::ExecutionRequest::leased(&lease);
    let home = match open_home(&execution, limits) {
        Ok(home) => home,
        Err(refused) => return vec![format!("open refused: {}", refused.detail())],
    };
    let clock = WorthQueryAdmissionClock::system();
    let scope = scope(&clock);
    let principal = local_principal(&home, &clock, "subject", Duration::from_secs(600))
        .authenticate_local(&scope)
        .unwrap();
    let request = || home.runtime().request(&principal, &scope);
    let held = request()
        .query(fixture::balance_query::AccountRead(
            "account-001".to_owned(),
        ))
        .execute();
    let mut trace = vec![format!(
        "read {:?}",
        held.as_ref()
            .map(|result| result.rows().to_vec())
            .map_err(|_| ())
    )];
    trace.push(format!(
        "held read {:?}",
        read_balance(&home, &principal, &scope)
    ));
    drop(held);
    let committed = {
        let outcome = request()
            .advanced()
            .mutate(fixture::native_mutation::SetAccountBalance {
                account: "account-001".to_owned(),
                balance: 8,
            })
            .without_source()
            .idempotency(&0x5354_4152_u64)
            .execute_in_program(home.runtime(), ExecutionAllocationPolicy::SystemAllocation);
        // The receipt is released before the Home closes.
        match outcome {
            Ok(WorthQueryApplicationMutationOutcome::Committed { result, .. }) => {
                format!("commit {result}")
            }
            other => format!("mutate {other:?}"),
        }
    };
    trace.push(committed);
    trace.push(format!(
        "read {:?}",
        read_balance(&home, &principal, &scope)
    ));
    drop(principal);
    close_home(home, &execution);
    trace
}

#[test]
fn preset_and_explicit_homes_enforce_identical_ceilings() {
    let clock = WorthQueryAdmissionClock::system();
    let ordinary = journey(preset(DEFAULTS, &clock).install().unwrap());
    assert_eq!(
        ordinary,
        [
            "read Ok([7])",
            "held read Ok([7])",
            "commit 8",
            "read Ok([8])"
        ]
    );
    assert_eq!(journey(explicit(DEFAULTS, &clock).unwrap()), ordinary);
    for (observations, refused_step, cause) in [
        (1, 0, "Branch(CapacityExhausted)"),
        (2, 2, "ObservationCapacityExhausted"),
    ] {
        let mut budgets = DEFAULTS;
        budgets.observations.active_observations = observations;
        let through_preset = journey(preset(budgets, &clock).install().unwrap());
        assert!(
            through_preset[refused_step].contains(cause),
            "{observations} observations refuse at step {refused_step}: {through_preset:?}"
        );
        assert_eq!(journey(explicit(budgets, &clock).unwrap()), through_preset);
    }
}
