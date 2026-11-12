//! The child acknowledges only terminal owner publications, then remains live.
use super::*;
use std::{
    io::{BufRead, Write},
    path::Path,
    process::{Child, Stdio},
    time::Duration,
};
const CHILD_ROOT: &str = "WORTH_BRIDGE_QUIESCENT_OWNER_ROOT";
const ACK: &str = "BRIDGE_SIGNAL_PUBLICATIONS_TERMINAL";
struct LiveChild(Child);
impl Drop for LiveChild {
    fn drop(&mut self) {
        let _ = self.0.kill();
        let _ = self.0.wait();
    }
}
fn child(root: &Path) -> ! {
    let funding = media::serial_request();
    let execution = ExecutionRequest::serial(&funding);
    let reference = source::reference(&execution);
    let (_serving, medium) = media::open(root, true, &execution);
    source::seed(medium.as_ref(), &reference, &execution);
    let owner = readmit(medium.clone(), &execution);
    let calls = AtomicUsize::new(0);
    demand(
        &owner,
        reference.program.value,
        &execution,
        &calls,
        derived_reference::value(&execution),
    );
    assert_eq!(calls.load(Ordering::Relaxed), 3);
    assert_sources(&owner, &reference, &execution, 1);
    assert!(matches!(
        medium.publication_task_posture(),
        SignalPublicationTaskPosture::Terminal
    ));
    println!("{ACK}");
    std::io::stdout().flush().unwrap();
    loop {
        std::thread::park_timeout(Duration::from_millis(50));
    }
}
#[test]
fn quiescent_kill_rebuilds_bridge_definitions_and_cold_values_from_store() {
    if let Some(root) = std::env::var_os(CHILD_ROOT) {
        child(Path::new(&root));
    }
    let directory = tempfile::tempdir().unwrap();
    let mut process = LiveChild(std::process::Command::new(std::env::current_exe().unwrap())
        .args(["--exact", "process::application::signal_medium::bridge_owner::quiescent_kill::quiescent_kill_rebuilds_bridge_definitions_and_cold_values_from_store", "--nocapture"])
        .env(CHILD_ROOT, directory.path()).stdout(Stdio::piped()).stderr(Stdio::inherit()).spawn().unwrap());
    let stdout = process.0.stdout.take().unwrap();
    let (sender, observed) = std::sync::mpsc::sync_channel(1);
    let reader = std::thread::spawn(move || {
        for line in std::io::BufReader::new(stdout).lines() {
            let line = line.unwrap();
            println!("writer: {line}");
            if line.contains(ACK) {
                sender.send(()).unwrap();
            }
        }
    });
    observed
        .recv_timeout(Duration::from_secs(120))
        .expect("child must finish original publications before kill");
    process.0.kill().unwrap();
    assert!(!process.0.wait().unwrap().success());
    reader.join().unwrap();
    let funding = media::serial_request();
    let execution = ExecutionRequest::serial(&funding);
    let reference = source::reference(&execution);
    let (serving, medium) = media::open_with_identity(
        directory.path(),
        false,
        2,
        "22222222-2222-4222-8222-222222222222",
        &execution,
    );
    let owner = readmit(medium.clone(), &execution);
    assert_sources(&owner, &reference, &execution, 1);
    let calls = AtomicUsize::new(0);
    demand(
        &owner,
        reference.program.value,
        &execution,
        &calls,
        derived_reference::value(&execution),
    );
    assert_eq!(calls.load(Ordering::Relaxed), 3);
    drop(owner);
    media::close(serving, medium, &execution);
}
