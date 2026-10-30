//! Candidate tag-7 bytes are not selected custody until namespace publication.

pub use worth_store::physical_runtime::production::PhysicalCheckpointStep;
pub use {
    crate::production_entry::certified_release_serving,
    crate::production_entry::release_reopen::released_world,
    crate::production_entry::release_reopen::selected_release_certificates,
    crate::production_entry::release_reopen::selected_release_certificates_from_bytes,
    crate::production_entry::run_entry, worth_proof::TransitionOutcome,
    worth_store::physical_runtime::PhysicalCheckpointDeadline,
    worth_store::physical_runtime::PhysicalCheckpointIdempotencyKey,
    worth_store::physical_runtime::PhysicalCheckpointRequest,
};
pub use {std::process::Command, std::process::Stdio};
pub use {std::time::Duration, std::time::Instant};

pub const CHILD_MARKER: &str = "WORTH_C8_RELEASE_CERTIFICATE_CRASH_MARKER";
pub const RECOVERY_CHILD_ROOT: &str = "WORTH_C8_RELEASE_CERTIFICATE_RECOVERY_ROOT";
pub const RECOVERY_CHILD_CHECKPOINT: &str = "WORTH_C8_RELEASE_CERTIFICATE_RECOVERY_CHECKPOINT";

pub fn run() {
    run_inner(false);
}

pub fn run_inner(require_successor_checkpoint: bool) {
    #[cfg(not(all()))]
    let _ = require_successor_checkpoint;
    let marker = tempfile::tempdir().expect("crash marker directory");
    let root_marker = marker.path().join("root.txt");
    let reached_marker = marker.path().join("candidate-certificate.reached");
    let mut child = std::process::Command::new(std::env::current_exe().expect("test executable"))
        .arg("--exact")
        .arg("process::recovery_runtime::production_entry::release_reopen::candidate_crash::candidate_certificate_child")
        .arg("--nocapture")
        .env(CHILD_MARKER, marker.path())
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::piped())
        .spawn()
        .expect("launch release-certificate writer process");
    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(90);
    while !reached_marker.exists() {
        if let Some(status) = child.try_wait().expect("poll release writer") {
            let output = child.wait_with_output().expect("release writer output");
            panic!(
                "release writer exited before candidate certificate: {status}; {}",
                String::from_utf8_lossy(&output.stderr)
            );
        }
        if std::time::Instant::now() >= deadline {
            child.kill().expect("kill timed-out release writer process");
            child.wait().expect("reap timed-out release writer process");
            panic!("candidate certificate pause timed out");
        }
        std::thread::sleep(std::time::Duration::from_millis(25));
    }
    let root = std::path::PathBuf::from(
        std::fs::read_to_string(&root_marker).expect("writer's exact root marker"),
    );
    let selected = root.join("families/checkpoint.current");
    let before = std::fs::read(&selected).expect("selected checkpoint before crash");
    let (_, before_accumulator) = selected_release_certificates_from_bytes(&before);
    assert!(before_accumulator.base().terminal());
    child.kill().expect("kill paused writer process");
    child.wait().expect("reap paused writer process");
    let after = std::fs::read(&selected).expect("selected checkpoint after crash");
    assert_eq!(
        after, before,
        "candidate certificate must not select a checkpoint"
    );
    let (_, after_accumulator) = selected_release_certificates_from_bytes(&after);
    assert_eq!(
        after_accumulator.base().tip(),
        before_accumulator.base().tip()
    );
    assert_eq!(
        after_accumulator.base().checkpoint(),
        before_accumulator.base().checkpoint()
    );
    let recovery = {
        let mut command = std::process::Command::new(std::env::current_exe().expect("test executable"));
        command
            .arg("--exact")
            .arg("process::recovery_runtime::production_entry::release_reopen::candidate_crash::candidate_certificate_serving_child")
            .arg("--nocapture")
            .env(RECOVERY_CHILD_ROOT, &root);
        if require_successor_checkpoint {
            command.env(RECOVERY_CHILD_CHECKPOINT, "1");
        }
        command.output()
    }
    .expect("launch independent recovery and Serving process");
    #[cfg(not(all()))]
    let recovery = run_entry(&root);
    assert!(
        recovery.status.success(),
        "selected predecessor custody must recover after candidate-only crash: {}",
        String::from_utf8_lossy(&recovery.stderr)
    );
    assert!(
        String::from_utf8_lossy(&recovery.stderr)
            .contains("C8_CANDIDATE_CRASH_SELECTED_PREDECESSOR_OPENED"),
        "fresh process must open Serving from selected predecessor, not candidate custody: {}",
        String::from_utf8_lossy(&recovery.stderr)
    );
    #[cfg(not(all()))]
    assert!(
        String::from_utf8_lossy(&recovery.stderr).contains("C8_RECOVERY_RUNTIME"),
        "fresh process must recover selected predecessor, not candidate custody: {}",
        String::from_utf8_lossy(&recovery.stderr)
    );
    let selected_after_recovery =
        std::fs::read(&selected).expect("selected checkpoint after recovery attempt");
    if require_successor_checkpoint {
        let (_, successor) = selected_release_certificates_from_bytes(&selected_after_recovery);
        assert_ne!(
            selected_after_recovery, before,
            "a completed successor checkpoint must replace the selected predecessor"
        );
        assert_eq!(
            successor.base().prior_checkpoint_sequence(),
            before_accumulator.base().checkpoint().sequence().get(),
            "successor must ratchet from the selected predecessor, not the killed candidate"
        );
        assert_eq!(
            successor.base().checkpoint().sequence().get(),
            before_accumulator.base().checkpoint().sequence().get() + 1,
            "candidate residue cleanup must preserve the exact next sequence"
        );
        assert_eq!(successor.base().tip(), before_accumulator.base().tip());
        assert_eq!(
            successor.prior_head_count(),
            before_accumulator.head_count()
        );
        assert_eq!(
            successor.prior_head_roster_digest(),
            before_accumulator.head_roster_digest()
        );
    } else {
        assert_eq!(
            selected_after_recovery, before,
            "fresh recovery must not elevate candidate custody"
        );
    }
    // The child was killed before its TempDir destructor. Resolve and check
    // this one fixture root before removing only the child-created directory.
    let resolved = root.canonicalize().expect("fixture root remains present");
    assert!(resolved.starts_with(std::env::temp_dir().canonicalize().unwrap()));
    assert!(resolved
        .file_name()
        .unwrap()
        .to_string_lossy()
        .starts_with("worth-store-release-checkpoint-reopen-"));
    std::fs::remove_dir_all(resolved).expect("remove one child-owned fixture root");
}
