use crate::consumer_kit::{
    load_support_snapshot_terminal_json_document, project_support_snapshot,
    WorthQueryExternalSupportSnapshotTerminalJsonDocument, WorthQuerySupportSnapshotErrorKind,
    WorthQuerySupportSnapshotSchemaVersion,
};

use super::{
    hostile_terminal_document::HostileSupportSnapshotTerminalDocument, live_support_matrix,
};

#[test]
fn support_snapshot_load_denies_schema_version_boundary_with_typed_error() {
    let terminal_json_document = terminal_support_snapshot_json_document_with_mutation(|value| {
        value.replace_top_level_number("schema_version", 2);
    });

    let error = load_support_snapshot_terminal_json_document(
        &terminal_json_document,
        WorthQuerySupportSnapshotSchemaVersion::current(),
    )
    .expect_err("schema version boundary must be denied");

    assert_eq!(
        error.kind(),
        &WorthQuerySupportSnapshotErrorKind::SchemaVersionMismatch
    );
}

#[test]
fn support_snapshot_load_denies_row_digest_drift_with_typed_error() {
    let terminal_json_document = terminal_support_snapshot_json_document_with_mutation(|value| {
        value.replace_first_row_string("live_row_digest", "fake-live-row-digest");
    });

    let error = load_support_snapshot_terminal_json_document(
        &terminal_json_document,
        WorthQuerySupportSnapshotSchemaVersion::current(),
    )
    .expect_err("row digest mutation must change the snapshot digest");

    assert_eq!(
        error.kind(),
        &WorthQuerySupportSnapshotErrorKind::RowDigestMismatch
    );
}

#[test]
fn support_snapshot_load_denies_unknown_status_before_digest_acceptance() {
    let matrix = live_support_matrix();
    let snapshot = project_support_snapshot(&matrix).expect("fixture digest must fit");
    let terminal_json_document =
        terminal_support_snapshot_json_document_from_snapshot_with_mutation(&snapshot, |value| {
            value.replace_first_row_string("status", "maybe-supported");
        });

    let error = load_support_snapshot_terminal_json_document(
        &terminal_json_document,
        WorthQuerySupportSnapshotSchemaVersion::current(),
    )
    .expect_err("unknown support status must be denied semantically");

    assert_eq!(
        error.kind(),
        &WorthQuerySupportSnapshotErrorKind::InvalidSupportStatus
    );
    assert_eq!(error.found(), Some("maybe-supported"));
    assert_eq!(error.surface(), Some(snapshot.rows()[0].surface()));
}

#[test]
fn support_snapshot_load_denies_unknown_backend_posture_with_structured_context() {
    let terminal_json_document = terminal_support_snapshot_json_document_with_mutation(|value| {
        value.replace_top_level_string("backend_posture", "ambient-runtime");
    });

    let error = load_support_snapshot_terminal_json_document(
        &terminal_json_document,
        WorthQuerySupportSnapshotSchemaVersion::current(),
    )
    .expect_err("unknown backend posture must be denied semantically");

    assert_eq!(
        error.kind(),
        &WorthQuerySupportSnapshotErrorKind::InvalidBackendPosture
    );
    assert_eq!(error.expected(), Some("primary|scaffold"));
    assert_eq!(error.found(), Some("ambient-runtime"));
}

#[test]
fn support_snapshot_load_denies_unknown_facade_family() {
    let terminal_json_document = terminal_support_snapshot_json_document_with_mutation(|value| {
        value.replace_first_row_string("facade_family", "side-channel");
    });

    let error = load_support_snapshot_terminal_json_document(
        &terminal_json_document,
        WorthQuerySupportSnapshotSchemaVersion::current(),
    )
    .expect_err("unknown facade family must be denied semantically");

    assert_eq!(
        error.kind(),
        &WorthQuerySupportSnapshotErrorKind::InvalidFacadeFamily
    );
    assert_eq!(error.found(), Some("side-channel"));
}

#[test]
fn support_snapshot_load_denies_unknown_teaching_posture() {
    let terminal_json_document = terminal_support_snapshot_json_document_with_mutation(|value| {
        value.replace_first_row_string("teaching_posture", "ambient-teaching");
    });

    let error = load_support_snapshot_terminal_json_document(
        &terminal_json_document,
        WorthQuerySupportSnapshotSchemaVersion::current(),
    )
    .expect_err("unknown teaching posture must be denied semantically");

    assert_eq!(
        error.kind(),
        &WorthQuerySupportSnapshotErrorKind::InvalidTeachingPosture
    );
    assert_eq!(error.found(), Some("ambient-teaching"));
}

#[test]
fn support_snapshot_load_denies_blank_required_field() {
    let terminal_json_document = terminal_support_snapshot_json_document_with_mutation(|value| {
        value.replace_first_row_string("owner_milestone", " ");
    });

    let error = load_support_snapshot_terminal_json_document(
        &terminal_json_document,
        WorthQuerySupportSnapshotSchemaVersion::current(),
    )
    .expect_err("blank required field must be denied semantically");

    assert_eq!(
        error.kind(),
        &WorthQuerySupportSnapshotErrorKind::InvalidRequiredField
    );
    assert_eq!(error.found(), Some("owner_milestone"));
}

#[test]
fn support_snapshot_load_denies_malformed_json_with_typed_error() {
    let error = load_support_snapshot_terminal_json_document(
        &WorthQueryExternalSupportSnapshotTerminalJsonDocument::from_external_terminal_json_document(
            "{not-json",
        ),
        WorthQuerySupportSnapshotSchemaVersion::current(),
    )
    .expect_err("malformed JSON must be denied");

    assert_eq!(
        error.kind(),
        &WorthQuerySupportSnapshotErrorKind::JsonDecodeFailed
    );
}

#[test]
fn support_snapshot_load_denies_document_digest_drift() {
    let terminal_json_document = terminal_support_snapshot_json_document_with_mutation(|value| {
        value.replace_top_level_string("snapshot_digest", "fake-document-digest");
    });

    let error = load_support_snapshot_terminal_json_document(
        &terminal_json_document,
        WorthQuerySupportSnapshotSchemaVersion::current(),
    )
    .expect_err("document digest mutation must be denied");

    assert_eq!(
        error.kind(),
        &WorthQuerySupportSnapshotErrorKind::SnapshotDigestMismatch
    );
}

fn terminal_support_snapshot_json_document_with_mutation(
    mutate: impl FnOnce(&mut HostileSupportSnapshotTerminalDocument),
) -> WorthQueryExternalSupportSnapshotTerminalJsonDocument {
    let mut document = HostileSupportSnapshotTerminalDocument::from_live_support_matrix();
    mutate(&mut document);
    document.into_external_terminal_json_document()
}

fn terminal_support_snapshot_json_document_from_snapshot_with_mutation(
    snapshot: &crate::consumer_kit::WorthQuerySupportSnapshot,
    mutate: impl FnOnce(&mut HostileSupportSnapshotTerminalDocument),
) -> WorthQueryExternalSupportSnapshotTerminalJsonDocument {
    let mut document = HostileSupportSnapshotTerminalDocument::from_snapshot(snapshot);
    mutate(&mut document);
    document.into_external_terminal_json_document()
}
