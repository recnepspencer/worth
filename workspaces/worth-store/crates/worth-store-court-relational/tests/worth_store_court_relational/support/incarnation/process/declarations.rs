//! Independent competing processes and kill/reopen incarnation continuity.
pub(crate) use crate::killed_child;
pub(crate) use {
    crate::durable_run::open, crate::incarnation::assert_contended, crate::workload, std::fs,
    std::path::Path, std::process::Command, worth_execution::ExecutionRequest,
    worth_store_relational::RelationalStoreHomeOpenRefusal as Refusal,
    worth_store_relational::RelationalStorePhysicalRefusalKind as Kind,
};
pub(crate) use {
    std::path::PathBuf, std::process::Child, std::process::Stdio, std::thread, std::time::Instant,
};

pub(crate) fn child(location: &Path, markers: &Path, start: &Path) -> Child {
    std::process::Command::new(std::env::current_exe().unwrap())
        .args([
            "--exact",
            "process::relational::relational_home::incarnation::process::competing_open_child",
            "--nocapture",
        ])
        .env("WORTH_INCARNATION_LOCATION", location)
        .env("WORTH_INCARNATION_MARKERS", markers)
        .env("WORTH_INCARNATION_START", start)
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .spawn()
        .unwrap()
}
