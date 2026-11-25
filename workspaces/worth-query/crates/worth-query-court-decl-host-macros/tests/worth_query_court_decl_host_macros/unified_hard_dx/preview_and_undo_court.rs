//! Directory Home acceptance court for preview forks, `'static` owned history,
//! and restart-safe authored-body undo/redo + typed conflict refusal.
use worth_foundational::facade::CanonicalDate;
use worth_query_host::facade::{
    admission::authenticated_principal::WorthQueryAdmissionClock,
    application_entry::WorthQueryOwnedApplicationHistoryDocument,
    WorthQueryHomeAuthoredCorrectionOutcome, WorthQueryHomeUndoDenial, WorthQueryRefusalCategory,
    WorthQueryUserFacingRefusal,
};

use super::actions::{CorrectTaskState, RescheduleTask, SetTaskState};
use super::courts::{open_daymark, temp_home_path};
use super::program::owner;
use super::queries::{DaymarkHistory, ReadProjectTasks, ReadTaskDetails};
use super::schema::DaymarkSchema;

#[test]
fn target_dx_3_4_preview_forks_static_owned_history_and_reopen_authored_undo_redo_conflict() {
    let path = temp_home_path("preview-and-undo");
    let clock = WorthQueryAdmissionClock::system();
    let home = open_daymark(&path, &clock);

    // 1. Conflicting authored-body undo refuses with typed Conflict; stale preview
    //    refuses adoption after root advances; fresh preview adopts atomically.
    let entry = {
        let me = owner(&home, &clock);
        let conflict_entry = home
            .session(&me)
            .run(SetTaskState {
                task: 2,
                state: "REVIEW".to_owned(),
            })
            .unwrap()
            .correction_entry();
        let _ = home
            .session(&me)
            .run(SetTaskState {
                task: 2,
                state: "IN_PROGRESS".to_owned(),
            })
            .unwrap();
        let conflict = home
            .undo_authored::<CorrectTaskState>(&conflict_entry, &me)
            .expect_err("conflicting edit must refuse with typed Conflict");
        assert!(
            matches!(conflict, WorthQueryHomeUndoDenial::Conflict),
            "expected Conflict, got {conflict:?}"
        );
        assert_eq!(
            home.session(&me)
                .query(ReadTaskDetails { task: 2 })
                .unwrap()[0]
                .state,
            "IN_PROGRESS"
        );

        let stale = home.fork_preview("Stale-What-If").unwrap();
        let commit = home
            .session(&me)
            .run(SetTaskState {
                task: 1,
                state: "DONE".to_owned(),
            })
            .unwrap();
        let entry = commit.correction_entry();
        let failure = home
            .adopt_preview(stale)
            .expect_err("stale preview must refuse adoption after main advanced");
        assert!(
            failure.denial().is_stale_head(),
            "expected stale-head adoption refusal, got {:?}",
            failure.denial()
        );
        assert_eq!(
            failure.denial().refusal_category(),
            WorthQueryRefusalCategory::Conflict
        );
        let (stale_preview, _) = failure.into_parts();
        drop(stale_preview);

        // 2. Fresh preview fork -> edit on preview -> root untouched -> adopt_preview -> active sees it.
        let preview = home.fork_preview("What-If").unwrap();
        assert_eq!(preview.label(), "What-If");
        let preview_session = home.session(&me).on_branch(preview.branch());
        let _ = preview_session
            .run(RescheduleTask {
                task: 3,
                due_date: Some(CanonicalDate::parse_iso("2026-12-01").unwrap()),
            })
            .unwrap();
        let main_before = home
            .session(&me)
            .query(ReadTaskDetails { task: 3 })
            .unwrap();
        assert_eq!(
            main_before[0].due_date,
            Some(CanonicalDate::parse_iso("2026-11-29").unwrap())
        );

        let previous = home.adopt_preview(preview).unwrap();
        assert_ne!(previous, home.active_branch());
        let main_after = home
            .session(&me)
            .query(ReadTaskDetails { task: 3 })
            .unwrap();
        assert_eq!(
            main_after[0].due_date,
            Some(CanonicalDate::parse_iso("2026-12-01").unwrap())
        );

        // 3. 'static owned history document held in an unborrowed UI state struct.
        struct UiHistorySlot {
            doc: WorthQueryOwnedApplicationHistoryDocument<DaymarkSchema, DaymarkHistory>,
        }
        fn require_static<T: 'static>() {}
        require_static::<UiHistorySlot>();
        let doc = home
            .owned_history::<DaymarkHistory>(&me, ReadProjectTasks { project: 1 })
            .unwrap();
        let slot = UiHistorySlot { doc };
        drop(slot.doc);

        entry
    };
    home.close_clean().unwrap();

    // 4. Reopen Directory Home -> authored-body undo -> redo -> conflicted state intact.
    let home = open_daymark(&path, &clock);
    {
        let me = owner(&home, &clock);
        let session = home.session(&me);
        assert_eq!(
            session.query(ReadTaskDetails { task: 1 }).unwrap()[0].state,
            "DONE"
        );
        assert_eq!(
            session.query(ReadTaskDetails { task: 2 }).unwrap()[0].state,
            "IN_PROGRESS"
        );

        let undone = home.undo_authored::<CorrectTaskState>(&entry, &me).unwrap();
        assert!(matches!(
            undone,
            WorthQueryHomeAuthoredCorrectionOutcome::Committed { .. }
        ));
        assert_eq!(
            session.query(ReadTaskDetails { task: 1 }).unwrap()[0].state,
            "OPEN"
        );

        let redone = home.redo_authored::<CorrectTaskState>(&entry, &me).unwrap();
        assert!(matches!(
            redone,
            WorthQueryHomeAuthoredCorrectionOutcome::Committed { .. }
        ));
        assert_eq!(
            session.query(ReadTaskDetails { task: 1 }).unwrap()[0].state,
            "DONE"
        );
    }
    home.close_clean().unwrap();
    std::fs::remove_dir_all(path).unwrap();
}
