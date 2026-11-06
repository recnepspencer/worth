use crate::evidence_identity::WorthQueryEvidenceIdentityDerivationDenial as IdentityDerivationDenial;
use crate::runtime::WorthQueryRuntimePublicSupportMatrixRow;

use super::error::{WorthQuerySupportSnapshotError, WorthQuerySupportSnapshotErrorKind};
use super::evidence::support_snapshot_row_identity;
use super::schema::WorthQuerySupportSnapshotSchemaVersion;
use super::semantic_admission::{
    admit_support_snapshot_facade_family, admit_support_snapshot_status,
    admit_support_snapshot_teaching_posture,
};

#[derive(Clone, Debug, Eq, PartialEq, serde::Deserialize, serde::Serialize)]
pub struct WorthQuerySupportSnapshotRow {
    surface: String,
    facade_family: Option<String>,
    status: String,
    teaching_posture: String,
    owner_milestone: String,
    extension_rule: String,
    parallel_api_forbidden: bool,
    admission_fail_closed: bool,
    support_contract_digest: Option<String>,
    live_row_digest: String,
    snapshot_row_digest: String,
}

impl WorthQuerySupportSnapshotRow {
    pub(crate) fn from_runtime_row(
        schema_version: WorthQuerySupportSnapshotSchemaVersion,
        row: &WorthQueryRuntimePublicSupportMatrixRow,
    ) -> Result<Self, IdentityDerivationDenial> {
        let mut snapshot_row = Self {
            surface: row.surface().to_string(),
            facade_family: row
                .facade_family()
                .map(|family| family.as_str().to_string()),
            status: row.status().as_str().to_string(),
            teaching_posture: row.teaching_posture().as_str().to_string(),
            owner_milestone: row.owner_milestone().to_string(),
            extension_rule: row.extension_rule().to_string(),
            parallel_api_forbidden: row.parallel_api_forbidden(),
            admission_fail_closed: row.admission_fail_closed(),
            support_contract_digest: row.support_contract_digest().map(str::to_string),
            live_row_digest: row
                .row_digest()
                .terminal_projection_for_reporting()
                .to_string(),
            snapshot_row_digest: String::new(),
        };
        snapshot_row.snapshot_row_digest =
            support_snapshot_row_identity(schema_version, &snapshot_row)?
                .terminal_projection_for_reporting()
                .to_string();
        Ok(snapshot_row)
    }

    pub fn surface(&self) -> &str {
        &self.surface
    }

    pub fn facade_family(&self) -> Option<&str> {
        self.facade_family.as_deref()
    }

    pub fn status(&self) -> &str {
        &self.status
    }

    pub fn teaching_posture(&self) -> &str {
        &self.teaching_posture
    }

    pub fn owner_milestone(&self) -> &str {
        &self.owner_milestone
    }

    pub fn extension_rule(&self) -> &str {
        &self.extension_rule
    }

    pub fn parallel_api_forbidden(&self) -> bool {
        self.parallel_api_forbidden
    }

    pub fn admission_fail_closed(&self) -> bool {
        self.admission_fail_closed
    }

    pub fn support_contract_digest(&self) -> Option<&str> {
        self.support_contract_digest.as_deref()
    }

    pub fn live_row_digest(&self) -> &str {
        &self.live_row_digest
    }

    pub fn snapshot_row_digest(&self) -> &str {
        &self.snapshot_row_digest
    }

    pub(crate) fn validate_snapshot_row_digest(
        &self,
        schema_version: WorthQuerySupportSnapshotSchemaVersion,
    ) -> Result<(), WorthQuerySupportSnapshotError> {
        let rebuilt = support_snapshot_row_identity(schema_version, self)
            .terminal_projection_for_reporting()
            .to_string();
        if rebuilt == self.snapshot_row_digest {
            Ok(())
        } else {
            Err(WorthQuerySupportSnapshotError::new(
                WorthQuerySupportSnapshotErrorKind::RowDigestMismatch,
                format!(
                    "support snapshot row digest mismatch for {}: expected {rebuilt}, found {}",
                    self.surface, self.snapshot_row_digest
                ),
            ))
        }
    }

    pub(crate) fn admit_schema_v1_semantics(&self) -> Result<(), WorthQuerySupportSnapshotError> {
        self.reject_blank_required_field("surface", &self.surface)?;
        self.reject_blank_required_field("status", &self.status)?;
        self.reject_blank_required_field("teaching_posture", &self.teaching_posture)?;
        self.reject_blank_required_field("owner_milestone", &self.owner_milestone)?;
        self.reject_blank_required_field("extension_rule", &self.extension_rule)?;
        self.reject_blank_required_field("live_row_digest", &self.live_row_digest)?;
        self.reject_blank_required_field("snapshot_row_digest", &self.snapshot_row_digest)?;
        admit_support_snapshot_facade_family(&self.surface, self.facade_family())?;
        admit_support_snapshot_status(&self.surface, &self.status)?;
        admit_support_snapshot_teaching_posture(&self.surface, &self.teaching_posture)
    }

    fn reject_blank_required_field(
        &self,
        field_name: &str,
        value: &str,
    ) -> Result<(), WorthQuerySupportSnapshotError> {
        if value.trim().is_empty() {
            Err(WorthQuerySupportSnapshotError::with_surface_found(
                WorthQuerySupportSnapshotErrorKind::InvalidRequiredField,
                format!("support snapshot row required field {field_name} is blank"),
                self.surface.clone(),
                field_name,
            ))
        } else {
            Ok(())
        }
    }
}
