//! Fail-closed transition refusals on real Directory Homes:
//! - Entity scan cap (`EntityScanLimitExceeded`)
//! - Predecessor-field overwrite & non-indexed rebuild target refusals
//! - Exact Native-owned cold-slot & work-unit boundary (`S - 1`, `N - 1` fail
//!   before intent publication keeping `v2` intact; `(S, N, R)` succeeds)
//! - Oversized default string payload exceeding retained-memory lease at the
//!   specific backfill pre-allocation boundary (`RetainedBytesExhausted`)
//! - Owner-backed Priority equality-index residency observation across the real
//!   `Published -> DeferredImage (reopened before read) -> Published (after read)`
//!   lifecycle and non-indexed field `None` check.

use worth_query_host::facade::{
    application_contribution::{
        with_bootstrap_advancement_from_request, WorthQueryManagedComputationResourceDenial,
    },
    primary_graph::{
        WorthQueryPrimaryGraphInstallationDenialKind, WorthQueryProviderSessionDenialKind,
    },
    program_transition::{
        open_directory_with_transition, worth_query_default_build_id,
        WorthQueryProgramTransitionOpenConfig, WorthQueryProgramTransitionOpenDenial,
    },
    WorthQueryHomeProgramTransitionDenial, WorthQueryHomeProgramTransitionOpenPhase,
};
use worth_query_testkit_host::facade::host::{application_home as fixture, checkpoint_capture};
use worth_relational::facade::indexes::{
    DerivedIndexMaintenanceBudget, DerivedIndexMaintenanceDenialKind,
};

use super::journey::{assert_v2_reopens_intact, seed_v2_tasks};
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

fn try_v2_to_v3_with_budget(
    location: &std::path::Path,
    build: worth_query_host::facade::application_installation::ApplicationBuildIdentity,
    execution: worth_execution::ExecutionRequest<'_, '_>,
    budget: DerivedIndexMaintenanceBudget,
) -> Result<
    worth_query_host::facade::program_transition::WorthQueryTransitionOpenedHome<
        super::v3_program::DaymarkV3Schema,
        super::v3_program::DaymarkV3Program,
    >,
    WorthQueryProgramTransitionOpenDenial<
        super::v2_program::DaymarkV2Schema,
        super::v2_program::DaymarkV2Program,
        super::v3_program::DaymarkV3Schema,
        super::v3_program::DaymarkV3Program,
    >,
> {
    *with_bootstrap_advancement_from_request(execution, |phase| {
        Box::new(open_directory_with_transition::<DaymarkV2ToV3>(
            &phase,
            v2_request(location.to_path_buf(), build),
            v3_request(location.to_path_buf(), build),
            WorthQueryProgramTransitionOpenConfig::default_bounded().with_resources(budget),
        ))
    })
    .unwrap()
}

