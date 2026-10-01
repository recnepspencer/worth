use super::*;
use worth_store::physical_runtime::RecordBootstrapDenial;

pub(crate) fn assert_plain_serving_denied(root: &Path) {
    let checkpoint_path = root.join("families/checkpoint.current");
    let before = std::fs::read(&checkpoint_path).expect("selected checkpoint before denied start");
    // A rooted release-head Store requires C8's independent custody rejoin
    // even for reads. A failed claim cannot fall back to ordinary Serving.
    assert!(open_serving_inner(
        root,
        None,
        Some(RecordBootstrapDenial::RecoveredCheckpointCustodyMismatch),
        false
    )
    .is_none());
    assert_eq!(
        std::fs::read(checkpoint_path).expect("selected checkpoint after denied start"),
        before,
        "plain Serving denial must leave selected checkpoint unchanged"
    );
}
