//! Cold serving and interrupted cold readmission use the existing kill driver.
#[allow(unused_imports)]
use crate::residency_process::*;

#[test]
fn a_killed_home_stays_cold_and_an_interrupted_cold_restore_reopens_again() {
    let host = crate::workload::serial_request();
    let execution = worth_execution::ExecutionRequest::serial(&host);
    let fixture = crate::residency_fixture::Fixture::new(&execution);
    let directory = tempfile::tempdir().unwrap();
    let location = directory.path().join("home");
    let markers = directory.path().join("markers");
    std::fs::create_dir(&markers).unwrap();
    let launch = |role: &str| {
        assert_eq!(
            crate::killed_child::run_and_kill(
                CHILD,
                &[
                    (ROLE, &std::path::PathBuf::from(role)),
                    (LOCATION, &location),
                    (MARKERS, &markers)
                ],
                &markers.join("ready"),
                &markers.join("finished")
            ),
            crate::killed_child::ChildEnd::KilledAtPark
        );
        let stage = std::fs::read_to_string(markers.join("ready")).unwrap();
        std::fs::remove_file(markers.join("ready")).unwrap();
        stage
    };
    launch("idle");
    let home = crate::durable_run::open(&location, &execution).unwrap();
    assert_eq!(home.cold_storage_reads().member_reads, 0);
    assert_eq!(home.cold_storage_reads().commit_reads, 1);
    let head = home
        .certification_serving()
        .canonical_commits()
        .unwrap()
        .head()
        .unwrap()
        .unwrap()
        .position();
    assert_eq!(head, 96, "the selected head of the history-bearing fixture");
    fixture.assert_reads(&home, &execution);
    home.close();
    std::fs::write(markers.join("head-position"), head.to_string()).unwrap();
    let stage = launch("restore");
    assert_eq!(
        stage,
        format!("selected-head-payload {head}: commits=1 members=0")
    );
    eprintln!("killed inside cold readmission: {stage}");
    for _ in 0..2 {
        let home = crate::durable_run::open(&location, &execution).unwrap();
        fixture.assert_reads(&home, &execution);
        home.close();
    }
}

#[test]
fn residency_child() {
    let Some(role) = std::env::var_os(ROLE) else {
        return;
    };
    let location = std::path::PathBuf::from(std::env::var_os(LOCATION).unwrap());
    let markers = std::path::PathBuf::from(std::env::var_os(MARKERS).unwrap());
    let host = crate::workload::serial_request();
    let execution = worth_execution::ExecutionRequest::serial(&host);
    if role == "idle" {
        let fixture = crate::residency_fixture::Fixture::new(&execution);
        let home = crate::durable_run::open(&location, &execution).unwrap();
        crate::durable_run::apply(&home, &execution, &fixture.steps);
        assert_eq!(
            crate::workload::artifacts_of(home.runtime()),
            crate::workload::artifacts_of(&fixture.reference)
        );
        crate::killed_child::announce(&markers.join("ready"), b"acknowledged checkpoint");
        crate::killed_child::park();
    }
    assert_eq!(role, "restore");
    let head = std::fs::read_to_string(markers.join("head-position"))
        .unwrap()
        .parse::<u64>()
        .unwrap();
    let opened =
        worth_store_relational::RelationalStoreHome::certification_open_with_readmission_observer(
            worth_store_relational::RelationalStoreHomeRequest::new(
                &location,
                crate::workload::runtime_builder(crate::schema_transition::SOURCE),
            ),
            move |position, reads| {
                assert_eq!(position.0, head, "readmission requests the selected head");
                assert_eq!(reads.commit_reads, 1, "first cold-port journal payload");
                assert_eq!(
                    reads.member_reads, 0,
                    "readmission has not demanded regions"
                );
                crate::killed_child::announce(
                    &markers.join("ready"),
                    format!(
                        "selected-head-payload {}: commits={} members={}",
                        position.0, reads.commit_reads, reads.member_reads
                    )
                    .as_bytes(),
                );
                crate::killed_child::park();
            },
            &execution,
        );
    let completed = opened.map(worth_store_relational::RelationalStoreHome::close);
    panic!("cold restore completed without its selected-head pause: {completed:?}");
}
