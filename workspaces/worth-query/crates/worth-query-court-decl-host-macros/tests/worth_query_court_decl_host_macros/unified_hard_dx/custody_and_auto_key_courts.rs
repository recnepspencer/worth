//! Courts for custody retention on preview refusal, active-branch
//! close protection, and restart-safe auto idempotency keys under frozen
//! clocks and pre-occupied keys.
use worth_query_host::facade::{
    admission::authenticated_principal::{
        WorthQueryAdmissionClock, WorthQueryAdmissionTimeController, WorthQueryCancellationSource,
        WorthQueryRequestScope,
    },
    application_contribution::{
        WorthQueryAdvancementDenial, WorthQueryManagedComputationInterruption,
    },
    application_installation::ApplicationBuildIdentity,
    product::WorthQueryProductBranchCreateError,
    WorthQueryApplicationHome, WorthQueryHomeAuthoredCorrectionOutcome, WorthQueryHomeConfig,
    WorthQueryPreviewDiscardDenial, WorthQueryPreviewForkDenial,
};

use super::actions::{CorrectTaskState, ProbeOccupiedTaskState, SequencedAutoKey, SetTaskState};
use super::courts::{open_daymark, temp_home_path};
use super::program::{owner, seed, DaymarkHome, DaymarkRuntime};
use super::queries::ReadTaskDetails;

fn open_memory_home(clock: &WorthQueryAdmissionClock) -> DaymarkHome {
    let build = ApplicationBuildIdentity::parse("5a5a5a5a-5a5a-5a5a-5a5a-5a5a5a5a5a5a").unwrap();
    WorthQueryApplicationHome::open_memory(
        WorthQueryHomeConfig::new(build, (DaymarkRuntime::new(),))
            .with_clock(clock.clone())
            .with_initial_state(|_, graph, installed| seed(graph, installed)),
    )
    .expect("open_memory succeeds")
}

fn cancelled_scope(clock: &WorthQueryAdmissionClock) -> WorthQueryRequestScope {
    let cancel = WorthQueryCancellationSource::new();
    cancel.cancel();
    WorthQueryRequestScope::new(None, cancel.token(), clock.clone())
}


#[test]
fn continue_preview_recovery_preserves_recovery_custody_on_refusal() {
    let path = temp_home_path("sol-blocker-6-recovery");
    let clock = WorthQueryAdmissionClock::system();
    let home = open_daymark(&path, &clock);
    {
        let healthy = WorthQueryRequestScope::new(
            None,
            WorthQueryCancellationSource::new().token(),
            clock.clone(),
        );
        let graph = home.primary_graph();
        let active = home.active_branch();
        // Obtain a real unpublished creation recovery on Directory Home.
        let initial = graph
            .with_application_advancement(&healthy, |phase| {
                let selected = graph.on_branch(active).select(&phase).unwrap();
                let fork = graph.branches().fork_expected(selected).unwrap();
                fork.components(|parts| parts.fork_relational().reuse_exact_signal_basis())
                    .create(&phase)
            })
            .unwrap();
        let recovery = match initial {
            Err(WorthQueryProductBranchCreateError::ProductUnpublished(recovery)) => recovery,
            Ok(immediate) => {
                // If published immediately on a given backend, verify fork_preview_with_scope
                // pre-entry refusal cleanly and return.
                drop(immediate);
                let cancelled = cancelled_scope(&clock);
                let err = home
                    .fork_preview_with_scope("Cancelled", &cancelled)
                    .err()
                    .expect("cancelled scope must refuse fork_preview_with_scope");
                assert!(matches!(
                    err,
                    WorthQueryPreviewForkDenial::Advancement(
                        WorthQueryAdvancementDenial::Interrupted(
                            WorthQueryManagedComputationInterruption::Cancelled
                        )
                    )
                ));
                return;
            }
            Err(other) => panic!("unexpected create error: {other:?}"),
        };

        // Pre-entry refusal on continue_preview_recovery must return exact recovery custody!
        let cancelled = cancelled_scope(&clock);
        let err = home
            .continue_preview_recovery("Recovered-Preview", recovery, &cancelled)
            .err()
            .expect("cancelled scope must refuse continuation pre-entry");
        assert!(
            matches!(
                err,
                WorthQueryPreviewForkDenial::ContinuationAdvancement {
                    denial: WorthQueryAdvancementDenial::Interrupted(
                        WorthQueryManagedComputationInterruption::Cancelled
                    ),
                    ..
                }
            ),
            "expected ContinuationAdvancement with Cancelled, got {err:?}"
        );
        let preserved = err.into_recovery().expect(
            "ContinuationAdvancement must preserve WorthQueryProductBranchCreationRecovery",
        );

        // Continuing the preserved recovery with a healthy scope succeeds!
        let preview = home
            .continue_preview_recovery("Recovered-Preview", preserved, &healthy)
            .expect("retrying continuation with preserved recovery succeeds");
        assert_eq!(preview.label(), "Recovered-Preview");
        drop(preview);
    }
    home.close_clean().unwrap();
    std::fs::remove_dir_all(path).unwrap();
}

