//! Full edit undo over an optional field: an absent due date is recorded as
//! `None` prior truth, restored by undo, reapplied by redo across a cold
//! reopen, and a concurrent due-only change (`Some -> Some`, `Some -> None`,
//! `None -> Some`) refuses through `expect_optional_input`.
use worth_foundational::facade::CanonicalDate;
use worth_query_host::facade::{
    admission::authenticated_principal::WorthQueryAdmissionClock, WorthQueryCorrectionEntry,
    WorthQueryCorrectionHistoryLimit, WorthQueryHomeAuthoredCorrectionOutcome,
    WorthQueryHomeUndoDenial,
};

use super::actions::RescheduleTask;
use super::courts::{open_daymark, temp_home_path};
use super::details_correction::{CorrectTaskDetails, EditTaskDetails};
use super::program::{owner, DaymarkHome, DaymarkProgram};
use super::queries::ReadTaskDetails;
use super::schema::DaymarkSchema;

fn date(iso: &str) -> CanonicalDate {
    CanonicalDate::parse_iso(iso).unwrap()
}

/// (title, due date) of `task`, read freshly through the public query surface.
fn details(
    home: &DaymarkHome,
    clock: &WorthQueryAdmissionClock,
    task: u64,
) -> (String, Option<CanonicalDate>) {
    let me = owner(home, clock);
    let row = home
        .session(&me)
        .query(ReadTaskDetails { task })
        .unwrap()
        .remove(0);
    (row.title, row.due_date)
}

fn reschedule(
    home: &DaymarkHome,
    clock: &WorthQueryAdmissionClock,
    task: u64,
    due: Option<CanonicalDate>,
) {
    home.session(owner(home, clock))
        .run(RescheduleTask {
            task,
            due_date: due,
        })
        .unwrap();
}

fn edit(
    home: &DaymarkHome,
    clock: &WorthQueryAdmissionClock,
    task: u64,
    title: &str,
    due: Option<CanonicalDate>,
) {
    home.session(owner(home, clock))
        .run(EditTaskDetails {
            task,
            title: title.to_owned(),
            due_date: due,
        })
        .unwrap();
}

fn top_undo(home: &DaymarkHome, clock: &WorthQueryAdmissionClock) -> WorthQueryCorrectionEntry {
    let listed = home
        .correction_history(
            owner(home, clock),
            WorthQueryCorrectionHistoryLimit::new(16).unwrap(),
        )
        .unwrap();
    let top = listed.next_undo().expect("an undoable edit");
    assert!(top.is_corrected_by::<CorrectTaskDetails>(), "{listed:?}");
    top.entry().clone()
}

fn top_redo(home: &DaymarkHome, clock: &WorthQueryAdmissionClock) -> WorthQueryCorrectionEntry {
    let listed = home
        .correction_history(
            owner(home, clock),
            WorthQueryCorrectionHistoryLimit::new(16).unwrap(),
        )
        .unwrap();
    let top = listed.next_redo().expect("a redoable edit");
    assert!(top.is_corrected_by::<CorrectTaskDetails>(), "{listed:?}");
    top.entry().clone()
}

fn committed(outcome: WorthQueryHomeAuthoredCorrectionOutcome<DaymarkSchema, DaymarkProgram>) {
    assert!(
        outcome.refusal_summary().is_none(),
        "expected a committed correction, got {outcome:?}"
    );
}

