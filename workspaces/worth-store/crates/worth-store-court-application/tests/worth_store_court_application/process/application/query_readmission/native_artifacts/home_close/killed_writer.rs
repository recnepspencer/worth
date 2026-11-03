//! The parent kills a live production Host after acknowledged journal durability.
use super::*;
use std::{
    io::{BufRead, Write},
    process::{Child, Command, Stdio},
    sync::mpsc,
    time::Duration,
};
const PATH: &str = "WORTH_HOST_KILLED_WRITER_HOME";
const ACK: &str = "HOST_ACKNOWLEDGED_COMMIT=";
struct LiveChild(Child);
impl Drop for LiveChild {
    fn drop(&mut self) {
        let _ = self.0.kill();
        let _ = self.0.wait();
    }
}
#[test]
fn killed_host_without_close_reopens_its_acknowledged_value_and_original_idempotency_record() {
    if let Some(path) = std::env::var_os(PATH) {
        writer(std::path::Path::new(&path));
        return;
    }
    let path = tempfile::tempdir().unwrap();
    let test = format!("{}::killed_host_without_close_reopens_its_acknowledged_value_and_original_idempotency_record",
        module_path!().split_once("::").unwrap().1);
    let mut child = LiveChild(
        Command::new(std::env::current_exe().unwrap())
            .args(["--exact", &test, "--nocapture"])
            .env(PATH, path.path())
            .stdout(Stdio::piped())
            .stderr(Stdio::inherit())
            .spawn()
            .unwrap(),
    );
    let stdout = child.0.stdout.take().unwrap();
    let (acknowledged, observed) = mpsc::sync_channel(1);
    let reader = std::thread::spawn(move || {
        for line in std::io::BufReader::new(stdout).lines() {
            let line = line.unwrap();
            println!("writer: {line}");
            if let Some((_, commit)) = line.split_once(ACK) {
                acknowledged
                    .send(commit.trim().parse::<u64>().unwrap())
                    .unwrap();
            }
        }
    });
    let commit = observed
        .recv_timeout(Duration::from_secs(60))
        .expect("actual writer acknowledged before kill");
    child
        .0
        .kill()
        .expect("kill the still-live Host without close or destructors");
    assert!(!child.0.wait().unwrap().success());
    reader.join().unwrap();
    let live = serial(None, None);
    let execution = ExecutionRequest::serial(&live);
    let initialized = Arc::new(AtomicUsize::new(0));
    let contacts = Arc::new(AtomicUsize::new(0));
    let home = open(
        path.path(),
        execution,
        initialized.clone(),
        contacts.clone(),
    );
    assert_eq!(
        initialized.load(Ordering::SeqCst),
        0,
        "reopening never reinitializes the original Home"
    );
    assert_eq!(balance(home.runtime()), 8);
    match set(home.runtime(), 8) {
        WorthQueryApplicationMutationOutcome::PreviouslyCommitted(observed) => {
            assert_eq!(observed.commit_id().0, commit)
        }
        other => panic!("actual old key retains its acknowledged journal fact: {other:?}"),
    }
    assert_eq!(contacts.load(Ordering::SeqCst), 0);
    let closed = home
        .close(&execution)
        .unwrap_or_else(|refused| panic!("reopened Host close: {refused:?}"));
    assert_physical_lease_released(path.path(), &execution);
    closed.reclaim();
}
fn writer(path: &std::path::Path) {
    let live = serial(None, None);
    let execution = ExecutionRequest::serial(&live);
    let home = open(
        path,
        execution,
        Arc::new(AtomicUsize::new(0)),
        Arc::new(AtomicUsize::new(0)),
    );
    let commit = match set(home.runtime(), 8) {
        WorthQueryApplicationMutationOutcome::Committed { receipt, result: 8 } => {
            receipt.commit_reference().commit_id.0
        }
        other => panic!("writer must acknowledge a real journal commit: {other:?}"),
    };
    assert_eq!(balance(home.runtime()), 8);
    println!("{ACK}{commit}");
    std::io::stdout().flush().unwrap();
    // The original Home remains live. Parent termination never executes close.
    let _original_home = home;
    loop {
        std::thread::park();
    }
}
