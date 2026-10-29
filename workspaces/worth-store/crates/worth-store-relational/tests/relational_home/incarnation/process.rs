//! Independent competing processes and kill/reopen incarnation continuity.
use super::super::killed_child;
use super::*;
use std::{
    path::PathBuf,
    process::{Child, Stdio},
    thread,
    time::Instant,
};

fn child(location: &Path, markers: &Path, start: &Path) -> Child {
    Command::new(std::env::current_exe().unwrap())
        .args([
            "--exact",
            "incarnation::process::competing_open_child",
            "--nocapture",
        ])
        .env("WORTH_INCARNATION_LOCATION", location)
        .env("WORTH_INCARNATION_MARKERS", markers)
        .env("WORTH_INCARNATION_START", start)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap()
}

#[test]
fn two_processes_open_empty_initialized_and_recovery_homes_with_one_writer() {
    let request = workload::serial_request();
    let execution = ExecutionRequest::serial(&request);
    for state in ["empty", "initialized", "recovery"] {
        let parent = tempfile::tempdir().unwrap();
        let location = parent.path().join("home");
        let initial = if state == "empty" {
            0
        } else {
            let home = open(&location, &execution).unwrap();
            if state == "recovery" {
                let script = workload::Script::generate(0x5eed_1202, 3, 3);
                super::super::durable_run::apply(
                    &home,
                    &execution,
                    super::super::durable_run::steps_before(&script, 2),
                );
                home.certification_abort();
            } else {
                home.close();
            }
            1
        };
        let first = parent.path().join("first");
        let second = parent.path().join("second");
        fs::create_dir(&first).unwrap();
        fs::create_dir(&second).unwrap();
        let start = parent.path().join("start");
        let mut children = [
            child(&location, &first, &start),
            child(&location, &second, &start),
        ];
        fs::write(&start, b"go").unwrap();
        let deadline = Instant::now() + killed_child::PARK_DEADLINE;
        while !first.join("result").exists() || !second.join("result").exists() {
            if Instant::now() >= deadline {
                for child in &mut children {
                    let _ = child.kill();
                }
                panic!("{state}: competing opens never answered");
            }
            thread::sleep(std::time::Duration::from_millis(10));
        }
        let results = [
            fs::read_to_string(first.join("result")).unwrap(),
            fs::read_to_string(second.join("result")).unwrap(),
        ];
        assert_eq!(
            results.iter().filter(|r| r.as_str() == "contended").count(),
            1,
            "{state}: {results:?}"
        );
        let winner = results
            .iter()
            .position(|r| r.starts_with("serving:"))
            .unwrap();
        let incarnation: u64 = results[winner]
            .strip_prefix("serving:")
            .unwrap()
            .parse()
            .unwrap();
        assert_eq!(
            incarnation,
            initial + 1,
            "exactly one initializer/open advances"
        );
        assert_contended(&location);
        children[winner].kill().unwrap();
        children[winner].wait().unwrap();
        assert!(children[1 - winner].wait().unwrap().success());
        let recovered = open(&location, &execution).unwrap();
        assert_eq!(recovered.incarnation(), incarnation + 1);
        let identity = recovered.home_identity();
        recovered.close();
        let again = open(&location, &execution).unwrap();
        assert_eq!(again.home_identity(), identity);
        assert_eq!(again.incarnation(), incarnation + 2);
        again.close();
    }
}

#[test]
fn competing_open_child() {
    let Some(location) = std::env::var_os("WORTH_INCARNATION_LOCATION") else {
        return;
    };
    let markers = PathBuf::from(std::env::var_os("WORTH_INCARNATION_MARKERS").unwrap());
    let start = PathBuf::from(std::env::var_os("WORTH_INCARNATION_START").unwrap());
    let deadline = Instant::now() + killed_child::PARK_DEADLINE;
    while !start.exists() {
        assert!(Instant::now() < deadline);
        thread::sleep(std::time::Duration::from_millis(10));
    }
    let request = workload::serial_request();
    let execution = ExecutionRequest::serial(&request);
    match open(Path::new(&location), &execution) {
        Ok(home) => {
            killed_child::announce(
                &markers.join("result"),
                format!("serving:{}", home.incarnation()).as_bytes(),
            );
            killed_child::park();
        }
        Err(Refusal::Physical(refusal)) if refusal.kind() == Kind::MutationOwnerContended => {
            killed_child::announce(&markers.join("result"), b"contended")
        }
        other => panic!("unexpected open outcome: {:?}", other.map(|_| ())),
    }
}

