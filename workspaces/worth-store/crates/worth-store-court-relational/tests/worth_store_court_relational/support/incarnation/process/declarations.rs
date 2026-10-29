//! Independent competing processes and kill/reopen incarnation continuity.
pub(crate) use {crate::killed_child};
pub(crate) use {std::process::Command, worth_execution::ExecutionRequest, worth_store_relational::RelationalStorePhysicalRefusalKind as Kind, std::path::Path, worth_store_relational::RelationalStoreHomeOpenRefusal as Refusal, crate::incarnation::assert_contended, std::fs, crate::durable_run::open, crate::workload};
pub(crate) use {std::path::PathBuf, std::process::Child, std::process::Stdio, std::thread, std::time::Instant};

pub(crate) fn child(location: &Path, markers: &Path, start: &Path) -> Child {
    std::process::Command::new(std::env::current_exe().unwrap())
        .args([
            "--exact",
            "incarnation::process::competing_open_child",
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