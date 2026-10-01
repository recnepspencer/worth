//! A syntactically valid, durably WAL-bound expiry cannot invent a later
//! completed checkpoint than the selected Store actually has.

use std::{fs, path::Path, process::Command, time::Duration};

use worth_store::physical_runtime::BlobResumeToken;

use super::{blob_crash, fixture::serving_from_open};

#[path = "blob_expiry_recovery_negative/wal.rs"]
mod wal;

#[test]
fn resealed_future_expiry_wal_blocks_before_any_recovery_effect() {
    let healthy = blob_crash::kill_at("crash-expiry-wal", Duration::from_secs(180));
    blob_crash::recover_closed_store(&healthy.root);
    let reopened = serving_from_open(&healthy.root);
    reopened.close();

    let mutant = blob_crash::kill_at("crash-expiry-wal", Duration::from_secs(180));
    let token =
        BlobResumeToken::decode(&fs::read(blob_crash::resume_token_path(&mutant.root)).unwrap())
            .expect("crashed writer persisted its authentic declaration token");
    let maximum = u64::from_le_bytes(token.encode()[108..116].try_into().unwrap());
    let records = mutant.root.join("families/records");
    let selected_before = fs::read(records.join("root-current.selector")).unwrap();
    let catalog_before = fs::read(records.join("bootstrap.catalog")).unwrap();
    let old_witness = wal::reseal_one_expiry_witness_after_selected_checkpoint(&mutant.root);
    assert_eq!(
        old_witness,
        maximum + 1,
        "only the completed crossing checkpoint exists"
    );
    assert_eq!(
        fs::read(records.join("root-current.selector")).unwrap(),
        selected_before
    );
    assert_eq!(
        fs::read(records.join("bootstrap.catalog")).unwrap(),
        catalog_before
    );

    let output = Command::new(recovery_executable())
        .arg(&mutant.root)
        .arg("--bounded-profile=c11-blob-crash-v1")
        .output()
        .unwrap();
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        !output.status.success(),
        "future witness unexpectedly recovered: {stderr}"
    );
    assert!(
        stderr.contains("C8_RECOVERY_BLOCKED kind=RedoPlanning")
            && stderr.contains("effects=0")
            && stderr.contains("planning_denial: None")
            && stderr.contains("canonical-redo-plan"),
        "admitted redo must reach expiry semantics then block pre-effect: {stderr}"
    );
    assert_eq!(
        fs::read(records.join("root-current.selector")).unwrap(),
        selected_before
    );
    assert_eq!(
        fs::read(records.join("bootstrap.catalog")).unwrap(),
        catalog_before
    );
}

fn recovery_executable() -> std::path::PathBuf {
    let binary = std::env::var_os("WORTH_C8_RECOVERY_EXECUTABLE")
        .map(std::path::PathBuf::from)
        .unwrap_or_else(|| {
            std::env::current_exe()
                .unwrap()
                .parent()
                .and_then(Path::parent)
                .unwrap()
                .join(format!(
                    "physical_store_recover{}",
                    std::env::consts::EXE_SUFFIX
                ))
        });
    assert!(
        binary.is_file(),
        "standalone C8 binary missing: {}",
        binary.display()
    );
    binary
}
