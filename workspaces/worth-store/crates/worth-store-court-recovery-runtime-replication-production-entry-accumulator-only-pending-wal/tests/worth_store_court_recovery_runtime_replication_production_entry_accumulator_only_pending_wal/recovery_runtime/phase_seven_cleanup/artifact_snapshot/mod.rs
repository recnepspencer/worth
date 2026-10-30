#[allow(unused_imports)]
use crate::artifact_snapshot::process_context::*;

#[test]
fn an_omitted_preexisting_artifact_cannot_be_hidden_by_the_cleanup_oracle() {
    let path = std::path::PathBuf::from("families/wal").join(
        worth_store::physical_runtime::recovery_wal::WalSegmentArtifactIdentity::parse(
            "segment-1-generation-1.wal",
        )
        .unwrap()
        .file_name(),
    );
    let snapshot = std::collections::BTreeSet::from([path.clone()]);
    assert_eq!(
        missing_preexisting_path(&snapshot, &std::collections::BTreeMap::new()),
        Some(path)
    );
}

#[test]
fn an_omitted_recovery_created_artifact_cannot_be_hidden_by_the_cleanup_oracle() {
    let path = std::path::PathBuf::from("families/records/segments/segment-9-generation-2.data");
    assert_eq!(
        missing_created_path(
            &std::collections::BTreeSet::from([path.clone()]),
            &std::collections::BTreeMap::new()
        ),
        Some(path)
    );
}
