use super::scope::{scope_encoder, seal};
use crate::evidence_identity::WorthQueryEvidenceIdentityDerivationDenial as IdentityDerivationDenial;
use crate::WorthQueryEvidenceTag;

pub(crate) fn compose_query_context_row_identity(
    family: &str,
    index: usize,
) -> Result<String, IdentityDerivationDenial> {
    Ok(seal(
        scope_encoder("projection_consumption_query_context_row_v1")
            .field_shape(WorthQueryEvidenceTag::new("family"), family)
            .field_usize(WorthQueryEvidenceTag::new("index"), index),
    )?)
}

pub(crate) fn compose_live_binding_row_identity(
    binding_digest: &str,
    view_name: &str,
    index: usize,
) -> Result<String, IdentityDerivationDenial> {
    Ok(seal(
        scope_encoder("projection_consumption_live_binding_row_v1")
            .field_shape(WorthQueryEvidenceTag::new("binding_digest"), binding_digest)
            .field_shape(WorthQueryEvidenceTag::new("view_name"), view_name)
            .field_usize(WorthQueryEvidenceTag::new("index"), index),
    )?)
}

pub(crate) fn compose_retained_binding_row_identity(
    binding_digest: &str,
    view_name: &str,
    index: usize,
) -> Result<String, IdentityDerivationDenial> {
    Ok(seal(
        scope_encoder("projection_consumption_retained_binding_row_v1")
            .field_shape(WorthQueryEvidenceTag::new("binding_digest"), binding_digest)
            .field_shape(WorthQueryEvidenceTag::new("view_name"), view_name)
            .field_usize(WorthQueryEvidenceTag::new("index"), index),
    )?)
}

pub(crate) fn compose_scoped_row_source_identity(
    contract_source_identity: &str,
    row_identity: &str,
) -> Result<String, IdentityDerivationDenial> {
    Ok(seal(
        scope_encoder("projection_consumption_scoped_row_source_identity_v1")
            .field_shape(
                WorthQueryEvidenceTag::new("contract_source"),
                contract_source_identity,
            )
            .field_shape(WorthQueryEvidenceTag::new("row_identity"), row_identity),
    )?)
}
