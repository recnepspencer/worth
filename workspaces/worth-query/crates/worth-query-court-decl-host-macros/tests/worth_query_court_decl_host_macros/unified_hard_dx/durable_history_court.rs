//! Durable correction history: discover, undo and redo authored writes after a
//! cold reopen with no application memory, and refuse a stale undo.
use worth_query_host::facade::{
    admission::authenticated_principal::{
        WorthQueryAdmissionClock, WorthQueryCancellationSource, WorthQueryRequestScope,
    },
    WorthQueryCorrectionHistory, WorthQueryCorrectionHistoryLimit,
    WorthQueryHomeAuthoredCorrectionOutcome, WorthQueryHomeUndoDenial, WorthQueryRefusalCategory,
    WorthQueryUserFacingRefusal,
};

use super::actions::{CorrectTaskState, SetTaskState};
use super::courts::{open_daymark, temp_home_path};
use super::program::{owner, DaymarkHome, DaymarkProgram};
use super::queries::ReadTaskDetails;
use super::refusal_summary_court::assert_readable;
use super::schema::DaymarkSchema;
use super::title_correction::{CorrectTaskTitle, RenameTask};

const SEEDED_TITLE: &str = "Review docs";
const RENAMED_TITLE: &str = "Review docs v2";

fn limit() -> WorthQueryCorrectionHistoryLimit {
    WorthQueryCorrectionHistoryLimit::new(16).unwrap()
}

/// (task 1 state, task 3 title) read freshly through the public query surface.
fn observed(home: &DaymarkHome, clock: &WorthQueryAdmissionClock) -> (String, String) {
    let me = owner(home, clock);
    let session = home.session(&me);
    let state = session.query(ReadTaskDetails { task: 1 }).unwrap()[0]
        .state
        .clone();
    let title = session.query(ReadTaskDetails { task: 3 }).unwrap()[0]
        .title
        .clone();
    (state, title)
}

fn history(
    home: &DaymarkHome,
    clock: &WorthQueryAdmissionClock,
) -> WorthQueryCorrectionHistory<DaymarkSchema> {
    home.correction_history(owner(home, clock), limit())
        .unwrap()
}

fn committed(outcome: WorthQueryHomeAuthoredCorrectionOutcome<DaymarkSchema, DaymarkProgram>) {
    assert!(
        outcome.refusal_summary().is_none(),
        "expected a committed correction, got {outcome:?}"
    );
}

