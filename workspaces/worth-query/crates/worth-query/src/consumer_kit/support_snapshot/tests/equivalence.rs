use crate::consumer_kit::{
    load_support_snapshot_terminal_json_document, project_support_snapshot,
    WorthQuerySupportSnapshotSchemaVersion,
};

use super::live_support_matrix;

#[test]
fn support_snapshot_matches_live_matrix_row_for_row_and_digest_for_digest() {
    let matrix = live_support_matrix();
    let snapshot = project_support_snapshot(&matrix).expect("fixture digest must fit");

    assert_eq!(
        snapshot.source_matrix_digest(),
        matrix.matrix_digest().terminal_projection_for_reporting()
    );
    assert_eq!(snapshot.rows().len(), matrix.rows().len());

    for (snapshot_row, matrix_row) in snapshot.rows().iter().zip(matrix.rows().iter()) {
        assert_eq!(snapshot_row.surface(), matrix_row.surface());
        assert_eq!(
            snapshot_row.facade_family(),
            matrix_row.facade_family().map(|family| family.as_str())
        );
        assert_eq!(snapshot_row.status(), matrix_row.status().as_str());
        assert_eq!(
            snapshot_row.teaching_posture(),
            matrix_row.teaching_posture().as_str()
        );
        assert_eq!(snapshot_row.owner_milestone(), matrix_row.owner_milestone());
        assert_eq!(snapshot_row.extension_rule(), matrix_row.extension_rule());
        assert_eq!(
            snapshot_row.parallel_api_forbidden(),
            matrix_row.parallel_api_forbidden()
        );
        assert_eq!(
            snapshot_row.admission_fail_closed(),
            matrix_row.admission_fail_closed()
        );
        assert_eq!(
            snapshot_row.support_contract_digest(),
            matrix_row.support_contract_digest()
        );
        assert_eq!(
            snapshot_row.live_row_digest(),
            matrix_row.row_digest().terminal_projection_for_reporting()
        );
    }

    snapshot
        .assert_equivalent_to_live_matrix(&matrix)
        .expect("projected snapshot should match its live source matrix");
}

#[test]
fn loaded_support_snapshot_still_compares_to_the_live_matrix() {
    let matrix = live_support_matrix();
    let snapshot = project_support_snapshot(&matrix).expect("fixture digest must fit");
    let terminal_json_document = snapshot
        .to_canonical_terminal_json_document()
        .expect("snapshot JSON should encode");

    let loaded = load_support_snapshot_terminal_json_document(
        &terminal_json_document.to_external_terminal_json_document(),
        WorthQuerySupportSnapshotSchemaVersion::current(),
    )
    .expect("current snapshot should load");

    loaded
        .assert_equivalent_to_live_matrix(&matrix)
        .expect("loaded snapshot should retain live matrix equivalence");
}