#[test]
fn close_branch_refuses_active_branch_protection() {
    let clock = WorthQueryAdmissionClock::system();
    let home = open_memory_home(&clock);
    let active = home.active_branch();

    let err = home
        .close_branch(active)
        .expect_err("close_branch must refuse retiring the Home's active branch");
    assert!(
        matches!(err, WorthQueryPreviewDiscardDenial::ActiveBranchProtected),
        "expected ActiveBranchProtected, got {err:?}"
    );

    // Home remains completely usable afterwards.
    let me = owner(&home, &clock);
    let session = home.session(&me);
    let _ = session
        .run(SetTaskState {
            task: 1,
            state: "REVIEW".to_owned(),
        })
        .expect("Home remains writable after active-branch close refusal");
    assert_eq!(
        session.query(ReadTaskDetails { task: 1 }).unwrap()[0].state,
        "REVIEW"
    );
}

#[test]
fn auto_keys_survive_frozen_clock_restarts_and_preoccupied_keys() {
    // 1. Directory Home reopen with repeated frozen clock (t = 0 across restarts).
    let path = temp_home_path("sol-blocker-8-frozen-clock");
    let first_entry = {
        let frozen_before = WorthQueryAdmissionTimeController::new().unwrap();
        let _ = frozen_before.hold();
        let clock = frozen_before.clock();
        let home = open_daymark(&path, &clock);
        let entry = {
            let me = owner(&home, &clock);
            let commit = home
                .session(&me)
                .run(SetTaskState {
                    task: 1,
                    state: "REVIEW".to_owned(),
                })
                .expect("commit 1 under frozen clock succeeds");
            commit.correction_entry()
        };
        home.close_clean().unwrap();
        entry
    };

    {
        // Brand-new frozen clock at the exact same t = 0 after restart!
        let frozen_after = WorthQueryAdmissionTimeController::new().unwrap();
        let _ = frozen_after.hold();
        let clock = frozen_after.clock();
        let home = open_daymark(&path, &clock);
        {
            let me = owner(&home, &clock);
            let session = home.session(&me);
            let undone = home
                .undo_authored::<CorrectTaskState>(&first_entry, &me)
                .expect("undo_authored under repeated frozen clock succeeds");
            assert!(matches!(
                undone,
                WorthQueryHomeAuthoredCorrectionOutcome::Committed { .. }
            ));
            assert_eq!(
                session.query(ReadTaskDetails { task: 1 }).unwrap()[0].state,
                "OPEN"
            );

            let redone = home
                .redo_authored::<CorrectTaskState>(&first_entry, &me)
                .expect("redo_authored under repeated frozen clock succeeds");
            assert!(matches!(
                redone,
                WorthQueryHomeAuthoredCorrectionOutcome::Committed { .. }
            ));
            assert_eq!(
                session.query(ReadTaskDetails { task: 1 }).unwrap()[0].state,
                "REVIEW"
            );

            let second = session
                .run(SetTaskState {
                    task: 1,
                    state: "DONE".to_owned(),
                })
                .expect("commit 2 under repeated frozen clock succeeds freshly");
            let _ = second.correction_entry();
            assert_eq!(
                session.query(ReadTaskDetails { task: 1 }).unwrap()[0].state,
                "DONE"
            );
        }
        home.close_clean().unwrap();
    }
    std::fs::remove_dir_all(path).unwrap();

    // 2. Pre-occupied keys (both IdempotencyIntentDrift and AlreadyCommitted):
    //    session.run skips occupied probe keys cleanly and commits freshly.
    let frozen = WorthQueryAdmissionTimeController::new().unwrap();
    let _ = frozen.hold();
    let clock = frozen.clock();
    let home = open_memory_home(&clock);
    let me = owner(&home, &clock);
    let session = home.session(&me);

    // Pre-occupy probe 0 (SequencedAutoKey(1)) with "REVIEW".
    let _ = session
        .run_with_key(
            ProbeOccupiedTaskState {
                task: 1,
                state: "REVIEW".to_owned(),
            },
            &SequencedAutoKey(1),
        )
        .expect("explicit key occupies probe 0");

    // Different intent: probe 0 hits IdempotencyIntentDrift -> skips to probe 1 (SequencedAutoKey(2)).
    let _ = session
        .run(ProbeOccupiedTaskState {
            task: 1,
            state: "DONE".to_owned(),
        })
        .expect("auto key skips IdempotencyIntentDrift at probe 0 and commits on probe 1");
    assert_eq!(
        session.query(ReadTaskDetails { task: 1 }).unwrap()[0].state,
        "DONE"
    );

    // Same intent again: probe 0 hits IdempotencyIntentDrift, probe 1 hits AlreadyCommitted ->
    // skips BOTH cleanly and commits freshly on probe 2 (SequencedAutoKey(3))!
    let _ = session
        .run(ProbeOccupiedTaskState {
            task: 1,
            state: "DONE".to_owned(),
        })
        .expect("auto key skips AlreadyCommitted at probe 1 and commits freshly on probe 2");
}