#[test]
fn undo_and_redo_restore_an_absent_due_date_across_cold_reopens() {
    let path = temp_home_path("option-preimage-undated");
    let clock = WorthQueryAdmissionClock::system();
    let home = open_daymark(&path, &clock);
    // Task 3 becomes undated, then one edit dates and renames it.
    reschedule(&home, &clock, 3, None);
    edit(&home, &clock, 3, "Review docs v2", Some(date("2026-12-01")));
    home.close_clean().unwrap();

    let home = open_daymark(&path, &clock);
    let me = owner(&home, &clock);
    committed(
        home.undo_authored::<CorrectTaskDetails>(&top_undo(&home, &clock), &me)
            .unwrap(),
    );
    assert_eq!(details(&home, &clock, 3), ("Review docs".to_owned(), None));
    committed(
        home.redo_authored::<CorrectTaskDetails>(&top_redo(&home, &clock), &me)
            .unwrap(),
    );
    assert_eq!(
        details(&home, &clock, 3),
        ("Review docs v2".to_owned(), Some(date("2026-12-01")))
    );
    drop(me);
    home.close_clean().unwrap();

    // The reapplied edit is undoable again after another cold reopen.
    let home = open_daymark(&path, &clock);
    let me = owner(&home, &clock);
    committed(
        home.undo_authored::<CorrectTaskDetails>(&top_undo(&home, &clock), &me)
            .unwrap(),
    );
    assert_eq!(details(&home, &clock, 3), ("Review docs".to_owned(), None));
    drop(me);
    home.close_clean().unwrap();
    std::fs::remove_dir_all(path).unwrap();
}

#[test]
fn undo_restores_a_due_date_an_edit_cleared() {
    let path = temp_home_path("option-preimage-cleared");
    let clock = WorthQueryAdmissionClock::system();
    let home = open_daymark(&path, &clock);
    let seeded = details(&home, &clock, 1);
    assert_eq!(seeded.1, Some(date("2026-11-25")));
    edit(&home, &clock, 1, "Draft spec v2", None);
    let me = owner(&home, &clock);
    committed(
        home.undo_authored::<CorrectTaskDetails>(&top_undo(&home, &clock), &me)
            .unwrap(),
    );
    assert_eq!(details(&home, &clock, 1), seeded);
    drop(me);
    home.close_clean().unwrap();
    std::fs::remove_dir_all(path).unwrap();
}

#[test]
fn a_concurrent_due_only_change_refuses_the_undo_in_either_direction() {
    let path = temp_home_path("option-preimage-concurrent");
    let clock = WorthQueryAdmissionClock::system();
    let home = open_daymark(&path, &clock);
    // None -> Some(d1) by the edit, then another session moves it to Some(d2).
    reschedule(&home, &clock, 3, None);
    edit(&home, &clock, 3, "Review docs v2", Some(date("2026-12-01")));
    let undated_edit = top_undo(&home, &clock);
    reschedule(&home, &clock, 3, Some(date("2026-12-02")));
    // Some(d0) -> Some(d1) by the edit, then another session clears it.
    edit(&home, &clock, 1, "Draft spec v2", Some(date("2026-12-03")));
    let dated_edit = top_undo(&home, &clock);
    reschedule(&home, &clock, 1, None);
    // Some(d0) -> None by the edit, then another session dates it again.
    edit(&home, &clock, 2, "Draft tests v2", None);
    let cleared_edit = top_undo(&home, &clock);
    reschedule(&home, &clock, 2, Some(date("2026-12-04")));

    let me = owner(&home, &clock);
    for (entry, task, kept) in [
        (&dated_edit, 1, ("Draft spec v2".to_owned(), None)),
        (
            &cleared_edit,
            2,
            ("Draft tests v2".to_owned(), Some(date("2026-12-04"))),
        ),
        (
            &undated_edit,
            3,
            ("Review docs v2".to_owned(), Some(date("2026-12-02"))),
        ),
    ] {
        let denial = home
            .undo_authored::<CorrectTaskDetails>(entry, &me)
            .expect_err("a due date changed since the edit must refuse the undo");
        assert!(
            matches!(denial, WorthQueryHomeUndoDenial::Conflict),
            "{denial:?}"
        );
        assert_eq!(
            details(&home, &clock, task),
            kept,
            "a refused undo changes nothing"
        );
    }
    drop(me);
    home.close_clean().unwrap();
    std::fs::remove_dir_all(path).unwrap();
}