#[test]
fn incarnation_advance_killed_at_every_store_seam_advances_exactly_once_if_durable() {
    use worth_store::physical_runtime::certification::CertificationPhysicalMutationCheckpoint as Seam;
    let request = workload::serial_request();
    let execution = ExecutionRequest::serial(&request);
    for seam in super::super::process_kill::SEAMS {
        let parent = tempfile::tempdir().unwrap();
        let location = parent.path().join("home");
        let initial = open(&location, &execution).unwrap();
        let identity = initial.home_identity();
        initial.close();
        let markers = tempfile::tempdir().unwrap();
        let label = PathBuf::from(format!("{seam:?}"));
        let result = killed_child::run_and_kill_observing(
            "incarnation::process::incarnation_advance_child",
            &[
                ("WORTH_INCARNATION_KILL_LOCATION", &location),
                ("WORTH_INCARNATION_KILL_MARKERS", markers.path()),
                ("WORTH_INCARNATION_KILL_SEAM", &label),
            ],
            &markers.path().join("ready"),
            &markers.path().join("finished"),
            || assert_contended(&location),
        );
        assert_eq!(result, killed_child::ChildEnd::KilledAtPark);
        let recovered = open(&location, &execution).unwrap();
        assert_eq!(recovered.home_identity(), identity);
        assert_eq!(
            recovered.incarnation(),
            if seam == Seam::BeforeWalAppend { 2 } else { 3 }
        );
        recovered.close();
    }
}

#[test]
fn incarnation_advance_child() {
    use worth_store::physical_runtime::{FilesystemAccessPosture, FilesystemMediaAdmission};
    use worth_store_relational::{RelationalStoreHome, RelationalStoreHomeRequest};
    let Some(location) = std::env::var_os("WORTH_INCARNATION_KILL_LOCATION") else {
        return;
    };
    let markers = PathBuf::from(std::env::var_os("WORTH_INCARNATION_KILL_MARKERS").unwrap());
    let wanted = std::env::var("WORTH_INCARNATION_KILL_SEAM").unwrap();
    let seam = super::super::process_kill::SEAMS
        .into_iter()
        .find(|s| format!("{s:?}") == wanted)
        .unwrap();
    let (send, receive) = std::sync::mpsc::channel();
    std::thread::scope(|scope| {
        let worker = scope.spawn(|| {
            let request = workload::serial_request();
            let execution = ExecutionRequest::serial(&request);
            RelationalStoreHome::certification_open_with_descriptor_observer(
                RelationalStoreHomeRequest::new(
                    PathBuf::from(location),
                    workload::runtime_builder(super::super::schema_transition::SOURCE),
                ),
                FilesystemMediaAdmission::production(
                    FilesystemAccessPosture::CoordinatedServiceAccount,
                ),
                |serving| {
                    send.send(serving.certification_pause_physical_mutation_at(seam))
                        .unwrap()
                },
                &execution,
            )
            .unwrap()
            .close();
        });
        let gate = receive.recv_timeout(killed_child::PARK_DEADLINE).unwrap();
        let deadline = Instant::now() + killed_child::PARK_DEADLINE;
        while !gate.await_arrival() {
            if worker.is_finished() || Instant::now() >= deadline {
                gate.release();
                panic!("incarnation never reached {seam:?}");
            }
        }
        killed_child::announce(&markers.join("ready"), b"incarnation publication");
        killed_child::park();
    });
}
