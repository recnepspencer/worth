//! Real termination-task custody; child admission observes the independent OS lease.
use std::{
    path::Path,
    sync::mpsc,
    time::{Duration, Instant},
};
use tempfile::tempdir;
use worth_store::physical_runtime::certification::CertificationPhysicalClosePauseGate;
use worth_store::physical_runtime::{
    FilesystemMediaAdmission, PhysicalRuntimeAdmission, PhysicalStore, PhysicalStoreCloseOutcome,
    PhysicalStoreClosePhase, PhysicalStoreCloseTask, PhysicalStoreCloseTaskPoll,
};
use worth_store_testkit_store::physical_record_journeys::physical_work::fixture::{
    runtime::{serving_from_initialization_with_work_profile, serving_from_open_with_work_profile},
    semantic_work::work_fixture,
};

const PROBE_ROOT: &str = "WORTH_PHYSICAL_CLOSE_TASK_PROBE_ROOT";
const PROBE_TEST: &str = "process::store::physical_record_journeys::physical_work::close_plan::task::contention_observer_child";

struct HeldCloseGate(CertificationPhysicalClosePauseGate);

impl HeldCloseGate {
    fn release(&self) {
        self.0.release();
    }
}

impl Drop for HeldCloseGate {
    fn drop(&mut self) {
        self.0.release();
    }
}

#[test]
fn contention_observer_child() {
    let Some(root) = std::env::var_os(PROBE_ROOT) else {
        return;
    };
    let runtime = PhysicalStore::admit(PhysicalRuntimeAdmission::new(root).unwrap()).unwrap();
    let opened = runtime.try_admit_filesystem_media(FilesystemMediaAdmission::production(
        worth_store_physical_backend::FilesystemAccessPosture::CoordinatedServiceAccount,
    ));
    match opened.into_raw() {
        worth_proof::TransitionOutcome::Deferred(refused) => {
            assert!(matches!(
                refused.reason(),
                worth_store_physical_backend::MediaQualificationDeferred::MutationOwnerContended { .. }
            ));
            refused.into_runtime().abort();
            println!("PHYSICAL_CLOSE_TASK_OS_CONTENDED");
        }
        _ => panic!("independent media admission must observe mutation ownership contention"),
    }
}

#[test]
fn pending_task_keeps_its_identity_and_lease_then_returns_the_actual_close() {
    let root = tempdir().unwrap();
    let (profile, _, request) = work_fixture();
    let serving = serving_from_initialization_with_work_profile(root.path(), profile);
    let receipt = super::success(serving.physical_mutation_submission().submit(request));
    let identity = serving.runtime_identity();
    let (task, gate) = paused_close(serving);
    assert_eq!(task.runtime_identity(), identity);
    let task = match task.poll() {
        PhysicalStoreCloseTaskPoll::Pending(task) => task,
        _ => panic!("actual phase gate must retain the unfinished task"),
    };
    assert_eq!(task.runtime_identity(), identity);
    let progress = task.observation().unwrap().clone();
    assert!(progress.reached(PhysicalStoreClosePhase::SignalDisposed));
    assert!(!progress.reached(PhysicalStoreClosePhase::ResidencyClosed));
    assert!(!progress.reached(PhysicalStoreClosePhase::MediaReleased));
    assert_os_contended(root.path());
    gate.release();
    let actual = finish(task);
    assert!(matches!(actual, PhysicalStoreCloseOutcome::Closed { .. }));
    assert_eq!(
        actual.shutdown().work().drain().cancelled_before_dispatch(),
        &[receipt.identity()]
    );
    assert_eq!(actual.phases().len(), 7);
    assert!(progress.reached(PhysicalStoreClosePhase::MediaReleased));
    let reopened = serving_from_open_with_work_profile(root.path(), work_fixture().0);
    assert_ne!(reopened.runtime_identity(), identity);
    let _ = reopened.close();
}

