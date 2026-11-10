//! Completed publication ends before a quiet kill or the coupled graceful close.
use super::*;
const END: &str = "WORTH_QUIESCENT_HOME_END";
const QUIET_CHILD: &str =
    "process::application::world_durable_release::unclean::quiescent::quiet_home_writer_child";
pub(super) enum End {
    Killed,
    Closed,
}

pub(super) fn journey(end: End) {
    let root = tempfile::tempdir().unwrap();
    let location = root.path().join("home");
    let markers = root.path().join("markers");
    std::fs::create_dir(&markers).unwrap();
    let (mode, expected) = match end {
        End::Killed => ("killed", crate::killed_child::ChildEnd::KilledAtPark),
        End::Closed => ("closed", crate::killed_child::ChildEnd::Finished),
    };
    assert_eq!(
        crate::killed_child::run_and_kill(
            QUIET_CHILD,
            &[
                (LOCATION, location.as_path()),
                (MARKERS, markers.as_path()),
                (END, std::path::Path::new(mode))
            ],
            &markers.join("ready"),
            &markers.join("finished"),
        ),
        expected
    );
    let request = crate::workload::serial_request();
    let execution = ExecutionRequest::serial(&request);
    let fixture = open(&location, &execution);
    let head = fixture
        .home
        .world()
        .observation_port()
        .observe_root_product_branch()
        .unwrap();
    assert_eq!(head.reference_generation().get(), 1);
    assert_eq!(
        fixture
            .home
            .world()
            .inspection_port()
            .history_snapshot()
            .unwrap()
            .installed_commits(),
        2
    );
    assert_eq!(
        fixture
            .home
            .relational()
            .runtime()
            .history()
            .immutable_commit_count(),
        1
    );
    assert!(fixture
        .home
        .world()
        .inspection_port()
        .recovery_page(None, maximum())
        .unwrap()
        .rows()
        .is_empty());
    drop(head);
    close(fixture.home, &execution);
}

#[test]
fn quiet_home_writer_child() {
    let Some(mode) = std::env::var_os(END) else {
        return;
    };
    let location = std::path::PathBuf::from(std::env::var_os(LOCATION).unwrap());
    let markers = std::path::PathBuf::from(std::env::var_os(MARKERS).unwrap());
    let request = crate::workload::serial_request();
    let execution = ExecutionRequest::serial(&request);
    let fixture = open(&location, &execution);
    let bootstrap = fixture.bootstrap_root(&execution);
    let expected = bootstrap.product_branch().clone();
    drop(bootstrap);
    let cancellation = RuntimeWorldCancellationSource::new();
    let prepared = fixture.prepare_publication(
        &execution,
        expected,
        crate::batches::create(0),
        &cancellation.token(),
    );
    let mut performed = fixture
        .publish(&execution, prepared, &cancellation.token())
        .consume();
    let successor = performed.take_successor_observation().unwrap();
    assert_eq!(successor.reference_generation().get(), 1);
    drop((successor, performed));
    match mode.to_str().unwrap() {
        "closed" => {
            close(fixture.home, &execution);
            crate::killed_child::announce(
                &markers.join("finished"),
                b"graceful coupled close complete",
            );
        }
        "killed" => {
            crate::killed_child::announce(
                &markers.join("ready"),
                b"quiescent after acknowledged occurrence",
            );
            crate::killed_child::park();
        }
        other => panic!("unknown child mode {other}"),
    }
}
