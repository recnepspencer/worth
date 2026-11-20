//! A World occurrence made durable before its live head moved readmits that
//! movement exactly once, with no recovered original left behind.
use super::*;

#[test]
fn killed_after_a_durable_occurrence_readmits_its_movement_exactly_once() {
    let directory = tempfile::tempdir().unwrap();
    let location = directory.path().join("home");
    let markers = directory.path().join("markers");
    std::fs::create_dir(&markers).unwrap();
    assert_eq!(
        crate::killed_child::run_and_kill(
            CHILD,
            &[
                (LOCATION, location.as_path()),
                (MARKERS, markers.as_path()),
                (OCCURRENCE, std::path::Path::new("occurrence")),
            ],
            &markers.join("ready"),
            &markers.join("finished")
        ),
        crate::killed_child::ChildEnd::KilledAtPark
    );
    let original = std::fs::read_to_string(markers.join("original")).unwrap();
    let fields: Vec<_> = original.lines().collect();
    let number = |index: usize| fields[index].parse::<u64>().unwrap();
    let request = crate::workload::serial_request();
    let execution = ExecutionRequest::serial(&request);
    for reopen in 0..2 {
        let fixture = open(&location, &execution);
        let world = fixture.home.world();
        let inspection = world.inspection_port();
        assert!(
            inspection
                .recovery_page(None, maximum())
                .unwrap()
                .rows()
                .is_empty(),
            "reopen {reopen}: the acknowledged occurrence leaves no original custody"
        );
        assert_eq!(
            inspection.history_snapshot().unwrap().installed_commits(),
            2
        );
        let head = world
            .observation_port()
            .observe_root_product_branch()
            .unwrap();
        assert_eq!(head.reference_generation().get(), 1);
        assert_eq!(head.selected_commit().ordinal(), number(1));
        assert_eq!(head.branch_identity().name().as_str(), fields[5]);
        assert_eq!(
            fixture
                .home
                .relational()
                .runtime()
                .history()
                .immutable_commit_count(),
            1
        );
        assert_eq!(
            std::fs::read_to_string(markers.join("handler-runs")).unwrap(),
            "1"
        );
        drop(head);
        close(fixture.home, &execution);
    }
}
