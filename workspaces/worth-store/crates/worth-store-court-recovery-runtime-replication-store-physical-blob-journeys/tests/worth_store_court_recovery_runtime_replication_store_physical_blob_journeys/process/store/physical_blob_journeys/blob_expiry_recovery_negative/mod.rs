//! A syntactically valid, durably WAL-bound expiry cannot invent a later
//! completed checkpoint than the selected Store actually has.

#[allow(unused_imports)]
use crate::physical_blob_journeys::blob_expiry_recovery_negative::*;

use worth_store_testkit_store::fixture::serving_from_open;

#[test]
fn resealed_future_expiry_wal_blocks_before_any_recovery_effect() {
    let healthy = crate::physical_blob_journeys::blob_crash::kill_at(
        "crash-expiry-wal",
        std::time::Duration::from_secs(180),
    );
    crate::blob_crash::recovery::recover_closed_store(&healthy.root);
    let reopened = serving_from_open(&healthy.root);
    reopened.close();

    let mutant = crate::physical_blob_journeys::blob_crash::kill_at(
        "crash-expiry-wal",
        std::time::Duration::from_secs(180),
    );
    let token = worth_store::physical_runtime::BlobResumeToken::decode(
        &std::fs::read(crate::physical_blob_journeys::blob_crash::resume_token_path(&mutant.root))
            .unwrap(),
    )
    .expect("crashed writer persisted its authentic declaration token");
    let maximum = u64::from_le_bytes(token.encode()[108..116].try_into().unwrap());
    let records = mutant.root.join("families/records");
    let selected_before = std::fs::read(records.join("root-current.selector")).unwrap();
    let catalog_before = std::fs::read(records.join("bootstrap.catalog")).unwrap();
    let old_witness = crate::blob_expiry_recovery_negative::wal::reseal_one_expiry_witness_after_selected_checkpoint(&mutant.root);
    assert_eq!(
        old_witness,
        maximum + 1,
        "only the completed crossing checkpoint exists"
    );
    assert_eq!(
        std::fs::read(records.join("root-current.selector")).unwrap(),
        selected_before
    );
    assert_eq!(
        std::fs::read(records.join("bootstrap.catalog")).unwrap(),
        catalog_before
    );

    let output = std::process::Command::new(recovery_executable())
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
        stderr.contains("C8_RECOVERY_BLOCKED cause=Damage(RedoPlanning)")
            && stderr.contains("effects=0")
            && stderr.contains("planning_denial: None")
            && stderr.contains("canonical-redo-plan"),
        "admitted redo must reach expiry semantics then block pre-effect: {stderr}"
    );
    assert_eq!(
        std::fs::read(records.join("root-current.selector")).unwrap(),
        selected_before
    );
    assert_eq!(
        std::fs::read(records.join("bootstrap.catalog")).unwrap(),
        catalog_before
    );
}
