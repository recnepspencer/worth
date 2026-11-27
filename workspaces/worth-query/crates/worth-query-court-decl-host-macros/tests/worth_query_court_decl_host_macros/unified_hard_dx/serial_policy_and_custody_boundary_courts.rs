//! Courts covering installed serial World policy ceilings (including
//! single-metered occupied-key probes and automatic undo/redo), watch refresh
//! delivery handoff under cancellation, and owned-history custody retention.
use std::num::NonZeroUsize;

use worth_foundational::facade::{
    DeterminismContract, ExecutionBudget, ExecutionPosture, ExecutionRequestPolicy,
};
use worth_query_host::facade::{
    admission::authenticated_principal::{
        WorthQueryAdmissionClock, WorthQueryCancellationSource, WorthQueryRequestScope,
    },
    application_contribution::{
        WorthQueryAdvancementDenial, WorthQueryManagedComputationInterruption,
        WorthQueryManagedComputationResourceDenial,
    },
    application_entry::{
        WorthQueryApplicationRequestMutationDenial, WorthQueryApplicationRequestQueryDenial,
    },
    application_installation::ApplicationBuildIdentity,
    WorthQueryApplicationHome, WorthQueryHomeConfig, WorthQueryHomeOwnedHistoryDenial,
    WorthQueryHomeUndoDenial, WorthQuerySessionRunDenial, WorthQueryTypedManagedIndexRefreshDenial,
};

use super::actions::{
    CorrectTaskState, ProbeOccupiedTaskState, RenameProject, SequencedAutoKey, SetTaskState,
};
use super::program::{owner, seed, DaymarkHome, DaymarkRuntime};
use super::queries::{
    DaymarkHistory, FindTasksByTitle, ReadProjectTasks, ReadTaskDetails, ReadyTasks,
};

pub(super) fn open_memory_home_with_policy(
    clock: &WorthQueryAdmissionClock,
    policy: Option<ExecutionRequestPolicy>,
) -> DaymarkHome {
    let build = ApplicationBuildIdentity::parse("5a5a5a5a-5a5a-5a5a-5a5a-5a5a5a5a5a5a").unwrap();
    let mut config = WorthQueryHomeConfig::new(build, (DaymarkRuntime::new(),))
        .with_clock(clock.clone())
        .with_initial_state(|_, graph, installed| seed(graph, installed));
    if let Some(policy) = policy {
        config = config.with_execution_policy(policy);
    }
    WorthQueryApplicationHome::open_memory(config).expect("open_memory succeeds")
}