#[test]
fn durable_history_drives_multi_step_undo_and_redo_across_cold_reopens() {
    assert!(WorthQueryCorrectionHistoryLimit::new(0).is_none());
    assert!(
        WorthQueryCorrectionHistoryLimit::new(WorthQueryCorrectionHistoryLimit::MAXIMUM + 1)
            .is_none()
    );
    let path = temp_home_path("durable-history");
    let clock = WorthQueryAdmissionClock::system();
    let home = open_daymark(&path, &clock);
    {
        let me = owner(&home, &clock);
        let session = home.session(&me);
        session
            .run(SetTaskState {
                task: 1,
                state: "IN_PROGRESS".to_owned(),
            })
            .unwrap();
        session
            .run(RenameTask {
                task: 3,
                title: RENAMED_TITLE.to_owned(),
            })
            .unwrap();
    }
    home.close_clean().unwrap();

    // Cold reopen: nothing about the writes survives in application memory.
    let home = open_daymark(&path, &clock);
    let listed = history(&home, &clock);
    let [newest, oldest] = listed.entries() else {
        panic!("expected exactly the two authored writes, got {listed:?}");
    };
    assert!(newest.sequence() > oldest.sequence());
    assert!(
        newest.is_corrected_by::<CorrectTaskTitle>()
            && !newest.is_corrected_by::<CorrectTaskState>()
    );
    assert!(oldest.is_corrected_by::<CorrectTaskState>());
    assert!(newest.is_applied() && oldest.is_applied());
    assert!(listed.next_redo().is_none());
    let bounded = home
        .correction_history(
            owner(&home, &clock),
            WorthQueryCorrectionHistoryLimit::new(1).unwrap(),
        )
        .unwrap();
    assert_eq!(bounded.entries().len(), 1);
    assert_eq!(bounded.entries()[0].original(), newest.original());

    // Undo step 2 (the rename), then step 1 (the state change), newest first.
    let me = owner(&home, &clock);
    let top = listed.next_undo().unwrap();
    assert!(top.is_corrected_by::<CorrectTaskTitle>());
    committed(
        home.undo_authored::<CorrectTaskTitle>(top.entry(), &me)
            .unwrap(),
    );
    assert_eq!(
        observed(&home, &clock),
        ("IN_PROGRESS".to_owned(), SEEDED_TITLE.to_owned())
    );
    let listed = history(&home, &clock);
    assert_eq!(
        listed.entries().len(),
        2,
        "a reversal is state, not a new entry: {listed:?}"
    );
    let top = listed.next_undo().unwrap();
    assert!(top.is_corrected_by::<CorrectTaskState>());
    committed(
        home.undo_authored::<CorrectTaskState>(top.entry(), &me)
            .unwrap(),
    );
    assert_eq!(
        observed(&home, &clock),
        ("OPEN".to_owned(), SEEDED_TITLE.to_owned())
    );

    // Redo restores the most recently undone write first.
    let listed = history(&home, &clock);
    assert!(listed.next_undo().is_none());
    assert!(listed.entries().iter().all(|entry| entry.is_reversed()));
    let redo = listed.next_redo().unwrap();
    assert!(redo.is_corrected_by::<CorrectTaskState>());
    committed(
        home.redo_authored::<CorrectTaskState>(redo.entry(), &me)
            .unwrap(),
    );
    assert_eq!(
        observed(&home, &clock),
        ("IN_PROGRESS".to_owned(), SEEDED_TITLE.to_owned())
    );
    drop(me);
    home.close_clean().unwrap();

    // Another cold reopen: the restored state and the remaining redo survive.
    let home = open_daymark(&path, &clock);
    assert_eq!(
        observed(&home, &clock),
        ("IN_PROGRESS".to_owned(), SEEDED_TITLE.to_owned())
    );
    let listed = history(&home, &clock);
    let me = owner(&home, &clock);
    let undo = listed.next_undo().unwrap();
    assert!(undo.is_corrected_by::<CorrectTaskState>());
    let redo = listed.next_redo().unwrap();
    assert!(redo.is_corrected_by::<CorrectTaskTitle>());
    committed(
        home.redo_authored::<CorrectTaskTitle>(redo.entry(), &me)
            .unwrap(),
    );
    assert_eq!(
        observed(&home, &clock),
        ("IN_PROGRESS".to_owned(), RENAMED_TITLE.to_owned())
    );

    // A redone write is undoable again through its latest reapplication.
    let listed = history(&home, &clock);
    let top = listed.next_undo().unwrap();
    assert!(top.is_corrected_by::<CorrectTaskTitle>());
    committed(
        home.undo_authored::<CorrectTaskTitle>(top.entry(), &me)
            .unwrap(),
    );
    assert_eq!(
        observed(&home, &clock),
        ("IN_PROGRESS".to_owned(), SEEDED_TITLE.to_owned())
    );

    // A second redo names the reapplication its active reversal undid.
    let listed = history(&home, &clock);
    let redo = listed.next_redo().unwrap();
    assert!(redo.is_corrected_by::<CorrectTaskTitle>());
    assert_ne!(redo.entry().coordinates(), redo.original());
    committed(
        home.redo_authored::<CorrectTaskTitle>(redo.entry(), &me)
            .unwrap(),
    );
    assert_eq!(
        observed(&home, &clock),
        ("IN_PROGRESS".to_owned(), RENAMED_TITLE.to_owned())
    );
    drop(me);
    home.close_clean().unwrap();
    std::fs::remove_dir_all(path).unwrap();
}

