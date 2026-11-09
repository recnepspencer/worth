use crate::consumer_kit::{
    project_support_snapshot, WorthQuerySupportSnapshot, WorthQuerySupportSnapshotErrorKind,
};

use super::{live_support_matrix, scaffold_support_matrix};

#[test]
fn support_snapshot_compare_denies_different_real_matrix_with_typed_digest_context() {
    let primary_matrix = live_support_matrix();
    let scaffold_matrix = scaffold_support_matrix();
    let snapshot = project_support_snapshot(&primary_matrix).expect("fixture digest must fit");

    let error = snapshot
        .assert_equivalent_to_live_matrix(&scaffold_matrix)
        .expect_err("snapshot from primary matrix must not compare to scaffold matrix");

    assert_eq!(
        error.kind(),
        &WorthQuerySupportSnapshotErrorKind::SourceMatrixDigestMismatch
    );
    assert_eq!(
        error.expected(),
        Some(
            scaffold_matrix
                .matrix_digest()
                .terminal_projection_for_reporting()
        )
    );
    assert_eq!(error.found(), Some(snapshot.source_matrix_digest()));
}

#[test]
fn support_snapshot_compare_denies_row_count_mismatch() {
    let matrix = live_support_matrix();
    let snapshot = project_support_snapshot(&matrix).expect("fixture digest must fit");
    let mut rows = snapshot.rows().to_vec();
    rows.pop();
    let truncated_snapshot = WorthQuerySupportSnapshot::from_validated_parts(
        snapshot.schema_version(),
        snapshot.schema_identity().to_string(),
        snapshot.backend_posture().to_string(),
        snapshot.source_matrix_digest().to_string(),
        rows,
        snapshot.snapshot_digest().to_string(),
    );

    let error = truncated_snapshot
        .assert_equivalent_to_live_matrix(&matrix)
        .expect_err("missing row must fail comparison");

    assert_eq!(
        error.kind(),
        &WorthQuerySupportSnapshotErrorKind::RowCountMismatch
    );
}

#[test]
fn support_snapshot_compare_reports_row_field_mismatch_with_structured_context() {
    let matrix = live_support_matrix();
    let snapshot = project_support_snapshot(&matrix).expect("fixture digest must fit");
    let scaffold_snapshot =
        project_support_snapshot(&scaffold_support_matrix()).expect("fixture digest must fit");
    let mismatched_snapshot = WorthQuerySupportSnapshot::from_validated_parts(
        snapshot.schema_version(),
        snapshot.schema_identity().to_string(),
        snapshot.backend_posture().to_string(),
        snapshot.source_matrix_digest().to_string(),
        scaffold_snapshot.rows().to_vec(),
        snapshot.snapshot_digest().to_string(),
    );

    let error = mismatched_snapshot
        .assert_equivalent_to_live_matrix(&matrix)
        .expect_err("row field drift must fail comparison");

    assert_eq!(
        error.kind(),
        &WorthQuerySupportSnapshotErrorKind::RowMismatch
    );
    let row_index = error
        .row_index()
        .expect("row mismatch should carry row index");
    let field = error.field().expect("row mismatch should carry field");
    assert_eq!(error.surface(), Some(matrix.rows()[row_index].surface()));
    match field {
        "support_contract_digest" => {
            assert_support_contract_digest_mismatch(&matrix, &scaffold_snapshot, &error, row_index)
        }
        "live_row_digest" => {
            assert_live_row_digest_mismatch(&matrix, &scaffold_snapshot, &error, row_index);
        }
        unexpected => panic!("unexpected first row mismatch field: {unexpected}"),
    }
}

fn assert_support_contract_digest_mismatch(
    matrix: &crate::runtime::WorthQueryRuntimePublicSupportMatrix,
    scaffold_snapshot: &WorthQuerySupportSnapshot,
    error: &crate::consumer_kit::WorthQuerySupportSnapshotError,
    row_index: usize,
) {
    assert_eq!(
        error.expected(),
        matrix.rows()[row_index].support_contract_digest()
    );
    assert_eq!(
        error.found(),
        scaffold_snapshot.rows()[row_index].support_contract_digest()
    );
}

fn assert_live_row_digest_mismatch(
    matrix: &crate::runtime::WorthQueryRuntimePublicSupportMatrix,
    scaffold_snapshot: &WorthQuerySupportSnapshot,
    error: &crate::consumer_kit::WorthQuerySupportSnapshotError,
    row_index: usize,
) {
    assert_eq!(
        error.expected(),
        Some(
            matrix.rows()[row_index]
                .row_digest()
                .terminal_projection_for_reporting()
        )
    );
    assert_eq!(
        error.found(),
        Some(scaffold_snapshot.rows()[row_index].live_row_digest())
    );
}
