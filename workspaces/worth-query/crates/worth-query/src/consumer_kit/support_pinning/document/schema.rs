use crate::evidence_identity::WorthQueryEvidenceIdentityDerivationDenial as IdentityDerivationDenial;
use crate::evidence_identity::{
    worth_query_evidence_identity, WorthQueryEvidenceIdentity, WorthQueryEvidenceScope,
    WorthQueryEvidenceTag,
};

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct WorthQuerySupportPinContractSchemaVersion {
    major: u16,
}

impl WorthQuerySupportPinContractSchemaVersion {
    pub const fn current() -> Self {
        Self { major: 1 }
    }

    pub const fn major(self) -> u16 {
        self.major
    }

    pub fn as_str(self) -> &'static str {
        match self.major {
            1 => "support-pin-contract-v1",
            _ => "unsupported-support-pin-contract-schema",
        }
    }

    pub(crate) fn identity(self) -> Result<WorthQueryEvidenceIdentity, IdentityDerivationDenial> {
        worth_query_evidence_identity(WorthQueryEvidenceScope::ConsumerSupportPinContractSchema)
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
                    "pinned_vocabulary_identity",
                    "support_snapshot_schema_identity",
                    "source_matrix_digest",
                    "consumer_name",
                    "contract_digest",
                    "requirements",
                    "observed_rows",
                ],
            )
            .field_value_sequence(
                WorthQueryEvidenceTag::new("requirement_field"),
                [
                    "family",
                    "surface",
                    "required_status",
                    "required_teaching_posture",
                    "pinned_live_row_digest",
                    "pinned_snapshot_row_digest",
                ],
            )
            .field_value_sequence(
                WorthQueryEvidenceTag::new("observed_field"),
                [
                    "family",
                    "surface",
                    "observed_status",
                    "observed_teaching_posture",
                    "observed_live_row_digest",
                ],
            )
            .seal()
    }
}

pub(crate) fn support_pin_vocabulary_identity(
) -> Result<WorthQueryEvidenceIdentity, IdentityDerivationDenial> {
    worth_query_evidence_identity(WorthQueryEvidenceScope::ConsumerSupportPinVocabulary)
        .field_value_sequence(
            WorthQueryEvidenceTag::new("support_status"),
            ["supported", "deferred-debt", "unsupported"],
        )
        .field_value_sequence(
            WorthQueryEvidenceTag::new("teaching_posture"),
            [
                "ordinary-runtime-dx",
                "visible-but-deferred",
                "visible-vocabulary-only",
                "support-gate-only",
            ],
        )
        .field_value_sequence(
            WorthQueryEvidenceTag::new("facade_family"),
            [
                "read",
                "live",
                "computed",
                "shared-read",
                "submission",
                "replay",
                "effect",
                "branch-preview",
                "write",
                "intent",
                "inspect",
                "temporal",
                "async-resource",
                "mixed-cause-delivery",
                "store-backed-execution",
                "durable-artifacts",
            ],
        )
        .seal()
}
