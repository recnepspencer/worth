//! Independent competing processes and kill/reopen incarnation continuity.
pub(crate) use std::path::Path;
pub(crate) use std::process::Child;

pub(crate) fn child(location: &Path, markers: &Path, start: &Path) -> Child {
    std::process::Command::new(std::env::current_exe().unwrap())
        .args([
            "--exact",
            "process::relational::relational_home::incarnation::writable_ownership::competing_open_child",
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
