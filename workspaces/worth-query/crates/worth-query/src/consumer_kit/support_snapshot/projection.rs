use crate::evidence_identity::WorthQueryEvidenceIdentityDerivationDenial as IdentityDerivationDenial;
use crate::runtime::{WorthQueryRuntimePublicSupportMatrix, WorthQueryWorkspace};

use super::document::{
    WorthQueryExternalSupportSnapshotTerminalJsonDocument, WorthQuerySupportSnapshotDocument,
};
use super::error::WorthQuerySupportSnapshotError;
use super::schema::WorthQuerySupportSnapshotSchemaVersion;
use super::snapshot::WorthQuerySupportSnapshot;

pub fn project_support_snapshot(
    matrix: &WorthQueryRuntimePublicSupportMatrix,
) -> Result<WorthQuerySupportSnapshot, IdentityDerivationDenial> {
    Ok(WorthQuerySupportSnapshot::from_public_support_matrix(
        matrix,
    )?)
}

pub fn project_workspace_support_snapshot(
    workspace: &WorthQueryWorkspace,
) -> Result<WorthQuerySupportSnapshot, IdentityDerivationDenial> {
    let matrix = workspace.public_support_matrix();
    Ok(project_support_snapshot(&matrix)?)
}

pub fn load_support_snapshot_terminal_json_document(
    terminal_json_document: &WorthQueryExternalSupportSnapshotTerminalJsonDocument,
    expected_schema_version: WorthQuerySupportSnapshotSchemaVersion,
) -> Result<WorthQuerySupportSnapshot, WorthQuerySupportSnapshotError> {
    WorthQuerySupportSnapshot::from_document(
        WorthQuerySupportSnapshotDocument::from_terminal_json_document(terminal_json_document)?,
        expected_schema_version,
    )
}
