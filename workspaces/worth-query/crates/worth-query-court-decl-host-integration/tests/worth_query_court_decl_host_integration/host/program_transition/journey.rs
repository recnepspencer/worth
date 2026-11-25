//! Real Daymark `v2 -> v3` Directory Home upgrade journeys, including
//! named-boundary budget-stop atomic rollback, managed equality-index rebuild,
//! idempotent reopen, entity scan bound refusal, predecessor-field overwrite
//! refusal, non-indexed rebuild refusal, and custom migration step composition.

use std::path::Path;
use std::sync::{
    atomic::{AtomicUsize, Ordering},
    Arc,
};

use worth_query_host::facade::{
    application_contribution::with_bootstrap_advancement_from_request,
    application_entry::{WorthQueryApplicationMutationOutcome, WorthQueryApplicationRequestExt},
    application_installation::ApplicationBuildIdentity,
    program_transition::{
        open_directory_with_transition, open_directory_with_transition_and_step,
        worth_query_default_build_id, WorthQueryProgramTransitionOpenConfig,
        WorthQueryProgramTransitionOpenDenial, WorthQueryTransitionOpenOutcome,
    },
    runtime::ExecutionAllocationPolicy,
    WorthQueryHomeProgramTransitionDenial, WorthQueryHomeProgramTransitionOpenPhase,
};
use worth_query_testkit_host::facade::host::{
    application_home as fixture, application_runtime_surface as identity, checkpoint_capture,
};
use worth_relational::facade::indexes::DerivedIndexMaintenanceBudget;

use super::driver::request_scope;
use super::v2_mutations::{CreateTaskV2, TaskV2Input};
use super::v2_program::v2_request;
use super::v2_queries::AllTasksV2Read;
use super::v3_mutations::{TaskV3Input, UpdateTaskV3};
use super::v3_program::{v3_request, DaymarkV2ToV3};
use super::v3_queries::{AllTasksV3Read, TasksByPriorityRead};

pub(super) fn seed_v2_tasks(
    location: &Path,
    build: ApplicationBuildIdentity,
    execution: worth_execution::ExecutionRequest<'_, '_>,
    tasks: &[(&str, &str)],
) {
    let v2_home = with_bootstrap_advancement_from_request(execution, |phase| {
        Box::new(
            v2_request(location.to_path_buf(), build)
                .open_in_advancement(&phase)
                .expect("v2 home creates"),
        )
    })
    .unwrap();
    {
        let scope = request_scope();
        let runtime = v2_home.runtime();
        let principal = identity::inspection_authentication::authenticate(
            runtime.runtime().installed_schema(),
            &scope,
        );
        for (idx, (task_key, v2_value)) in tasks.iter().enumerate() {
            let committed = runtime
                .request(&principal, &scope)
                .advanced()
                .mutate(CreateTaskV2(TaskV2Input {
                    task_key: (*task_key).to_owned(),
                    v2_value: (*v2_value).to_owned(),
                }))
                .without_source()
                .idempotency(&((idx as u64) + 1))
                .execute_in_program(runtime, ExecutionAllocationPolicy::SystemAllocation)
                .unwrap();
            assert!(matches!(
                committed,
                WorthQueryApplicationMutationOutcome::Committed { .. }
            ));
        }
    }
    (*v2_home).close(&execution).unwrap().reclaim();
}

pub(super) fn assert_v2_reopens_intact(
    location: &Path,
    build: ApplicationBuildIdentity,
    execution: worth_execution::ExecutionRequest<'_, '_>,
    expected_rows: usize,
) {
    let reopened_v2 = with_bootstrap_advancement_from_request(execution, |phase| {
        Box::new(
            v2_request(location.to_path_buf(), build)
                .open_in_advancement(&phase)
                .expect("plain v2 home must reopen cleanly without TransitionPending"),
        )
    })
    .unwrap();
    {
        let scope = request_scope();
        let runtime = reopened_v2.runtime();
        let principal = identity::inspection_authentication::authenticate(
            runtime.runtime().installed_schema(),
            &scope,
        );
        let rows = runtime
            .request(&principal, &scope)
            .query(AllTasksV2Read("p-main".to_owned()))
            .execute()
            .unwrap();
        assert_eq!(rows.rows().len(), expected_rows);
    }
    (*reopened_v2).close(&execution).unwrap().reclaim();
}

