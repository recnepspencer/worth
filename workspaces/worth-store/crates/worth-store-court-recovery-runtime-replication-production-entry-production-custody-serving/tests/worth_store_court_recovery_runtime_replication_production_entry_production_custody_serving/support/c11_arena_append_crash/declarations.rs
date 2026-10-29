pub(crate) use {std::path::Path, std::path::PathBuf, std::process::Command, std::process::Stdio, std::thread, std::time::Duration, std::time::Instant};
pub(crate) use {worth_proof::TransitionOutcome};
pub(crate) use {worth_store::physical_runtime::certification::CertificationPhysicalMutationCheckpoint, worth_store::physical_runtime::PhysicalCheckpointDeadline, worth_store::physical_runtime::PhysicalCheckpointIdempotencyKey, worth_store::physical_runtime::PhysicalCheckpointOutcome, worth_store::physical_runtime::PhysicalCheckpointRequest};
pub(crate) use {worth_store_recovery_runtime::PhysicalRecoveryOutcome, worth_store_recovery_runtime::WorthStoreRecovery};

pub(crate) const CHILD: &str = "arena_append_after_data_child";
pub(crate) const MARKER: &str = "C11_ARENA_APPEND_CRASH_MARKER";

pub(crate) use {worth_store_testkit_recovery_runtime::support::EXTENT_BYTES, worth_store_testkit_recovery_runtime::support::append, worth_store_testkit_recovery_runtime::support::configuration, worth_store_testkit_recovery_runtime::support::generation, worth_store_testkit_recovery_runtime::support::initialize, worth_store_testkit_recovery_runtime::support::open, worth_store_testkit_recovery_runtime::support::recovery_request, worth_store_testkit_recovery_runtime::support::scan};

pub(crate) use {crate::c11_arena_append_crash::observation::observe_arena};

pub(crate) fn kill_after_data_settlement(directory: &Path) {
    let mut child = std::process::Command::new(std::env::current_exe().unwrap())
        .args(["--exact", CHILD, "--ignored", "--nocapture"])
        .env(MARKER, directory)
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .spawn()
        .unwrap();
    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(60);
    while !directory.join("reached").exists() {
        if child.try_wait().unwrap().is_some() || std::time::Instant::now() >= deadline {
            let _ = child.kill();
            let output = child.wait_with_output().unwrap();
            panic!(
                "data-settled kill seam not reached: {} {}",
                String::from_utf8_lossy(&output.stdout),
                String::from_utf8_lossy(&output.stderr)
            );
        }
        std::thread::sleep(std::time::Duration::from_millis(10));
    }
    child.kill().unwrap();
    child.wait().unwrap();
}