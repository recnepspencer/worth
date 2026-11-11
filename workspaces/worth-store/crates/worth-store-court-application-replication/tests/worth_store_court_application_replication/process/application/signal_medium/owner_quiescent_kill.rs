//! Kill only after the owner reports terminal publication and no callback is running.
use super::{medium_fixture as media, owner_fixture as fixture};
use std::{
    io::{BufRead, Write},
    path::Path,
    process::{Child, Stdio},
    time::Duration,
};
use worth_execution::ExecutionRequest;
use worth_signal::facade::durability::*;
use worth_signal::facade::*;

const CHILD_ROOT: &str = "WORTH_SIGNAL_QUIESCENT_OWNER_ROOT";
const QUIESCENT_ACK: &str = "SIGNAL_ORIGINAL_ADVANCES_TERMINAL";
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
    let (_serving, medium) = media::open(root, true, &execution);
    let reference = fixture::seed(medium.as_ref(), &execution);
    let owner = fixture::readmit(medium.clone(), &execution);
    for (branch, _) in owner.source_port().branches().unwrap() {
        let port = owner.source_port();
        let descriptor = port.observe(branch).unwrap();
        let handle = port.readmit_descriptor(&descriptor).unwrap();
        let task = owner
            .mutation_port()
            .begin_advance(execution, &handle, &mut (), |tx| {
                tx.target(reference.node).read(
                    &|_: &mut EvaluationContext<'_, ()>| -> Result<AspectVersion, SignalError> {
                        Ok(AspectVersion::from_updates([(Aspect::new(0), 71)]))
                    },
                )?;
                Ok(())
            })
            .unwrap();
        drop(fixture::finish(task, &execution));
    }
    assert!(matches!(
        medium.publication_task_posture(),
        SignalPublicationTaskPosture::Terminal
    ));
    println!("{QUIESCENT_ACK}");
    std::io::stdout().flush().unwrap();
    // Keep the live owner and Store serving share until the parent kills us.
    loop {
        std::thread::park_timeout(Duration::from_millis(50));
    }
}
#[test]
fn quiescent_kill_reopens_all_branch_sources_and_recomputes_values() {
    if let Some(root) = std::env::var_os(CHILD_ROOT) {
        child(Path::new(&root));
    }
    let directory = tempfile::tempdir().unwrap();
    let mut process = LiveChild(std::process::Command::new(std::env::current_exe().unwrap())
        .args(["--exact", "process::application::signal_medium::owner_quiescent_kill::quiescent_kill_reopens_all_branch_sources_and_recomputes_values", "--nocapture"])
        .env(CHILD_ROOT, directory.path())
        .stdout(Stdio::piped())
        .stderr(Stdio::inherit())
        .spawn().unwrap());
    let stdout = process.0.stdout.take().unwrap();
    let (acknowledged, observed) = std::sync::mpsc::sync_channel(1);
    let reader = std::thread::spawn(move || {
        for line in std::io::BufReader::new(stdout).lines() {
            let line = line.unwrap();
            println!("writer: {line}");
            if line.contains(QUIESCENT_ACK) {
                acknowledged.send(()).unwrap();
            }
        }
    });
    observed
        .recv_timeout(Duration::from_secs(120))
        .expect("live owner must acknowledge terminal original advances before kill");
    process.0.kill().unwrap();
    assert!(!process.0.wait().unwrap().success());
    reader.join().unwrap();
    let funding = media::serial_request();
    let execution = ExecutionRequest::serial(&funding);
    let reference = fixture::memory_only(&execution);
    let (serving, medium) = media::open_with_identity(
        directory.path(),
        false,
        2,
        "22222222-2222-4222-8222-222222222222",
        &execution,
    );
    let owner = fixture::readmit(medium.clone(), &execution);
    let calls = std::sync::atomic::AtomicUsize::new(0);
    for (branch, expected) in &reference.heads {
        let port = owner.source_port();
        let descriptor = port.observe(*branch).unwrap();
        assert_eq!(
            descriptor.generation().ordinal(),
            expected.generation().ordinal() + 1
        );
        let handle = port.readmit_descriptor(&descriptor).unwrap();
        let (graph, runtime) = port.capture_authored_images(&handle, &execution).unwrap();
        assert_eq!(graph.as_bytes(), reference.images[branch].0);
        assert_eq!(runtime.as_bytes(), reference.images[branch].1);
        let task = owner
            .mutation_port()
            .begin_advance(execution, &handle, &mut (), |tx| {
                let value =
                    tx.target(reference.node).read(
                        &|_: &mut EvaluationContext<'_, ()>| -> Result<AspectVersion, SignalError> {
                            calls.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
                            Ok(AspectVersion::from_updates([(Aspect::new(0), 71)]))
                        },
                    )?;
                assert_eq!(value, AspectVersion::from_updates([(Aspect::new(0), 71)]));
                Ok(())
            })
            .unwrap();
        drop(fixture::finish(task, &execution));
    }
    assert_eq!(calls.load(std::sync::atomic::Ordering::Relaxed), 3);
    drop(owner);
    media::close(serving, medium, &execution);
}