fn verify_budget_stop_keeps_v2_intact(
    location: &Path,
    build: ApplicationBuildIdentity,
    execution: worth_execution::ExecutionRequest<'_, '_>,
    expected_rows: usize,
) {
    let tight_index_budget = DerivedIndexMaintenanceBudget {
        maximum_work_units: Some(1),
        maximum_cold_record_slots: Some(1),
        maximum_derived_rows: Some(1),
    };
    let config =
        WorthQueryProgramTransitionOpenConfig::default_bounded().with_resources(tight_index_budget);
    let stopped = with_bootstrap_advancement_from_request(execution, |phase| {
        open_directory_with_transition::<DaymarkV2ToV3>(
            &phase,
            v2_request(location.to_path_buf(), build),
            v3_request(location.to_path_buf(), build),
            config,
        )
    })
    .expect("outer bootstrap must succeed so budget stop fires at transition boundary");
    let denial = stopped
        .err()
        .expect("budget-exhausted transition must fail closed");
    let WorthQueryProgramTransitionOpenDenial::BudgetOrCancellationStopped { refused } = denial
    else {
        panic!("expected typed BudgetOrCancellationStopped, got {denial:?}");
    };
    assert!(
        matches!(
            refused.phase(),
            WorthQueryHomeProgramTransitionOpenPhase::Transition {
                cause: WorthQueryHomeProgramTransitionDenial::ColdSlotBudgetExceeded(_)
                    | WorthQueryHomeProgramTransitionDenial::Indexes(_),
                ..
            }
        ),
        "expected pre-intent Transition cold-slot/index budget refusal, got {:?}",
        refused.phase()
    );
    drop(refused);
    assert_v2_reopens_intact(location, build, execution, expected_rows);
}

fn verify_upgraded_v3_and_promote_t1(
    v3_home: &worth_query_host::facade::WorthQueryApplicationHome<
        super::v3_program::DaymarkV3Schema,
        super::v3_program::DaymarkV3Program,
    >,
) {
    let scope = request_scope();
    let runtime = v3_home.runtime();
    let principal = identity::inspection_authentication::authenticate(
        runtime.runtime().installed_schema(),
        &scope,
    );
    let normal = runtime
        .request(&principal, &scope)
        .query(TasksByPriorityRead {
            project: "p-main".to_owned(),
            priority: "Normal".to_owned(),
        })
        .execute()
        .unwrap();
    assert_eq!(normal.rows().len(), 2);

    let edit = runtime
        .request(&principal, &scope)
        .advanced()
        .mutate(UpdateTaskV3(TaskV3Input {
            task_key: "t1".to_owned(),
            v2_value: "a".to_owned(),
            priority: "High".to_owned(),
        }))
        .without_source()
        .idempotency(&99_u64)
        .execute_in_program(runtime, ExecutionAllocationPolicy::SystemAllocation)
        .unwrap();
    assert!(matches!(
        edit,
        WorthQueryApplicationMutationOutcome::Committed { .. }
    ));
}

fn verify_reopened_v3_preserves_high(
    v3_home: &worth_query_host::facade::WorthQueryApplicationHome<
        super::v3_program::DaymarkV3Schema,
        super::v3_program::DaymarkV3Program,
    >,
) {
    let scope = request_scope();
    let runtime = v3_home.runtime();
    let principal = identity::inspection_authentication::authenticate(
        runtime.runtime().installed_schema(),
        &scope,
    );
    let all = runtime
        .request(&principal, &scope)
        .query(AllTasksV3Read("p-main".to_owned()))
        .execute()
        .unwrap();
    assert_eq!(all.rows().len(), 2);
    let high = runtime
        .request(&principal, &scope)
        .query(TasksByPriorityRead {
            project: "p-main".to_owned(),
            priority: "High".to_owned(),
        })
        .execute()
        .unwrap();
    assert_eq!(high.rows().len(), 1);
    assert_eq!(high.rows()[0].task_key, "t1");
}

