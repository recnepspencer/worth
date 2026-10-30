//! Open identity, incarnation, pre-effect refusals and independent OS contention.
pub(crate) use worth_execution::ExecutionRequest;
pub(crate) use {crate::durable_run::open, crate::workload};
pub(crate) use {std::fs, std::path::Path, std::process::Command};
pub(crate) use {
    worth_store_relational::RelationalStoreHomeOpenRefusal as Refusal,
    worth_store_relational::RelationalStorePhysicalRefusalKind as Kind,
};

pub(crate) fn assert_contended(location: &Path) {
    let output = std::process::Command::new(std::env::current_exe().unwrap())
        .args([
            "--exact",
            "process::relational::relational_home::incarnation::contention_observer_child",
            "--nocapture",
        ])
        .env("WORTH_INCARNATION_OBSERVE", location)
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "observer: {} {}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
}
