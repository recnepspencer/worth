//! Cold serving and interrupted cold readmission use the existing kill driver.
use super::{
    durable_run, killed_child, media_faults, residency_fixture::Fixture, schema_transition,
    workload,
};
use std::{fs, num::NonZeroU64, path::PathBuf, time::Duration};
use worth_execution::ExecutionRequest;
use worth_store::physical_runtime::{
    certification::{MediaFaultDirective, MediaOperationRole},
    FilesystemAccessPosture, FilesystemMediaAdmission,
};
use worth_store_relational::{RelationalStoreHome, RelationalStoreHomeRequest};

const ROLE: &str = "WORTH_RESIDENCY_CHILD";
const LOCATION: &str = "WORTH_RESIDENCY_LOCATION";
const MARKERS: &str = "WORTH_RESIDENCY_MARKERS";
const CHILD: &str = "residency_process::residency_child";

#[test]
fn a_killed_home_stays_cold_and_an_interrupted_cold_restore_reopens_again() {
    let host = workload::serial_request();
    let execution = ExecutionRequest::serial(&host);
    let fixture = Fixture::new(&execution);
    let directory = tempfile::tempdir().unwrap();
    let location = directory.path().join("home");
    let markers = directory.path().join("markers");
    fs::create_dir(&markers).unwrap();
    let launch = |role: &str| {
        assert_eq!(
            killed_child::run_and_kill(
                CHILD,
                &[
                    (ROLE, &PathBuf::from(role)),
                    (LOCATION, &location),
                    (MARKERS, &markers)
                ],
                &markers.join("ready"),
                &markers.join("finished")
            ),
            killed_child::ChildEnd::KilledAtPark
        );
        fs::remove_file(markers.join("ready")).unwrap();
    };
    launch("idle");
    // Measure a production open with the same media observers used by sweeps.
    // The last positioned read is the selected head's journal payload inside
    // native cold-checkpoint readmission; no payload members are read at open.
    let media = media_faults::FaultedMedia::admit(&media_faults::counters()).unwrap();
    media.arm();
    let (home, counters) = media.open(&location, schema_transition::SOURCE, &execution);
    let reads = media_faults::counts(&counters)
        .into_iter()
        .find(|(role, _)| *role == MediaOperationRole::PositionedRead)
        .unwrap()
        .1;
    assert!(reads > 0);
    fixture.assert_reads(&home, &execution);
    home.close();
    fs::write(markers.join("read-ordinal"), reads.to_string()).unwrap();
    launch("restore");
    for _ in 0..2 {
        let home = durable_run::open(&location, &execution).unwrap();
        fixture.assert_reads(&home, &execution);
        home.close();
    }
}

#[test]
fn residency_child() {
    let Some(role) = std::env::var_os(ROLE) else {
        return;
    };
    let location = PathBuf::from(std::env::var_os(LOCATION).unwrap());
    let markers = PathBuf::from(std::env::var_os(MARKERS).unwrap());
    let host = workload::serial_request();
    let execution = ExecutionRequest::serial(&host);
    if role == "idle" {
        let fixture = Fixture::new(&execution);
        let home = durable_run::open(&location, &execution).unwrap();
        durable_run::apply(&home, &execution, &fixture.steps);
        assert_eq!(
            workload::artifacts_of(home.runtime()),
            workload::artifacts_of(&fixture.reference)
        );
        killed_child::announce(&markers.join("ready"), b"acknowledged checkpoint");
        killed_child::park();
    }
    assert_eq!(role, "restore");
    let ordinal = fs::read_to_string(markers.join("read-ordinal"))
        .unwrap()
        .parse()
        .unwrap();
    let media =
        FilesystemMediaAdmission::certification(FilesystemAccessPosture::CoordinatedServiceAccount);
    let authority = media.fault_schedule_authority();
    let gate = authority.pause_gate();
    let activation = authority.one_shot_activation();
    let rule = authority
        .rule(
            MediaOperationRole::PositionedRead,
            1,
            MediaFaultDirective::PauseAfter(gate.clone()),
        )
        .for_nth_identified_operation_after_activation(
            activation.clone(),
            NonZeroU64::new(ordinal).unwrap(),
        );
    let media = media.with_fault_schedule(authority.schedule(vec![rule]).unwrap());
    activation.arm().unwrap();
    std::thread::scope(|scope| {
        let worker = scope.spawn(|| {
            let host = workload::serial_request();
            let execution = ExecutionRequest::serial(&host);
            RelationalStoreHome::certification_open_with_media(
                RelationalStoreHomeRequest::new(
                    &location,
                    workload::runtime_builder(schema_transition::SOURCE),
                ),
                media,
                &execution,
            )
        });
        while gate.reached_context().is_none() {
            assert!(
                !worker.is_finished(),
                "cold restore completed without its measured read"
            );
            std::thread::sleep(Duration::from_millis(10));
        }
        assert!(activation.is_consumed());
        killed_child::announce(&markers.join("ready"), b"cold readmission parked");
        killed_child::park();
    });
}