#[test]
fn daymark_v2_to_v3_upgrade_preserves_data_rebuilds_index_and_rolls_back_on_budget_stop() {
    let location = fixture::location();
    let build = worth_query_default_build_id!();
    let lease = checkpoint_capture::lease(128 * 1024 * 1024);
    let execution = worth_execution::ExecutionRequest::leased(&lease);

    seed_v2_tasks(&location, build, execution, &[("t1", "a"), ("t2", "b")]);
    verify_budget_stop_keeps_v2_intact(&location, build, execution, 2);

    let upgraded = with_bootstrap_advancement_from_request(execution, |phase| {
        Box::new(
            open_directory_with_transition::<DaymarkV2ToV3>(
                &phase,
                v2_request(location.clone(), build),
                v3_request(location.clone(), build),
                WorthQueryProgramTransitionOpenConfig::default_bounded(),
            )
            .expect("v2 -> v3 upgrade succeeds"),
        )
    })
    .unwrap();
    let (v3_home, outcome) = (*upgraded).into_parts();
    assert!(matches!(
        outcome,
        WorthQueryTransitionOpenOutcome::UpgradedInPlace {
            from_version: 2,
            to_version: 3,
            backfilled_records: 2,
            ref indexes_marked_for_rebuild,
        } if indexes_marked_for_rebuild.len() == 1
    ));
    verify_upgraded_v3_and_promote_t1(&v3_home);
    v3_home.close(&execution).unwrap().reclaim();

    let reopened_v3 = with_bootstrap_advancement_from_request(execution, |phase| {
        Box::new(
            open_directory_with_transition::<DaymarkV2ToV3>(
                &phase,
                v2_request(location.clone(), build),
                v3_request(location, build),
                WorthQueryProgramTransitionOpenConfig::default_bounded(),
            )
            .expect("reopened v3 home succeeds"),
        )
    })
    .unwrap();
    let (v3_home, outcome) = (*reopened_v3).into_parts();
    assert_eq!(
        outcome,
        WorthQueryTransitionOpenOutcome::OpenedCurrent { program_version: 3 }
    );
    verify_reopened_v3_preserves_high(&v3_home);
    v3_home.close(&execution).unwrap().reclaim();
}

fn open_with_counted_step(
    location: &Path,
    build: ApplicationBuildIdentity,
    execution: worth_execution::ExecutionRequest<'_, '_>,
    calls: &Arc<AtomicUsize>,
) -> WorthQueryTransitionOpenOutcome {
    let counter = calls.clone();
    let opened = with_bootstrap_advancement_from_request(execution, |phase| {
        Box::new(
            open_directory_with_transition_and_step::<DaymarkV2ToV3, _>(
                &phase,
                (
                    v2_request(location.to_path_buf(), build),
                    v3_request(location.to_path_buf(), build),
                ),
                WorthQueryProgramTransitionOpenConfig::default_bounded(),
                move |_source, _target| {
                    counter.fetch_add(1, Ordering::SeqCst);
                    Ok(Vec::new())
                },
            )
            .expect("counted-step open succeeds"),
        )
    })
    .unwrap();
    let (home, outcome) = (*opened).into_parts();
    home.close(&execution).unwrap().reclaim();
    outcome
}

#[test]
fn open_directory_with_transition_and_step_runs_custom_step_once_on_upgrade_only() {
    let location = fixture::location();
    let build = worth_query_default_build_id!();
    let lease = checkpoint_capture::lease(128 * 1024 * 1024);
    let execution = worth_execution::ExecutionRequest::leased(&lease);

    seed_v2_tasks(&location, build, execution, &[("t1", "a")]);
    let calls = Arc::new(AtomicUsize::new(0));

    let upgrade_outcome = open_with_counted_step(&location, build, execution, &calls);
    assert_eq!(calls.load(Ordering::SeqCst), 1);
    assert!(matches!(
        upgrade_outcome,
        WorthQueryTransitionOpenOutcome::UpgradedInPlace {
            backfilled_records: 1,
            ..
        }
    ));

    let reopen_outcome = open_with_counted_step(&location, build, execution, &calls);
    assert_eq!(calls.load(Ordering::SeqCst), 1);
    assert_eq!(
        reopen_outcome,
        WorthQueryTransitionOpenOutcome::OpenedCurrent { program_version: 3 }
    );
}
