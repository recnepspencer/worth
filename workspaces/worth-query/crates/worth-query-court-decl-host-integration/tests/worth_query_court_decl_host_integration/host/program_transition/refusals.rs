//! Fail-closed transition refusals on real Directory Homes:
//! - Entity scan cap (`EntityScanLimitExceeded`)
//! - Predecessor-field overwrite & non-indexed rebuild target refusals
//! - Intermediate cold-slot & work budgets that exceed backfill-only counts
//!   (proving zero post-intent `TransitionPending` stranding of `v2`)
//! - Oversized default string payload exceeding retained-memory lease before clone
//! - Owner-backed Priority equality-index residency observation (including
//!   negative proof that non-indexed or non-published indexes never report `"Ready"`)

use worth_query_host::facade::{
    application_contribution::with_bootstrap_advancement_from_request,
    program_transition::{
        open_directory_with_transition, worth_query_default_build_id,
        WorthQueryProgramTransitionOpenConfig, WorthQueryProgramTransitionOpenDenial,
    },
    WorthQueryHomeProgramTransitionDenial, WorthQueryHomeProgramTransitionOpenPhase,
};
use worth_query_testkit_host::facade::host::{application_home as fixture, checkpoint_capture};
use worth_relational::facade::indexes::{
    DerivedIndexGenerationResidency, DerivedIndexMaintenanceBudget,
};

use super::journey::{assert_v2_reopens_intact, seed_v2_tasks};
use super::observe::index_residency_to_state;
use super::schema::{task_priority_field, task_v2_value_field};
use super::v2_program::v2_request;
use super::v3_program::{
    v3_request, DaymarkV2ToV3, NonIndexedRebuildTransition, OversizedDefaultTransition,
    OverwritePredecessorFieldTransition,
};

#[test]
fn transition_refuses_entity_scan_cap_with_typed_denial_and_keeps_v2_intact() {
    let location = fixture::location();
    let build = worth_query_default_build_id!();
    let lease = checkpoint_capture::lease(128 * 1024 * 1024);
    let execution = worth_execution::ExecutionRequest::leased(&lease);

    seed_v2_tasks(&location, build, execution, &[("t1", "a"), ("t2", "b")]);

    let scan_refused = with_bootstrap_advancement_from_request(execution, |phase| {
        open_directory_with_transition::<DaymarkV2ToV3>(
            &phase,
            v2_request(location.clone(), build),
            v3_request(location.clone(), build),
            WorthQueryProgramTransitionOpenConfig::default_bounded().with_max_entities_per_kind(1),
        )
    })
    .unwrap()
    .err()
    .expect("max_entities_per_kind=1 on 2 tasks must refuse");
    assert!(matches!(
        scan_refused,
        WorthQueryProgramTransitionOpenDenial::EntityScanLimitExceeded {
            maximum_entities_per_kind: 1,
            ..
        }
    ));
    drop(scan_refused);
    assert_v2_reopens_intact(&location, build, execution, 2);
}

#[test]
fn transition_refuses_predecessor_field_overwrite_and_non_indexed_rebuild_without_stranding_v2() {
    let location = fixture::location();
    let build = worth_query_default_build_id!();
    let lease = checkpoint_capture::lease(128 * 1024 * 1024);
    let execution = worth_execution::ExecutionRequest::leased(&lease);

    seed_v2_tasks(&location, build, execution, &[("t1", "a"), ("t2", "b")]);

    let overwrite_refused = with_bootstrap_advancement_from_request(execution, |phase| {
        open_directory_with_transition::<OverwritePredecessorFieldTransition>(
            &phase,
            v2_request(location.clone(), build),
            v3_request(location.clone(), build),
            WorthQueryProgramTransitionOpenConfig::default_bounded(),
        )
    })
    .unwrap()
    .err()
    .expect("default targeting existing predecessor field must refuse");
    assert!(matches!(
        overwrite_refused,
        WorthQueryProgramTransitionOpenDenial::Host(_)
    ));
    drop(overwrite_refused);

    let non_indexed_refused = with_bootstrap_advancement_from_request(execution, |phase| {
        open_directory_with_transition::<NonIndexedRebuildTransition>(
            &phase,
            v2_request(location.clone(), build),
            v3_request(location.clone(), build),
            WorthQueryProgramTransitionOpenConfig::default_bounded(),
        )
    })
    .unwrap()
    .err()
    .expect("rebuild target with no installed equality index must refuse");
    assert!(matches!(
        non_indexed_refused,
        WorthQueryProgramTransitionOpenDenial::Host(_)
    ));
    drop(non_indexed_refused);

    assert_v2_reopens_intact(&location, build, execution, 2);
}

