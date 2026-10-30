use worth_store_authority::ControlStoreGeneration;

use super::{ControlMediaLocation, PhysicalOperationalControlStore};

const CHILD_PATH: &str = "WORTH_STORE_CONTROL_PROCESS_PATH";
const CHILD_START: &str = "WORTH_STORE_CONTROL_PROCESS_START";
const CHILD_TRANSITION: &str = "WORTH_STORE_CONTROL_PROCESS_TRANSITION";

#[expect(
    clippy::disallowed_methods,
    reason = "category: coordinated storage-owner unit-test harness; purpose: current tail append serializes independent handles across threads; source: worth-store-physical-backend::operational_control::concurrent_append_tests::current_tail_append_serializes_independent_handles_across_threads; lexical counts: std::thread::spawn=1, std::thread::scope=0, std::thread::Scope::spawn=0, available_parallelism=0; barriers, joins and original bounds coordinate the storage-owner claim"
)]
#[test]
fn current_tail_append_serializes_independent_handles_across_threads() {
    let directory = tempfile::tempdir().expect("temp directory");
    let path = directory.path().join("control.log");
    PhysicalOperationalControlStore::open(ControlMediaLocation::new(&path)).expect("initialize");
    let barrier = std::sync::Arc::new(std::sync::Barrier::new(3));
    let mut workers = Vec::new();
    for index in 0..2 {
        let path = path.clone();
        let barrier = barrier.clone();
        workers.push(std::thread::spawn(move || {
            let store = PhysicalOperationalControlStore::open(ControlMediaLocation::new(path))
                .expect("worker open");
            barrier.wait();
            store
                .append_at_current_tail(&format!("worker-{index}"), b"payload")
                .expect("serialized append")
                .generation()
        }));
    }
    barrier.wait();
    let mut generations = workers
        .into_iter()
        .map(|worker| worker.join().expect("worker"))
        .collect::<Vec<_>>();
    generations.sort_by_key(|generation| generation.get());

    assert_eq!(generations[0], ControlStoreGeneration::initial());
    assert_eq!(generations[1].get(), 2);
}

#[test]
fn independent_processes_serialize_current_tail_appends() {
    if append_from_child_process() {
        return;
    }

    let directory = tempfile::tempdir().expect("temp directory");
    let path = directory.path().join("control.log");
    let start = directory.path().join("start");
    PhysicalOperationalControlStore::open(ControlMediaLocation::new(&path)).expect("initialize");
    let executable = std::env::current_exe().expect("test executable");
    let mut children = (0..2)
        .map(|index| spawn_control_writer(&executable, &path, &start, index))
        .collect::<Vec<_>>();
    std::fs::write(&start, b"go").expect("release process writers");
    for child in &mut children {
        assert!(child.wait().expect("control writer status").success());
    }

    let inspection = PhysicalOperationalControlStore::open(ControlMediaLocation::new(path))
        .expect("reopen")
        .inspect()
        .expect("inspect");
    assert!(inspection.damage().is_none());
    assert_eq!(inspection.records().len(), 2);
    assert_eq!(inspection.records()[0].generation().get(), 1);
    assert_eq!(inspection.records()[1].generation().get(), 2);
}

fn append_from_child_process() -> bool {
    let (Some(path), Some(start), Some(transition)) = (
        std::env::var_os(CHILD_PATH),
        std::env::var_os(CHILD_START),
        std::env::var_os(CHILD_TRANSITION),
    ) else {
        return false;
    };
    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(10);
    while !std::path::Path::new(&start).exists() {
        assert!(std::time::Instant::now() < deadline, "parent start barrier");
        std::thread::sleep(std::time::Duration::from_millis(1));
    }
    PhysicalOperationalControlStore::open(ControlMediaLocation::new(path))
        .expect("child open")
        .append_at_current_tail(&transition.to_string_lossy(), b"payload")
        .expect("child append");
    true
}

fn spawn_control_writer(
    executable: &std::path::Path,
    path: &std::path::Path,
    start: &std::path::Path,
    index: usize,
) -> std::process::Child {
    std::process::Command::new(executable)
        .arg("--exact")
        .arg(
            "operational_control::concurrent_append_tests::independent_processes_serialize_current_tail_appends",
        )
        .arg("--nocapture")
        .env(CHILD_PATH, path)
        .env(CHILD_START, start)
        .env(CHILD_TRANSITION, format!("process-{index}"))
        .spawn()
        .expect("spawn control writer")
}
