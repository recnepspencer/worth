use crate::evidence_identity::WorthQueryEvidenceIdentityDerivationDenial as IdentityDerivationDenial;
use crate::evidence_identity::{
    worth_query_evidence_identity, WorthQueryEvidenceIdentity, WorthQueryEvidenceScope,
    WorthQueryEvidenceTag,
};
use crate::runtime::{WorthQueryAspectMutationOperation, WorthQueryMutationMetadata};

pub(super) fn declared_aspect_operation_identities(
    operations: &[WorthQueryAspectMutationOperation],
) -> Result<Vec<WorthQueryEvidenceIdentity>, IdentityDerivationDenial> {
    operations
        .iter()
        .map(|operation| {
            worth_query_evidence_identity(
                WorthQueryEvidenceScope::WriteReceiptDeclaredAspectOperation,
            )
            .field_shape(
                WorthQueryEvidenceTag::new("kind"),
                operation.kind().as_str(),
            )
            .field_value(
                WorthQueryEvidenceTag::new("admitted_aspect_touch"),
                operation.aspect_touch().admitted_touch_digest_part(),
            )
            .seal()
        })
        .collect::<Result<Vec<_>, _>>()
}

pub(super) fn mutation_metadata_entry_identities(
    mutation_metadata: &WorthQueryMutationMetadata,
) -> Result<Vec<WorthQueryEvidenceIdentity>, IdentityDerivationDenial> {
    mutation_metadata
        .entries()
        .map(|(key, value)| {
            worth_query_evidence_identity(
                WorthQueryEvidenceScope::WriteReceiptMutationMetadataEntry,
            )
            .field_value(WorthQueryEvidenceTag::new("key"), key.as_str())
            .field_value(
                WorthQueryEvidenceTag::new("value"),
                value.terminal_digest_text(),
            )
            .seal()
        })
        .collect::<Result<Vec<_>, _>>()
}
