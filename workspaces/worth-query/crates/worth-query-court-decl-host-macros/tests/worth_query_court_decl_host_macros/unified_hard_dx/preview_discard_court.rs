//! Memory Home verification of explicit preview discard, branch retirement, and owned history close.
use worth_foundational::facade::CanonicalDate;
use worth_query_host::facade::{
    admission::authenticated_principal::WorthQueryAdmissionClock,
    application_entry::WorthQueryOwnedApplicationHistoryDocument,
    application_installation::ApplicationBuildIdentity, WorthQueryApplicationHome,
    WorthQueryHomeConfig, WorthQueryHomeOpenDisposition,
};

use super::actions::RescheduleTask;
use super::program::{owner, seed, DaymarkHome, DaymarkRuntime};
use super::queries::{DaymarkHistory, ReadProjectTasks, ReadTaskDetails};
use super::schema::DaymarkSchema;

#[test]
fn target_dx_3_memory_home_preview_discard_and_close_branch() {
    let clock = WorthQueryAdmissionClock::system();
    let build = ApplicationBuildIdentity::parse("5a5a5a5a-5a5a-5a5a-5a5a-5a5a5a5a5a5a").unwrap();
    let home: DaymarkHome = WorthQueryApplicationHome::open_memory(
        WorthQueryHomeConfig::new(build, (DaymarkRuntime::new(),))
            .with_clock(clock.clone())
            .with_initial_state(|_, graph, installed| seed(graph, installed)),
    )
    .expect("open_memory succeeds");
    assert_eq!(
        home.open_disposition(),
        WorthQueryHomeOpenDisposition::Created
    );

    {
        let me = owner(&home, &clock);
        let discarded = home.fork_preview("Discard-Me").unwrap();
        let _ = home
            .session(&me)
            .on_branch(discarded.branch())
            .run(RescheduleTask {
                task: 3,
                due_date: Some(CanonicalDate::parse_iso("2026-12-15").unwrap()),
            })
            .unwrap();
        let receipt = discarded.discard(&home).unwrap();
        assert!(receipt.retired_component_count() >= 1);
        let main_row = home
            .session(&me)
            .query(ReadTaskDetails { task: 3 })
            .unwrap();
        assert_eq!(
            main_row[0].due_date,
            Some(CanonicalDate::parse_iso("2026-11-29").unwrap())
        );

        let adopted = home.fork_preview("Adopt-Me").unwrap();
        let _ = home
            .session(&me)
            .on_branch(adopted.branch())
            .run(RescheduleTask {
                task: 3,
                due_date: Some(CanonicalDate::parse_iso("2026-12-20").unwrap()),
            })
            .unwrap();
        let previous = home.adopt_preview(adopted).unwrap();
        let closed_prev = home.close_branch(previous).unwrap();
        assert_eq!(closed_prev.product_branch(), previous);
        let after = home
            .session(&me)
            .query(ReadTaskDetails { task: 3 })
            .unwrap();
        assert_eq!(
            after[0].due_date,
            Some(CanonicalDate::parse_iso("2026-12-20").unwrap())
        );

        let doc: WorthQueryOwnedApplicationHistoryDocument<DaymarkSchema, DaymarkHistory> = home
            .owned_history::<DaymarkHistory>(&me, ReadProjectTasks { project: 1 })
            .unwrap();
        let close_scope = home.session(&me).fresh_scope();
        home.primary_graph()
            .with_application_advancement(&close_scope, |phase| doc.close(&phase))
            .unwrap()
            .unwrap();
    }
    drop(home);
}