#[test]
fn installed_serial_ceiling_and_single_metered_probe_budget() {
    let clock = WorthQueryAdmissionClock::system();
    let installed_ceiling = 2_000_000_u64;
    let installed_policy = ExecutionRequestPolicy::new(
        ExecutionPosture::Serial,
        DeterminismContract::CanonicalBitwise,
        ExecutionBudget::new(NonZeroUsize::MIN, None, Some(installed_ceiling)),
    );
    let home = open_memory_home_with_policy(&clock, Some(installed_policy));
    let me = owner(&home, &clock);

    // 1. Default session inherits the installed serial ceiling and succeeds.
    let default_session = home.session(&me);
    let rows = default_session
        .query(ReadyTasks {
            project: 1,
            priority: None,
            due_before: None,
        })
        .expect("default session stays within installed serial World ceiling");
    assert_eq!(rows.len(), 2);

    // 2. Requesting a budget exceeding the installed serial World ceiling MUST
    //    fail closed with WorkLimit on both query and run!
    let wider = default_session.clone().with_budget(ExecutionBudget::new(
        NonZeroUsize::MIN,
        None,
        Some(installed_ceiling + 1),
    ));
    let query_err = wider
        .query(ReadyTasks {
            project: 1,
            priority: None,
            due_before: None,
        })
        .expect_err("session exceeding installed serial ceiling must refuse query");
    assert!(
        matches!(
            query_err,
            WorthQueryApplicationRequestQueryDenial::ExecutionRequest(
                WorthQueryAdvancementDenial::Resource(
                    WorthQueryManagedComputationResourceDenial::WorkLimit
                )
            )
        ),
        "expected WorkLimit refusal on wider serial policy, got {query_err:?}"
    );
    let run_err = wider
        .run(SetTaskState {
            task: 1,
            state: "REVIEW".to_owned(),
        })
        .expect_err("session exceeding installed serial ceiling must refuse run");
    assert!(
        matches!(
            run_err,
            WorthQuerySessionRunDenial::Request(
                WorthQueryApplicationRequestMutationDenial::ExecutionRequest(
                    WorthQueryAdvancementDenial::Resource(
                        WorthQueryManagedComputationResourceDenial::WorkLimit
                    )
                )
            )
        ),
        "expected WorkLimit refusal on wider serial run, got {run_err:?}"
    );

    // 3. Single metered request across seed read + occupied-key probes:
    //    Occupy 48 consecutive keys so `run` must charge seed read + 48 probes
    //    cumulatively against ONE request budget.
    let funded = home.session(&me);
    for slot in 1..=48_u64 {
        let _ = funded
            .run_with_key(
                ProbeOccupiedTaskState {
                    task: 1,
                    state: format!("SLOT-{slot}"),
                },
                &SequencedAutoKey(slot),
            )
            .expect("occupying probe slot succeeds");
    }

    let mut found = None;
    let mut candidate = 1_000_u64;
    let mut explicit_key = 10_000_u64;
    while candidate <= installed_ceiling {
        explicit_key += 1;
        let tight = funded.clone().with_budget(ExecutionBudget::new(
            NonZeroUsize::MIN,
            None,
            Some(candidate),
        ));
        if tight
            .run_with_key(
                ProbeOccupiedTaskState {
                    task: 1,
                    state: "SINGLE-COMMIT".to_owned(),
                },
                &SequencedAutoKey(explicit_key),
            )
            .is_ok()
        {
            found = Some(candidate);
            break;
        }
        candidate = candidate.saturating_mul(2);
    }
    let single_commit_budget = found
        .expect("found a finite budget where a single direct commit succeeds")
        .saturating_add(found.unwrap() / 2)
        .min(installed_ceiling);
    // Confirm a single direct commit succeeds at `single_commit_budget`.
    explicit_key += 1;
    let tight_direct = funded.clone().with_budget(ExecutionBudget::new(
        NonZeroUsize::MIN,
        None,
        Some(single_commit_budget),
    ));
    let _ = tight_direct
        .run_with_key(
            ProbeOccupiedTaskState {
                task: 1,
                state: "SINGLE-COMMIT-OK".to_owned(),
            },
            &SequencedAutoKey(explicit_key),
        )
        .expect("single direct commit fits within calibrated single_commit_budget");

    let tight_auto = funded.clone().with_budget(ExecutionBudget::new(
        NonZeroUsize::MIN,
        None,
        Some(single_commit_budget),
    ));
    let cumulative_err = tight_auto
        .run(ProbeOccupiedTaskState {
            task: 1,
            state: "CUMULATIVE-OVERFLOW".to_owned(),
        })
        .expect_err("cumulative work across 48 occupied probes must exhaust single-commit budget");
    let rendered = format!("{cumulative_err:?}");
    assert!(
        rendered.contains("WorkExhausted"),
        "expected cumulative WorkExhausted refusal, got {rendered}"
    );
}

#[test]
fn watch_refresh_preserves_delivery_and_handles_cancellation() {
    let clock = WorthQueryAdmissionClock::system();
    let home = open_memory_home_with_policy(&clock, None);
    let me = owner(&home, &clock);
    let session = home.session(&me);
    let _idx = session
        .ensure_index_ready(FindTasksByTitle {
            project: 1,
            prefix: String::new(),
        })
        .unwrap();

    let mut watch = session
        .watch(FindTasksByTitle {
            project: 1,
            prefix: "Draft".to_owned(),
        })
        .expect("watch opens");
    assert_eq!(watch.current().unwrap().len(), 2);

    let _ = session
        .run(SetTaskState {
            task: 1,
            state: "REVIEW".to_owned(),
        })
        .unwrap();

    // Pre-cancelled refresh refuses before acknowledging the notification and
    // clears stale rows; subsequent healthy refresh delivers via `finish_next`.
    let cancel = WorthQueryCancellationSource::new();
    cancel.cancel();
    let cancelled_session = session.clone().with_cancellation(cancel.token());
    let cancel_err = watch
        .refresh(&cancelled_session)
        .expect_err("cancelled session must refuse watch refresh");
    assert!(
        matches!(
            cancel_err,
            WorthQueryTypedManagedIndexRefreshDenial::Query(
                WorthQueryApplicationRequestQueryDenial::ExecutionRequest(
                    WorthQueryAdvancementDenial::Interrupted(
                        WorthQueryManagedComputationInterruption::Cancelled
                    )
                )
            )
        ),
        "expected Cancelled refresh denial, got {cancel_err:?}"
    );
    assert!(!watch.is_current());
    assert!(watch.current().is_none());

    let delivered = watch
        .refresh(&session)
        .expect("healthy refresh after pre-entry cancellation succeeds")
        .expect("unconsumed notification delivers updated rows");
    assert_eq!(delivered.len(), 2);
    assert_eq!(delivered[0].state, "REVIEW");
    assert!(watch.is_current());
    watch.close().unwrap();
}

