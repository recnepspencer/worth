use super::*;
use crate::evidence_identity::WorthQueryEvidenceIdentityDerivationDenial as IdentityDerivationDenial;

pub(crate) fn runtime_state_snapshot_basis_label_identity(
    basis_identity: &WorthQueryEvidenceIdentity,
) -> WorthQueryEvidenceIdentity {
    worth_query_evidence_identity(WorthQueryEvidenceScope::LowerRuntimeBoundaryEvidence)
            .field_shape(
                WorthQueryEvidenceTag::new("identity_family"),
                "worth_query_runtime_state_snapshot_basis_v1",
            )
            .field_evidence_identity(WorthQueryEvidenceTag::new("basis"), basis_identity)
            .seal().expect("fixed digest grammar: literal tags and values, primitive values, or sealed identity tokens")
}

pub(crate) fn runtime_state_snapshot_result_shape_label_identity(
    result_shape_identity: &WorthQueryEvidenceIdentity,
) -> WorthQueryEvidenceIdentity {
    worth_query_evidence_identity(WorthQueryEvidenceScope::LowerRuntimeBoundaryEvidence)
            .field_shape(
                WorthQueryEvidenceTag::new("identity_family"),
                "worth_query_runtime_state_snapshot_result_shape_v1",
            )
            .field_evidence_identity(
                WorthQueryEvidenceTag::new("result_shape"),
                result_shape_identity,
            )
            .seal().expect("fixed digest grammar: literal tags and values, primitive values, or sealed identity tokens")
}

pub(in crate::runtime) fn runtime_state_snapshot_result_shape_facade_family_identity(
    facade_family: WorthQueryRuntimeFacadeFamily,
) -> WorthQueryEvidenceIdentity {
    worth_query_evidence_identity(WorthQueryEvidenceScope::LowerRuntimeBoundaryEvidence)
            .field_shape(
                WorthQueryEvidenceTag::new("identity_family"),
                "worth_query_runtime_state_snapshot_result_shape_facade_family_v1",
            )
            .field_shape(
                WorthQueryEvidenceTag::new("facade_family"),
                facade_family.as_str(),
            )
            .seal().expect("fixed digest grammar: literal tags, primitive values, closed enum labels, or sealed digest tokens")
}

pub(in crate::runtime) fn runtime_state_snapshot_result_shape_write_receipt_identity(
    receipt: &WorthQueryWriteReceipt,
) -> Result<WorthQueryEvidenceIdentity, IdentityDerivationDenial> {
    let declared_entity_identity = receipt
        .declared_entity_identity()
        .map(|identity| identity.evidence_identity());
    Ok(
        worth_query_evidence_identity(WorthQueryEvidenceScope::LowerRuntimeBoundaryEvidence)
            .field_shape(
                WorthQueryEvidenceTag::new("identity_family"),
                "worth_query_runtime_state_snapshot_result_shape_write_receipt_v1",
            )
            .field_shape(
                WorthQueryEvidenceTag::new("mutation_family"),
                receipt.mutation_family().as_str(),
            )
            .optional_shape(
                WorthQueryEvidenceTag::new("declared_collection"),
                receipt.terminal_declared_collection_projection(),
            )
            .field_evidence_identity(
                WorthQueryEvidenceTag::new("commit_evidence_identity"),
                receipt.commit_evidence_identity(),
            )
            .optional_evidence_identity(
                WorthQueryEvidenceTag::new("declared_entity_identity"),
                declared_entity_identity.as_ref(),
            )
            .seal()?,
    )
}

pub(in crate::runtime) fn runtime_state_snapshot_result_shape_batch_write_receipt_identity(
    receipt: &WorthQueryBatchWriteReceipt,
) -> WorthQueryEvidenceIdentity {
    worth_query_evidence_identity(WorthQueryEvidenceScope::LowerRuntimeBoundaryEvidence)
            .field_shape(
                WorthQueryEvidenceTag::new("identity_family"),
                "worth_query_runtime_state_snapshot_result_shape_batch_write_receipt_v1",
            )
            .field_usize(
                WorthQueryEvidenceTag::new("write_count"),
                receipt.write_count(),
            )
            .seal().expect("fixed digest grammar: literal tags and values, primitive values, or sealed identity tokens")
}

pub(in crate::runtime) fn runtime_live_view_consumer_attachment_identity(
    view_name: &str,
) -> Result<WorthQueryEvidenceIdentity, IdentityDerivationDenial> {
    Ok(
        worth_query_evidence_identity(WorthQueryEvidenceScope::LowerRuntimeBoundaryEvidence)
            .field_shape(
                WorthQueryEvidenceTag::new("identity_family"),
                "runtime_live_view_consumer_attachment_v1",
            )
            .field_shape(WorthQueryEvidenceTag::new("view"), view_name)
            .seal()?,
    )
}