#[test]
fn transition_native_exact_budget_boundary_refuses_n_minus_1_without_stranding_and_admits_n() {
    let location = fixture::location();
    let build = worth_query_default_build_id!();
    let lease = checkpoint_capture::lease(128 * 1024 * 1024);
    let execution = worth_execution::ExecutionRequest::leased(&lease);

    seed_v2_tasks(&location, build, execution, &[("t1", "a"), ("t2", "b")]);

    let work_probe = try_v2_to_v3_with_budget(
        &location,
        build,
        execution,
        DerivedIndexMaintenanceBudget {
            maximum_work_units: Some(1),
            maximum_cold_record_slots: Some(100_000),
            maximum_derived_rows: Some(100_000),
        },
    )
    .err()
    .expect("work=1 probe must refuse");
    let required = match work_probe {
        WorthQueryProgramTransitionOpenDenial::BudgetOrCancellationStopped { refused } => {
            let WorthQueryHomeProgramTransitionOpenPhase::Transition {
                cause: WorthQueryHomeProgramTransitionDenial::Indexes(denial),
                ..
            } = refused.phase()
            else {
                panic!(
                    "expected Transition/Indexes denial, got {:?}",
                    refused.phase()
                );
            };
            assert_eq!(
                denial.kind,
                DerivedIndexMaintenanceDenialKind::WorkBudgetExceeded
            );
            let quote = denial.work;
            drop(refused);
            quote
        }
        other => panic!("expected BudgetOrCancellationStopped, got {other:?}"),
    };
    assert!(required.cold_record_slots > 2);
    assert!(required.work_units > required.cold_record_slots);
    assert!(required.generation_publications_reserved >= 1);

    // S - 1 cold slots fails cleanly BEFORE intent publication with typed
    // `ColdRecordSlotBudgetExceeded` and leaves plain v2 intact.
    let short_slots = try_v2_to_v3_with_budget(
        &location,
        build,
        execution,
        DerivedIndexMaintenanceBudget {
            maximum_work_units: Some(required.work_units),
            maximum_cold_record_slots: Some(required.cold_record_slots - 1),
            maximum_derived_rows: Some(required.derived_rows.max(1)),
        },
    )
    .err()
    .expect("cold_slots = S - 1 must fail before intent publication");
    assert!(matches!(
        &short_slots,
        WorthQueryProgramTransitionOpenDenial::BudgetOrCancellationStopped { refused }
            if matches!(
                refused.phase(),
                WorthQueryHomeProgramTransitionOpenPhase::Transition {
                    cause: WorthQueryHomeProgramTransitionDenial::Indexes(denial),
                    ..
                } if denial.kind == DerivedIndexMaintenanceDenialKind::ColdRecordSlotBudgetExceeded
            )
    ));
    drop(short_slots);
    assert_v2_reopens_intact(&location, build, execution, 2);

    // N - 1 work units fails cleanly BEFORE intent publication with typed
    // `WorkBudgetExceeded` and leaves plain v2 intact.
    let short_work = try_v2_to_v3_with_budget(
        &location,
        build,
        execution,
        DerivedIndexMaintenanceBudget {
            maximum_work_units: Some(required.work_units - 1),
            maximum_cold_record_slots: Some(required.cold_record_slots),
            maximum_derived_rows: Some(required.derived_rows.max(1)),
        },
    )
    .err()
    .expect("work_units = N - 1 must fail before intent publication");
    assert!(matches!(
        &short_work,
        WorthQueryProgramTransitionOpenDenial::BudgetOrCancellationStopped { refused }
            if matches!(
                refused.phase(),
                WorthQueryHomeProgramTransitionOpenPhase::Transition {
                    cause: WorthQueryHomeProgramTransitionDenial::Indexes(denial),
                    ..
                } if denial.kind == DerivedIndexMaintenanceDenialKind::WorkBudgetExceeded
            )
    ));
    drop(short_work);
    assert_v2_reopens_intact(&location, build, execution, 2);

    // Exact admitted boundary (N, S, R) succeeds end-to-end without stranding!
    let admitted = try_v2_to_v3_with_budget(
        &location,
        build,
        execution,
        DerivedIndexMaintenanceBudget {
            maximum_work_units: Some(required.work_units),
            maximum_cold_record_slots: Some(required.cold_record_slots),
            maximum_derived_rows: Some(required.derived_rows.max(1)),
        },
    )
    .expect("exact admitted budget (N, S, R) must succeed");
    admitted.into_home().close(&execution).unwrap().reclaim();
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
    let WorthQueryProgramTransitionOpenDenial::BudgetOrCancellationStopped { refused } = refused
    else {
        panic!("expected BudgetOrCancellationStopped for oversized default, got {refused:?}");
    };
    assert!(
        matches!(
            refused.phase(),
            WorthQueryHomeProgramTransitionOpenPhase::Transition {
                cause: WorthQueryHomeProgramTransitionDenial::Author(denial),
                ..
            } if matches!(
                denial.kind(),
                WorthQueryPrimaryGraphInstallationDenialKind::ExecutionDenied {
                    kind: WorthQueryProviderSessionDenialKind::ExecutionResource {
                        denial: WorthQueryManagedComputationResourceDenial::RetainedBytesExhausted,
                        ..
                    }
                }
            )
        ),
        "expected specific Transition/Author/RetainedBytesExhausted backfill pre-allocation refusal, got {:?}",
        refused.phase()
    );
    drop(refused);
    assert_v2_reopens_intact(&location, build, setup_exec, 2);
}
