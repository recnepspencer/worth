//! Shared subprocess roles, mounted by each consuming court.
use super::*;

pub fn dispatch_recovery_interruption_child() {
    let Some(label) = std::env::var_os(RECOVERY_ROLE_ENV) else {
        return;
    };
    let stage = worth_store::physical_runtime::PhysicalRecoveryYieldpointStage::from_label(
        label.to_str().unwrap(),
    )
    .expect("recovery stage label");
    let root = std::path::PathBuf::from(std::env::var_os(RECOVERY_ROOT_ENV).unwrap());
    let markers = std::path::PathBuf::from(std::env::var_os(RECOVERY_MARKERS_ENV).unwrap());
    let yieldpoint = worth_store::physical_runtime::PhysicalRecoveryProcessYieldpoint::new(
        stage,
        markers.join("reached"),
        markers.join("release"),
        markers.join("cancel"),
    );
    let _ = worth_store_recovery_runtime::WorthStoreRecovery::recover_with_process_yieldpoint(
        crate::production_entry::certified_release_serving::recovery_request::request(&root),
        yieldpoint,
    );
    std::fs::write(markers.join("finished"), b"1").unwrap();
}
