//! A key committed in one process resolves the same way in another process
//! that restores the capture, even when the two processes mint the same
//! runtime ordinals. Each phase runs in its own child process, so every
//! process-wide counter starts over.

use std::path::{Path, PathBuf};
use std::process::Command;
use worth_query_host::facade::application_installation::WorthQueryCheckpointCapturePolicy as CapturePolicy;

use worth_query_host::facade::application_entry::WorthQueryApplicationMutationOutcome;
use worth_query_host::facade::application_installation::{
    WorthQueryApplicationCheckpoint, WorthQueryApplicationProgramRoster,
};
use worth_relational::facade::history::CommitId;

use super::{assert_the_key_resolves_by_its_durable_record, KEY, RETENTION};
use crate::document_retention_model::host::{
    publish_on_first_program, restore_on_first_program, DocumentRetentionRuntime,
};
use crate::document_retention_model::presented_request::set_retention;
use crate::document_retention_model::programs::{validated_second_program, RetentionProgramP0};

const PHASE_DIRECTORY: &str = "WORTH_QUERY_RESTORED_REPLAY_DIRECTORY";

#[test]
fn a_key_committed_in_another_process_resolves_by_its_durable_record() {
    let directory = std::env::temp_dir().join(format!(
        "worth-query-restored-replay-{}",
        std::process::id()
    ));
    std::fs::create_dir_all(&directory).expect("the phase directory is created");
    run_phase("capture_phase", &directory);
    run_phase("restore_phase", &directory);
    std::fs::remove_dir_all(&directory).expect("the phase directory is removed");
}

#[test]
fn capture_phase() {
    let Some(directory) = phase_directory() else {
        return;
    };
    let host = publish_on_first_program();
    let WorthQueryApplicationMutationOutcome::Committed { receipt, .. } =
        set_retention(&host, host.current_world(), RETENTION, KEY)
            .expect("the first request settles")
    else {
        panic!("the first request commits");
    };
    let checkpoint = host
        .runtime()
        .capture_application_checkpoint(CapturePolicy::SystemAllocation)
        .expect("the host captures");
    std::fs::write(directory.join("checkpoint"), checkpoint.bytes()).expect("capture is written");
    let commit = receipt.committed_changes().commit_reference().commit_id.0;
    let ordinal = binding_runtime_ordinal(&host);
    std::fs::write(directory.join("record"), format!("{commit} {ordinal}"))
        .expect("the record is written");
}

#[test]
fn restore_phase() {
    let Some(directory) = phase_directory() else {
        return;
    };
    let bytes = std::fs::read(directory.join("checkpoint")).expect("capture is read");
    let record = std::fs::read_to_string(directory.join("record")).expect("the record is read");
    let [commit, ordinal] = record
        .split(' ')
        .map(|part| part.parse::<u64>().expect("the record holds numbers"))
        .collect::<Vec<_>>()[..]
    else {
        panic!("the record holds the commit and the ordinal");
    };
    let restored = restore_on_first_program(
        WorthQueryApplicationCheckpoint::from_untrusted_bytes(bytes),
        WorthQueryApplicationProgramRoster::new().support(validated_second_program()),
    )
    .expect("the host restores under the roster it installed");
    assert_eq!(
        binding_runtime_ordinal(&restored),
        ordinal,
        "a fresh process mints the capturing process's runtime ordinal again"
    );
    assert_the_key_resolves_by_its_durable_record(&restored, CommitId(commit));
}

fn run_phase(phase: &str, directory: &Path) {
    let test = format!("{}::{phase}", own_test_module());
    let output = crate::process_deadline::output(
        Command::new(std::env::current_exe().expect("the test binary is known"))
            .args([test.as_str(), "--exact", "--nocapture", "--test-threads=1"])
            .env(PHASE_DIRECTORY, directory),
    )
    .expect("the phase process runs");
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        output.status.success() && stdout.contains("1 passed"),
        "{phase} failed:\n{stdout}\n{}",
        String::from_utf8_lossy(&output.stderr),
    );
}

/// This module's path inside the test binary, without the binary's own name.
fn own_test_module() -> &'static str {
    let path = module_path!();
    path.split_once("::").map_or(path, |(_, module)| module)
}

fn phase_directory() -> Option<PathBuf> {
    std::env::var_os(PHASE_DIRECTORY).map(PathBuf::from)
}

fn binding_runtime_ordinal(host: &DocumentRetentionRuntime<RetentionProgramP0>) -> u64 {
    host.installed_schema().binding_identity().runtime_ordinal()
}
