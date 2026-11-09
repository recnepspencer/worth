use crate::evidence_identity::WorthQueryEvidenceIdentityDerivationDenial as IdentityDerivationDenial;
use crate::evidence_identity::{
    worth_query_evidence_identity, WorthQueryEvidenceIdentity, WorthQueryEvidenceScope,
    WorthQueryEvidenceTag,
};

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct WorthQuerySupportSnapshotSchemaVersion {
    major: u16,
}

impl WorthQuerySupportSnapshotSchemaVersion {
    pub const fn current() -> Self {
        Self { major: 1 }
    }

    pub const fn major(self) -> u16 {
        self.major
    }

    pub fn as_str(self) -> &'static str {
        match self.major {
            1 => "support-snapshot-v1",
            _ => "unsupported-support-snapshot-schema",
        }
    }

    pub(crate) fn identity(self) -> Result<WorthQueryEvidenceIdentity, IdentityDerivationDenial> {
        Ok(
            worth_query_evidence_identity(WorthQueryEvidenceScope::ConsumerSupportSnapshotSchema)
                .field_shape(WorthQueryEvidenceTag::new("schema_version"), self.as_str())
                .field_usize(
                    WorthQueryEvidenceTag::new("schema_major"),
                    usize::from(self.major),
                )
                .field_value_sequence(
                    WorthQueryEvidenceTag::new("document_field"),
                    [
                        "schema_version",
                        "schema_identity",
                        "backend_posture",
                        "source_matrix_digest",
                        "snapshot_digest",
                        "rows",
                    ],
                )
                .field_value_sequence(
                    WorthQueryEvidenceTag::new("row_field"),
                    [
                        "surface",
                        "facade_family",
                        "status",
                        "teaching_posture",
                        "owner_milestone",
                        "extension_rule",
                        "parallel_api_forbidden",
                        "admission_fail_closed",
                        "support_contract_digest",
                        "live_row_digest",
                        "snapshot_row_digest",
                    ],
                )
                .seal()?,
        )
    }
}
