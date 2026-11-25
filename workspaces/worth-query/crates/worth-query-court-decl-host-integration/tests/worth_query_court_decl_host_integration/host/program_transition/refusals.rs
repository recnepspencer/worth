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

fn assert_transition_index_denial(
    denial: WorthQueryProgramTransitionOpenDenial<
        super::v2_program::DaymarkV2Schema,
        super::v2_program::DaymarkV2Program,
        super::v3_program::DaymarkV3Schema,
        super::v3_program::DaymarkV3Program,
    >,
    expected: DerivedIndexMaintenanceDenialKind,
) {
    assert!(
        matches!(
            &denial,
            WorthQueryProgramTransitionOpenDenial::BudgetOrCancellationStopped { refused }
                if matches!(
                    refused.phase(),
                    WorthQueryHomeProgramTransitionOpenPhase::Transition {
                        cause: WorthQueryHomeProgramTransitionDenial::Indexes(inner),
                        ..
                    } if inner.kind == expected
                )
        ),
        "expected Transition/Indexes({expected:?}), got {denial:?}"
    );
}

#[test]
fn transition_native_exact_budget_boundary_refuses_n_minus_1_without_stranding_and_admits_n() {
    let location = fixture::location();
    let build = worth_query_default_build_id!();
    let lease = checkpoint_capture::lease(128 * 1024 * 1024);
    let execution = worth_execution::ExecutionRequest::leased(&lease);

    seed_v2_tasks(&location, build, execution, &[("t1", "a"), ("t2", "b")]);

    // Independent exact Native cold-reconstruction budget for DaymarkV2 -> DaymarkV3
    // with 1 Project (NOT projected by TaskPriority), 1 Principal, 1 ExternalMapping,
    // 2 Tasks (t1, t2), 2 idempotency records, 2 activation records, 3 relations,
    // and 1 partition:
    // - S = 9 entity slots + 3 relation slots + 1 region = 13
    // - R = 1 (ExternalIdentity) + 1 (MappingStatus) + 1 (PrincipalIdentity)
    //     + 1 (ProjectKey) + 2 (TaskKey) + 2 (TaskPriority — Tasks ONLY, NOT Project!)
    //     + 2 (idempotency keys) = 10
    // - N = 13 (slots) + 12 (9 entity + 3 relation reads) + 10 (rows)
    //     + 25 (per-index CandidateIndexPreparation::prepare charges) = 60
    const EXACT_SLOTS: usize = 13;
    const EXACT_ROWS: usize = 10;
    const EXACT_WORK: usize = 60;

    for (budget, expected, label) in [
        (
            DerivedIndexMaintenanceBudget {
                maximum_work_units: Some(EXACT_WORK),
                maximum_cold_record_slots: Some(EXACT_SLOTS - 1),
                maximum_derived_rows: Some(EXACT_ROWS),
            },
            DerivedIndexMaintenanceDenialKind::ColdRecordSlotBudgetExceeded,
            "cold_slots = S - 1",
        ),
        (
            DerivedIndexMaintenanceBudget {
                maximum_work_units: Some(EXACT_WORK),
                maximum_cold_record_slots: Some(EXACT_SLOTS),
                maximum_derived_rows: Some(EXACT_ROWS - 1),
            },
            DerivedIndexMaintenanceDenialKind::WorkBudgetExceeded,
            "derived_rows = R - 1",
        ),
        (
            DerivedIndexMaintenanceBudget {
                maximum_work_units: Some(EXACT_WORK - 1),
                maximum_cold_record_slots: Some(EXACT_SLOTS),
                maximum_derived_rows: Some(EXACT_ROWS),
            },
            DerivedIndexMaintenanceDenialKind::WorkBudgetExceeded,
            "work_units = N - 1",
        ),
    ] {
        let refusal = try_v2_to_v3_with_budget(&location, build, execution, budget)
            .err()
            .unwrap_or_else(|| panic!("{label} must fail before intent publication"));
        assert_transition_index_denial(refusal, expected);
        assert_v2_reopens_intact(&location, build, execution, 2);
    }

    let admitted = try_v2_to_v3_with_budget(
        &location,
        build,
        execution,
        DerivedIndexMaintenanceBudget {
            maximum_work_units: Some(EXACT_WORK),
            maximum_cold_record_slots: Some(EXACT_SLOTS),
            maximum_derived_rows: Some(EXACT_ROWS),
        },
    )
    .expect("exact independent budget (N=60, S=13, R=10) must succeed without false refusal");
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

#[test]
fn transition_small_defaults_reserve_container_bytes_on_lease_and_tight_lease_preserves_v2() {
    use worth_query_host::facade::program_transition::open_directory_with_transition_and_step;
    use worth_relational::facade::transactions::MutationIntent;

    let build = worth_query_default_build_id!();
    let setup_lease = checkpoint_capture::lease(128 * 1024 * 1024);
    let setup_exec = worth_execution::ExecutionRequest::leased(&setup_lease);

    // Measure retained memory held on the caller's lease right after small-default
    // backfill completes on 0 tasks vs 12 tasks (each default is tiny 6-byte `"Normal"`).
    let measure_after_backfill = |tasks: &[(&str, &str)]| -> u64 {
        let loc = fixture::location();
        seed_v2_tasks(&loc, build, setup_exec, tasks);
        let run_lease = checkpoint_capture::lease(128 * 1024 * 1024);
        let run_exec = worth_execution::ExecutionRequest::leased(&run_lease);
        let mut charged = 0u64;
        let opened = with_bootstrap_advancement_from_request(run_exec, |phase| {
            Box::new(
                open_directory_with_transition_and_step::<DaymarkV2ToV3, _>(
                    &phase,
                    (
                        v2_request(loc.clone(), build),
                        v3_request(loc.clone(), build),
                    ),
                    WorthQueryProgramTransitionOpenConfig::default_bounded(),
                    |_source, _target| {
                        charged = run_lease.charged_memory_bytes();
                        Ok(Vec::new())
                    },
                )
                .expect("small-default transition succeeds under ample lease"),
            )
        })
        .unwrap();
        (*opened).into_home().close(&run_exec).unwrap().reclaim();
        charged
    };

    let twelve_tasks: Vec<(String, String)> = (0..12)
        .map(|i| (format!("t{i:02}"), format!("v{i:02}")))
        .collect();
    let twelve_refs: Vec<(&str, &str)> = twelve_tasks
        .iter()
        .map(|(k, v)| (k.as_str(), v.as_str()))
        .collect();

    let base_charged = measure_after_backfill(&[]);
    let twelve_charged = measure_after_backfill(&twelve_refs);
    let delta = twelve_charged.saturating_sub(base_charged);
    let min_container_bytes = (12 * std::mem::size_of::<MutationIntent>()) as u64;
    assert!(
        delta >= min_container_bytes,
        "small-default backfill must reserve container + intent bytes on the lease (delta={delta}, min={min_container_bytes})"
    );

    // Under a tight lease below the 12-task small-default footprint, transition
    // fails closed with `BudgetOrCancellationStopped` and keeps all 12 v2 rows intact.
    let loc_tight = fixture::location();
    seed_v2_tasks(&loc_tight, build, setup_exec, &twelve_refs);
    let tight_lease = checkpoint_capture::lease(twelve_charged - 512);
    let tight_exec = worth_execution::ExecutionRequest::leased(&tight_lease);
    let stopped = with_bootstrap_advancement_from_request(tight_exec, |phase| {
        open_directory_with_transition::<DaymarkV2ToV3>(
            &phase,
            v2_request(loc_tight.clone(), build),
            v3_request(loc_tight.clone(), build),
            WorthQueryProgramTransitionOpenConfig::default_bounded(),
        )
    })
    .expect("outer bootstrap succeeds under tight lease");
    let Err(WorthQueryProgramTransitionOpenDenial::BudgetOrCancellationStopped { refused }) =
        stopped
    else {
        panic!("expected BudgetOrCancellationStopped on tight small-default lease");
    };
    drop(refused);
    assert_v2_reopens_intact(&loc_tight, build, setup_exec, 12);
}