#[test]
fn a_newer_write_clears_redo_and_a_cancelled_listing_refuses_instead_of_shrinking() {
    let path = temp_home_path("durable-history-redo-cleared");
    let clock = WorthQueryAdmissionClock::system();
    let home = open_daymark(&path, &clock);
    let me = owner(&home, &clock);
    home.session(&me)
        .run(SetTaskState {
            task: 1,
            state: "IN_PROGRESS".to_owned(),
        })
        .unwrap();
    let top = history(&home, &clock).next_undo().unwrap().entry().clone();
    committed(home.undo_authored::<CorrectTaskState>(&top, &me).unwrap());
    assert!(history(&home, &clock).next_redo().is_some());

    home.session(&me)
        .run(RenameTask {
            task: 3,
            title: RENAMED_TITLE.to_owned(),
        })
        .unwrap();
    let listed = history(&home, &clock);
    assert!(
        listed.next_redo().is_none(),
        "a newer write clears redo: {listed:?}"
    );
    assert!(listed
        .next_undo()
        .unwrap()
        .is_corrected_by::<CorrectTaskTitle>());

    let cancellation = WorthQueryCancellationSource::new();
    cancellation.cancel();
    let cancelled = WorthQueryRequestScope::new(None, cancellation.token(), clock.clone());
    let stopped = home
        .correction_history_with_scope(&me, limit(), &cancelled)
        .expect_err("a cancelled listing refuses rather than reporting less history");
    assert_eq!(
        stopped.refusal_category(),
        WorthQueryRefusalCategory::Interrupted,
        "{stopped:?}"
    );
    assert_readable(&stopped.user_message());
    drop(me);
    home.close_clean().unwrap();
    std::fs::remove_dir_all(path).unwrap();
}

#[test]
fn a_stale_history_entry_fails_closed_after_another_session_edits_the_task() {
    let path = temp_home_path("durable-history-stale");
    let clock = WorthQueryAdmissionClock::system();
    let home = open_daymark(&path, &clock);
    home.session(owner(&home, &clock))
        .run(SetTaskState {
            task: 1,
            state: "IN_PROGRESS".to_owned(),
        })
        .unwrap();
    home.close_clean().unwrap();

    let home = open_daymark(&path, &clock);
    let recorded = history(&home, &clock).next_undo().unwrap().entry().clone();
    // A second session, opened independently, edits the same task.
    home.session(owner(&home, &clock))
        .run(SetTaskState {
            task: 1,
            state: "DONE".to_owned(),
        })
        .unwrap();
    let denial = home
        .undo_authored::<CorrectTaskState>(&recorded, owner(&home, &clock))
        .expect_err("an undo of a write changed since must refuse");
    assert!(
        matches!(denial, WorthQueryHomeUndoDenial::Conflict),
        "{denial:?}"
    );
    assert_eq!(
        denial.refusal_category(),
        WorthQueryRefusalCategory::Conflict
    );
    assert!(!denial.retryable());
    assert_readable(&denial.user_message());
    assert_eq!(observed(&home, &clock).0, "DONE");

    // Edits made on a preview branch are not part of the active branch history.
    let preview = home.fork_preview("Isolated").unwrap();
    home.session(owner(&home, &clock))
        .on_branch(preview.branch())
        .run(RenameTask {
            task: 3,
            title: RENAMED_TITLE.to_owned(),
        })
        .unwrap();
    let listed = history(&home, &clock);
    assert_eq!(listed.entries().len(), 2, "{listed:?}");
    assert!(listed
        .entries()
        .iter()
        .all(|entry| entry.is_corrected_by::<CorrectTaskState>()));
    drop(preview);
    home.close_clean().unwrap();
    std::fs::remove_dir_all(path).unwrap();
}
