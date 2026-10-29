//! A key committed in one process resolves the same way in another process
//! that restores the capture, even when the two processes mint the same
//! runtime ordinals. Each phase runs in its own child process, so every
//! process-wide counter starts over.

pub use std::path::{Path, PathBuf};
pub use std::process::Command;
pub use worth_query_host::facade::application_installation::WorthQueryCheckpointCapturePolicy as CapturePolicy;

pub use worth_query_host::facade::application_entry::WorthQueryApplicationMutationOutcome;
pub use worth_query_host::facade::application_installation::{
    WorthQueryApplicationCheckpoint, WorthQueryApplicationProgramRoster,
};
pub use worth_relational::facade::history::CommitId;

pub use super::{assert_the_key_resolves_by_its_durable_record, KEY, RETENTION};
pub use worth_query_testkit_document_retention::facade::document_retention_model::host::{
    publish_on_first_program, restore_on_first_program, DocumentRetentionRuntime,
};
pub use worth_query_testkit_document_retention::facade::document_retention_model::presented_request::set_retention;
pub use worth_query_testkit_document_retention::facade::document_retention_model::programs::{
    validated_second_program, RetentionProgramP0,
};

pub const PHASE_DIRECTORY: &str = "WORTH_QUERY_RESTORED_REPLAY_DIRECTORY";

pub fn run_phase(phase: &str, directory: &Path) {
    let test = format!("{}::{phase}", own_test_module());
    let output = worth_query_testkit_host::facade::host::certification_support::process_deadline::output(
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
pub fn own_test_module() -> String {
    let path = module_path!();
    format!(
        "process::{}",
        path.split_once("::facade::")
            .expect("world fixture namespace is known")
            .1
    )
}

pub fn phase_directory() -> Option<PathBuf> {
    std::env::var_os(PHASE_DIRECTORY).map(PathBuf::from)
}

pub fn binding_runtime_ordinal(host: &DocumentRetentionRuntime<RetentionProgramP0>) -> u64 {
    host.installed_schema().binding_identity().runtime_ordinal()
}