#[test]
fn owned_history_completion_boundary_preserves_head_custody() {
    let clock = WorthQueryAdmissionClock::system();
    let home = open_memory_home_with_policy(&clock, None);
    let me = owner(&home, &clock);

    let cancel = WorthQueryCancellationSource::new();
    cancel.cancel();
    let cancelled_scope = WorthQueryRequestScope::new(None, cancel.token(), clock.clone());
    let fork_err = home
        .owned_history_with_scope::<DaymarkHistory>(
            &me,
            ReadProjectTasks { project: 1 },
            &cancelled_scope,
        )
        .expect_err("already-cancelled scope must refuse before allocating head");
    assert!(matches!(
        fork_err,
        WorthQueryHomeOwnedHistoryDenial::Fork(_)
    ));

    let attach_err = home
        .owned_history::<DaymarkHistory>(&me, ReadProjectTasks { project: 999 })
        .expect_err("missing project scope must refuse attachment");
    assert!(matches!(
        attach_err,
        WorthQueryHomeOwnedHistoryDenial::Attachment { .. }
    ));
    let head = attach_err
        .into_owned_head()
        .expect("Attachment failure retains owned head");

    let pre_entry_err = home
        .attach_owned_history_head::<DaymarkHistory>(
            head,
            &me,
            ReadProjectTasks { project: 1 },
            &cancelled_scope,
        )
        .expect_err("cancelled scope refuses pre-entry");
    let head_again = pre_entry_err
        .into_owned_head()
        .expect("pre-entry refusal retains owned head");
    let healthy = home.session(&me).fresh_scope();
    let doc = home
        .attach_owned_history_head::<DaymarkHistory>(
            head_again,
            &me,
            ReadProjectTasks { project: 1 },
            &healthy,
        )
        .expect("retrying retained head under healthy scope succeeds");
    home.primary_graph()
        .with_application_advancement(&healthy, |phase| doc.close(&phase))
        .unwrap()
        .unwrap();
}

#[test]
fn automatic_undo_and_redo_honor_installed_serial_ceiling() {
    let clock = WorthQueryAdmissionClock::system();
    let installed_policy = ExecutionRequestPolicy::new(
        ExecutionPosture::Serial,
        DeterminismContract::CanonicalBitwise,
        ExecutionBudget::new(NonZeroUsize::MIN, None, Some(2_000_000)),
    );
    let home = open_memory_home_with_policy(&clock, Some(installed_policy));
    let me = owner(&home, &clock);
    let session = home.session(&me);

    // 1. Conventional `home.undo` and `home.redo` seed selection both honor
    //    the installed 2_000_000 serial ceiling and advance past seed selection
    //    into history selection (never refusing with `WorkLimit`).
    let rename = session
        .run(RenameProject {
            project: 1,
            label: "Core Serial".to_owned(),
        })
        .expect("RenameProject commits within 2_000_000 serial ceiling");
    let rename_entry = rename.correction_entry();
    let non_correctable = home
        .undo(&rename_entry, &me)
        .expect_err("RenameProject has no recorded_inverse correction");
    assert!(
        matches!(non_correctable, WorthQueryHomeUndoDenial::Correction(_))
            && format!("{non_correctable:?}").contains("SourceNotCorrectable"),
        "expected home.undo to pass installed serial seed selection and reach SourceNotCorrectable, got {non_correctable:?}"
    );
    let redo_err = home
        .redo(&rename_entry, &me)
        .expect_err("RenameProject has no recorded_inverse correction for redo");
    assert!(
        matches!(redo_err, WorthQueryHomeUndoDenial::Correction(_))
            && format!("{redo_err:?}").contains("SourceNotCorrectable"),
        "expected home.redo to pass installed serial seed selection and reach SourceNotCorrectable, got {redo_err:?}"
    );

    // 2. Authored-body automatic undo + redo on a 2_000_000 serial Home.
    let state_edit = session
        .run(SetTaskState {
            task: 1,
            state: "REVIEW".to_owned(),
        })
        .expect("SetTaskState commits within 2_000_000 serial ceiling");
    let state_entry = state_edit.correction_entry();
    let authored_undone = home
        .undo_authored::<CorrectTaskState>(&state_entry, &me)
        .expect("home.undo_authored honors installed 2_000_000 serial ceiling");
    assert!(authored_undone.live_receipt().is_some());
    assert_eq!(
        session.query(ReadTaskDetails { task: 1 }).unwrap()[0].state,
        "OPEN"
    );
    let authored_redone = home
        .redo_authored::<CorrectTaskState>(&state_entry, &me)
        .expect("home.redo_authored honors installed 2_000_000 serial ceiling");
    assert!(authored_redone.live_receipt().is_some());
    assert_eq!(
        session.query(ReadTaskDetails { task: 1 }).unwrap()[0].state,
        "REVIEW"
    );
}