#[test]
fn transition_cold_slot_and_work_budgets_above_backfill_count_do_not_strand_v2() {
    let location = fixture::location();
    let build = worth_query_default_build_id!();
    let lease = checkpoint_capture::lease(128 * 1024 * 1024);
    let execution = worth_execution::ExecutionRequest::leased(&lease);

    seed_v2_tasks(&location, build, execution, &[("t1", "a"), ("t2", "b")]);

    // Cold-slot limit == 2 (equals the backfilled Task count, which previously
    // slipped past backfill-only preflight and failed post-intent inside Native's
    // `ChangedRecords::cold`, stranding v2 in `TransitionPending`).
    let cold_slot_budget = DerivedIndexMaintenanceBudget {
        maximum_work_units: Some(100_000),
        maximum_cold_record_slots: Some(2),
        maximum_derived_rows: Some(100_000),
    };
    let stopped_slots = with_bootstrap_advancement_from_request(execution, |phase| {
        open_directory_with_transition::<DaymarkV2ToV3>(
            &phase,
            v2_request(location.clone(), build),
            v3_request(location.clone(), build),
            WorthQueryProgramTransitionOpenConfig::default_bounded()
                .with_resources(cold_slot_budget),
        )
    })
    .unwrap()
    .err()
    .expect("cold-slot limit of 2 must fail closed before intent publication");
    let WorthQueryProgramTransitionOpenDenial::BudgetOrCancellationStopped { refused } =
        stopped_slots
    else {
        panic!("expected BudgetOrCancellationStopped, got {stopped_slots:?}");
    };
    assert!(
        matches!(
            refused.phase(),
            WorthQueryHomeProgramTransitionOpenPhase::Transition {
                cause: WorthQueryHomeProgramTransitionDenial::ColdSlotBudgetExceeded(_),
                ..
            }
        ),
        "expected ColdSlotBudgetExceeded before intent publication, got {:?}",
        refused.phase()
    );
    drop(refused);
    assert_v2_reopens_intact(&location, build, execution, 2);

    // Work-unit limit == 5 (above backfill count 2, below full target cold
    // reconstruction work) must also fail closed without stranding v2.
    let work_budget = DerivedIndexMaintenanceBudget {
        maximum_work_units: Some(5),
        maximum_cold_record_slots: Some(100_000),
        maximum_derived_rows: Some(100_000),
    };
    let stopped_work = with_bootstrap_advancement_from_request(execution, |phase| {
        open_directory_with_transition::<DaymarkV2ToV3>(
            &phase,
            v2_request(location.clone(), build),
            v3_request(location.clone(), build),
            WorthQueryProgramTransitionOpenConfig::default_bounded().with_resources(work_budget),
        )
    })
    .unwrap()
    .err()
    .expect("tight work-unit budget must fail closed before intent publication");
    assert!(matches!(
        stopped_work,
        WorthQueryProgramTransitionOpenDenial::BudgetOrCancellationStopped { .. }
    ));
    drop(stopped_work);
    assert_v2_reopens_intact(&location, build, execution, 2);
}

#[test]
fn transition_refuses_oversized_string_default_under_tight_memory_lease_before_cloning() {
    let location = fixture::location();
    let build = worth_query_default_build_id!();
    let setup_lease = checkpoint_capture::lease(128 * 1024 * 1024);
    let setup_exec = worth_execution::ExecutionRequest::leased(&setup_lease);

    seed_v2_tasks(&location, build, setup_exec, &[("t1", "a"), ("t2", "b")]);

    // Lease is ample to open the Home and compile the transition, but smaller
    // than the 24 MiB default string payload in `OversizedDefaultTransition`.
    let tight_lease = checkpoint_capture::lease(16 * 1024 * 1024);
    let tight_exec = worth_execution::ExecutionRequest::leased(&tight_lease);

    let refused = with_bootstrap_advancement_from_request(tight_exec, |phase| {
        open_directory_with_transition::<OversizedDefaultTransition>(
            &phase,
            v2_request(location.clone(), build),
            v3_request(location.clone(), build),
            WorthQueryProgramTransitionOpenConfig::default_bounded(),
        )
    })
    .expect("outer bootstrap succeeds under 16 MiB lease")
    .err()
    .expect("24 MiB default string must exceed 16 MiB retained-memory lease");
    assert!(
        matches!(
            refused,
            WorthQueryProgramTransitionOpenDenial::BudgetOrCancellationStopped { .. }
        ),
        "expected BudgetOrCancellationStopped for oversized default, got {refused:?}"
    );
    drop(refused);
    assert_v2_reopens_intact(&location, build, setup_exec, 2);
}

#[test]
fn priority_index_observer_uses_owner_residency_and_never_reports_unindexed_or_cold_as_ready() {
    let location = fixture::location();
    let build = worth_query_default_build_id!();
    let lease = checkpoint_capture::lease(128 * 1024 * 1024);
    let execution = worth_execution::ExecutionRequest::leased(&lease);

    seed_v2_tasks(&location, build, execution, &[("t1", "a")]);

    let opened = with_bootstrap_advancement_from_request(execution, |phase| {
        Box::new(
            open_directory_with_transition::<DaymarkV2ToV3>(
                &phase,
                v2_request(location.clone(), build),
                v3_request(location, build),
                WorthQueryProgramTransitionOpenConfig::default_bounded(),
            )
            .expect("v2 -> v3 upgrade succeeds"),
        )
    })
    .unwrap();
    let v3_home = (*opened).into_home();

    // Indexed field (`TaskPriority.Priority`) reports Published -> "Ready".
    let priority_residency = v3_home
        .runtime()
        .observe_equality_index_residency(&execution, task_priority_field())
        .expect("residency observation succeeds");
    assert_eq!(
        priority_residency,
        Some(DerivedIndexGenerationResidency::Published)
    );
    assert_eq!(index_residency_to_state(priority_residency), "Ready");

    // Non-indexed field (`TaskV2.V2Value`) reports None -> "None", NEVER "Ready".
    let unindexed_residency = v3_home
        .runtime()
        .observe_equality_index_residency(&execution, task_v2_value_field())
        .expect("residency observation succeeds");
    assert_eq!(unindexed_residency, None);
    assert_eq!(index_residency_to_state(unindexed_residency), "None");

    // Absent / deferred index generations NEVER report "Ready".
    assert_eq!(
        index_residency_to_state(Some(DerivedIndexGenerationResidency::Absent)),
        "Cold"
    );
    assert_eq!(
        index_residency_to_state(Some(DerivedIndexGenerationResidency::DeferredImage)),
        "Cold"
    );

    v3_home.close(&execution).unwrap().reclaim();
}
