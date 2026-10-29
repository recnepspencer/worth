//! Independent competing processes and kill/reopen incarnation continuity.
#[allow(unused_imports)]
use crate::incarnation::process::*;
use crate::killed_child;
use {
    crate::durable_run::open, crate::incarnation::assert_contended, crate::workload, std::fs,
    std::path::Path, std::process::Command, worth_execution::ExecutionRequest,
    worth_store_relational::RelationalStoreHomeOpenRefusal as Refusal,
    worth_store_relational::RelationalStorePhysicalRefusalKind as Kind,
};
use {
    std::path::PathBuf, std::process::Child, std::process::Stdio, std::thread, std::time::Instant,
};

#[test]
fn two_processes_open_empty_initialized_and_recovery_homes_with_one_writer() {
    let request = crate::workload::serial_request();
    let execution = worth_execution::ExecutionRequest::serial(&request);
    for state in ["empty", "initialized", "recovery"] {
        let parent = tempfile::tempdir().unwrap();
        let location = parent.path().join("home");
        let initial = if state == "empty" {
            0
        } else {
            let home = open(&location, &execution).unwrap();
            if state == "recovery" {
                let script = crate::workload::Script::generate(0x5eed_1202, 3, 3);
                crate::durable_run::apply(
                    &home,
                    &execution,
                    crate::durable_run::steps_before(&script, 2),
                );
                home.certification_abort();
            } else {
                home.close();
            }
            1
        };
        let first = parent.path().join("first");
        let second = parent.path().join("second");
        std::fs::create_dir(&first).unwrap();
        std::fs::create_dir(&second).unwrap();
        let start = parent.path().join("start");
        let mut children = [
            child(&location, &first, &start),
            child(&location, &second, &start),
        ];
        std::fs::write(&start, b"go").unwrap();
        let deadline = std::time::Instant::now() + crate::killed_child::PARK_DEADLINE;
        while !first.join("result").exists() || !second.join("result").exists() {
            if std::time::Instant::now() >= deadline {
                for child in &mut children {
                    let _ = child.kill();
                }
                panic!("{state}: competing opens never answered");
            }
            std::thread::sleep(std::time::Duration::from_millis(10));
        }
        let results = [
            std::fs::read_to_string(first.join("result")).unwrap(),
            std::fs::read_to_string(second.join("result")).unwrap(),
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
    let markers = std::path::PathBuf::from(std::env::var_os("WORTH_INCARNATION_MARKERS").unwrap());
    let start = std::path::PathBuf::from(std::env::var_os("WORTH_INCARNATION_START").unwrap());
    let deadline = std::time::Instant::now() + crate::killed_child::PARK_DEADLINE;
    while !start.exists() {
        assert!(std::time::Instant::now() < deadline);
        std::thread::sleep(std::time::Duration::from_millis(10));
    }
    let request = crate::workload::serial_request();
    let execution = worth_execution::ExecutionRequest::serial(&request);
    match open(std::path::Path::new(&location), &execution) {
        Ok(home) => {
            crate::killed_child::announce(
                &markers.join("result"),
                format!("serving:{}", home.incarnation()).as_bytes(),
            );
            crate::killed_child::park();
        }
        Err(worth_store_relational::RelationalStoreHomeOpenRefusal::Physical(refusal)) if refusal.kind() == worth_store_relational::RelationalStorePhysicalRefusalKind::MutationOwnerContended => {
            crate::killed_child::announce(&markers.join("result"), b"contended")
        }
        other => panic!("unexpected open outcome: {:?}", other.map(|_| ())),
    }
}

#[test]
fn incarnation_advance_killed_at_every_store_seam_advances_exactly_once_if_durable() {
    use worth_store::physical_runtime::certification::CertificationPhysicalMutationCheckpoint as Seam;
    let request = crate::workload::serial_request();
    let execution = worth_execution::ExecutionRequest::serial(&request);
    for seam in crate::process_kill::SEAMS {
        let parent = tempfile::tempdir().unwrap();
        let location = parent.path().join("home");
        let initial = open(&location, &execution).unwrap();
        let identity = initial.home_identity();
        initial.close();
        let markers = tempfile::tempdir().unwrap();
        let label = std::path::PathBuf::from(format!("{seam:?}"));
        let result = crate::killed_child::run_and_kill_observing(
            "process::relational::relational_home::incarnation::process::incarnation_advance_child",
            &[
                ("WORTH_INCARNATION_KILL_LOCATION", &location),
                ("WORTH_INCARNATION_KILL_MARKERS", markers.path()),
                ("WORTH_INCARNATION_KILL_SEAM", &label),
            ],
            &markers.path().join("ready"),
            &markers.path().join("finished"),
            || assert_contended(&location),
        );
        assert_eq!(result, crate::killed_child::ChildEnd::KilledAtPark);
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
    use {
        worth_store::physical_runtime::FilesystemAccessPosture,
        worth_store::physical_runtime::FilesystemMediaAdmission,
    };
    use {
        worth_store_relational::RelationalStoreHome,
        worth_store_relational::RelationalStoreHomeRequest,
    };
    let Some(location) = std::env::var_os("WORTH_INCARNATION_KILL_LOCATION") else {
        return;
    };
    let markers =
        std::path::PathBuf::from(std::env::var_os("WORTH_INCARNATION_KILL_MARKERS").unwrap());
    let wanted = std::env::var("WORTH_INCARNATION_KILL_SEAM").unwrap();
    let seam = crate::process_kill::SEAMS
        .into_iter()
        .find(|s| format!("{s:?}") == wanted)
        .unwrap();
    let (send, receive) = std::sync::mpsc::channel();
    std::thread::scope(|scope| {
        let worker = scope.spawn(|| {
            let request = crate::workload::serial_request();
            let execution = worth_execution::ExecutionRequest::serial(&request);
            RelationalStoreHome::certification_open_with_descriptor_observer(
                RelationalStoreHomeRequest::new(
                    std::path::PathBuf::from(location),
                    crate::workload::runtime_builder(crate::schema_transition::SOURCE),
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
        let gate = receive
            .recv_timeout(crate::killed_child::PARK_DEADLINE)
            .unwrap();
        let deadline = std::time::Instant::now() + crate::killed_child::PARK_DEADLINE;
        while !gate.await_arrival() {
            if worker.is_finished() || std::time::Instant::now() >= deadline {
                gate.release();
                panic!("incarnation never reached {seam:?}");
            }
        }
        crate::killed_child::announce(&markers.join("ready"), b"incarnation publication");
        crate::killed_child::park();
    });
}