#[test]
fn abandoning_observation_keeps_the_accepted_close_until_real_media_release() {
    let root = tempdir().unwrap();
    let serving = serving_from_initialization_with_work_profile(root.path(), work_fixture().0);
    let (task, gate) = paused_close(serving);
    let progress = task.observation().unwrap().clone();
    drop(task);
    // Dropping observer custody neither joins this held worker nor drops Store.
    assert!(!progress.reached(PhysicalStoreClosePhase::MediaReleased));
    assert_os_contended(root.path());
    gate.release();
    let deadline = Instant::now() + Duration::from_secs(5);
    while !progress.reached(PhysicalStoreClosePhase::MediaReleased) {
        assert!(
            Instant::now() < deadline,
            "accepted close did not release media"
        );
        std::thread::sleep(Duration::from_millis(1));
    }
    // No outcome was observed: successful actual reopen proves availability,
    // rather than treating the dropped handle or progress as a Closed outcome.
    let reopened = serving_from_open_with_work_profile(root.path(), work_fixture().0);
    let _ = reopened.close();
}

#[test]
fn ordinary_task_start_delivers_the_existing_terminal_outcome() {
    let root = tempdir().unwrap();
    let serving = serving_from_initialization_with_work_profile(root.path(), work_fixture().0);
    let task = PhysicalStoreCloseTask::start(serving)
        .unwrap_or_else(|refused| panic!("termination task start refused: {}", refused.error()));
    assert!(matches!(
        finish(task),
        PhysicalStoreCloseOutcome::Closed { .. }
    ));
}

fn paused_close(
    serving: worth_store::physical_runtime::ServingPhysicalRuntime,
) -> (PhysicalStoreCloseTask, HeldCloseGate) {
    let (sender, receiver) = mpsc::sync_channel(1);
    let task = PhysicalStoreCloseTask::start_with_plan_observer_for_test(serving, move |plan| {
        let gate = plan.certification_pause_at(PhysicalStoreClosePhase::SignalDisposed);
        sender.send(gate).unwrap();
    })
    .unwrap_or_else(|refused| panic!("termination task start refused: {}", refused.error()));
    let gate = HeldCloseGate(receiver.recv_timeout(Duration::from_secs(5)).unwrap());
    assert!(gate.0.await_arrival());
    (task, gate)
}

fn finish(mut task: PhysicalStoreCloseTask) -> PhysicalStoreCloseOutcome {
    let deadline = Instant::now() + Duration::from_secs(5);
    loop {
        match task.poll() {
            PhysicalStoreCloseTaskPoll::Completed(actual) => return actual,
            PhysicalStoreCloseTaskPoll::Failed(failure) => {
                panic!("termination task failed: {:?}", failure.cause())
            }
            PhysicalStoreCloseTaskPoll::Pending(pending) => {
                task = pending;
                assert!(Instant::now() < deadline, "termination task did not finish");
                std::thread::sleep(Duration::from_millis(1));
            }
        }
    }
}

fn assert_os_contended(root: &Path) {
    let mut child = std::process::Command::new(std::env::current_exe().unwrap())
        .args(["--exact", PROBE_TEST, "--nocapture"])
        .env(PROBE_ROOT, root)
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .spawn()
        .unwrap();
    let deadline = Instant::now() + Duration::from_secs(5);
    loop {
        if child.try_wait().unwrap().is_some() {
            break;
        }
        if Instant::now() >= deadline {
            let _ = child.kill();
            let output = child.wait_with_output().unwrap();
            panic!(
                "independent ownership observer exceeded its deadline: {} {}",
                String::from_utf8_lossy(&output.stdout),
                String::from_utf8_lossy(&output.stderr)
            );
        }
        std::thread::sleep(Duration::from_millis(1));
    }
    let output = child.wait_with_output().unwrap();
    assert!(
        output.status.success(),
        "child failed: {} {}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(String::from_utf8_lossy(&output.stdout).contains("PHYSICAL_CLOSE_TASK_OS_CONTENDED"));
}
