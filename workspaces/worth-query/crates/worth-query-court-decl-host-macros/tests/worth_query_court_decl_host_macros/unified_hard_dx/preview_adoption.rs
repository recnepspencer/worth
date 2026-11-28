//! Directory Home acceptance court for preview forks.
use worth_foundational::facade::CanonicalDate;
use worth_query_host::facade::{
    admission::authenticated_principal::WorthQueryAdmissionClock,
    WorthQueryRefusalCategory,
    WorthQueryUserFacingRefusal,
};

use super::actions::{RescheduleTask, SetTaskState};
use super::courts::{open_daymark, temp_home_path};
use super::program::owner;
use super::queries::ReadTaskDetails;

#[test]
fn preview_adoption_refuses_a_stale_head_and_persists_an_adopted_edit() {
    let path = temp_home_path("preview-adoption");
    let clock = WorthQueryAdmissionClock::system();
    let home = open_daymark(&path, &clock);

    {
        let me = owner(&home, &clock);
        let stale = home.fork_preview("Stale-What-If").unwrap();
        home.session(&me)
            .run(SetTaskState {
                task: 1,
                state: "DONE".to_owned(),
            })
            .unwrap();
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

        // Fresh preview fork -> edit on preview -> root untouched -> adopt_preview -> active sees it.
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

    }
    home.close_clean().unwrap();

    // Reopen and observe the adopted head.
    let home = open_daymark(&path, &clock);
    {
        let me = owner(&home, &clock);
        let session = home.session(&me);
        assert_eq!(
            session.query(ReadTaskDetails { task: 1 }).unwrap()[0].state,
            "DONE"
        );
    }
    home.close_clean().unwrap();
    std::fs::remove_dir_all(path).unwrap();
}
