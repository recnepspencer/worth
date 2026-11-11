use crate::consumer_kit::project_support_snapshot;

use super::{
    hostile_terminal_document::HostileSupportSnapshotTerminalDocument, live_support_matrix,
};

#[test]
fn support_snapshot_export_is_deterministic_for_the_same_live_matrix() {
    let matrix = live_support_matrix();

    let left = project_support_snapshot(&matrix).expect("fixture digest must fit");
    let right = project_support_snapshot(&matrix).expect("fixture digest must fit");

    assert_eq!(left.snapshot_digest(), right.snapshot_digest());
    assert_eq!(
        left.to_canonical_terminal_json_document()
            .expect("left JSON should encode"),
        right
            .to_canonical_terminal_json_document()
            .expect("right JSON should encode")
    );
}

#[test]
fn support_snapshot_load_reexport_is_byte_stable() {
    let matrix = live_support_matrix();
    let snapshot = project_support_snapshot(&matrix).expect("fixture digest must fit");

    let original_terminal_json_document = snapshot
        .to_canonical_terminal_json_document()
        .expect("original canonical JSON should encode");
    let loaded = crate::consumer_kit::load_support_snapshot_terminal_json_document(
        &original_terminal_json_document.to_external_terminal_json_document(),
        crate::consumer_kit::WorthQuerySupportSnapshotSchemaVersion::current(),
    )
    .expect("canonical JSON should load");
    let reexported_terminal_json_document = loaded
        .to_canonical_terminal_json_document()
        .expect("loaded canonical JSON should encode");

    assert_eq!(
        original_terminal_json_document,
        reexported_terminal_json_document
    );
    assert_eq!(snapshot.snapshot_digest(), loaded.snapshot_digest());
}

#[test]
fn support_snapshot_digest_includes_schema_identity() {
    let matrix = live_support_matrix();
    let snapshot = project_support_snapshot(&matrix).expect("fixture digest must fit");
    let mut terminal_json_document =
        HostileSupportSnapshotTerminalDocument::from_snapshot(&snapshot);
    terminal_json_document.replace_top_level_string("schema_identity", "fake-schema");

    let error = crate::consumer_kit::load_support_snapshot_terminal_json_document(
        &terminal_json_document.into_external_terminal_json_document(),
        crate::consumer_kit::WorthQuerySupportSnapshotSchemaVersion::current(),
    )
    .expect_err("schema identity mutation should be denied");

    assert_eq!(
        error.kind(),
        &crate::consumer_kit::WorthQuerySupportSnapshotErrorKind::SchemaIdentityMismatch
    );
}
