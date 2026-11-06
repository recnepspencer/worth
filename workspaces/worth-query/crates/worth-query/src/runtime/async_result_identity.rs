use crate::evidence_identity::{
    worth_query_evidence_identity, WorthQueryEvidenceIdentity, WorthQueryEvidenceScope,
    WorthQueryEvidenceTag,
};

use super::async_result_state::WorthQueryRuntimeAsyncResultStateKind;

pub(super) fn runtime_async_causality_from_bridge(
    source_identity: &str,
    source_digest: &str,
) -> Result<WorthQueryEvidenceIdentity, crate::evidence_identity::WorthQueryEvidenceIdentityDerivationDenial> {
    let source =
        worth_query_evidence_identity(WorthQueryEvidenceScope::LowerRuntimeBoundaryEvidence)
            .field_shape(
                WorthQueryEvidenceTag::new("identity_family"),
                "worth_query_runtime_bridge_async_cause_v1",
            )
            .field_shape(
                WorthQueryEvidenceTag::new("bridge_source_identity"),
                source_identity,
            )
            .field_shape(
                WorthQueryEvidenceTag::new("bridge_source_digest"),
                source_digest,
            )
            .seal()?;
    Ok(runtime_async_causality_identity(&source)?)
}

pub(super) fn runtime_async_causality_label_identity(label: &str) -> Result<WorthQueryEvidenceIdentity, crate::evidence_identity::WorthQueryEvidenceIdentityDerivationDenial> {
    Ok(worth_query_evidence_identity(WorthQueryEvidenceScope::LowerRuntimeBoundaryEvidence)
        .field_shape(
            WorthQueryEvidenceTag::new("identity_family"),
            "worth_query_runtime_async_causality_label_v1",
        )
        .field_shape(WorthQueryEvidenceTag::new("label"), label)
        .seal()?)
}

pub(crate) fn runtime_async_causality_identity(
    causality_identity: &WorthQueryEvidenceIdentity,
) -> Result<WorthQueryEvidenceIdentity, crate::evidence_identity::WorthQueryEvidenceIdentityDerivationDenial> {
    Ok(worth_query_evidence_identity(WorthQueryEvidenceScope::LowerRuntimeBoundaryEvidence)
        .field_shape(
            WorthQueryEvidenceTag::new("identity_family"),
            "worth_query_runtime_async_causality_v1",
        )
        .field_evidence_identity(WorthQueryEvidenceTag::new("causality"), causality_identity)
        .seal()?)
}

pub(super) fn runtime_async_result_state_identity(
    kind: WorthQueryRuntimeAsyncResultStateKind,
    causality_identity: &WorthQueryEvidenceIdentity,
    basis_identity: &WorthQueryEvidenceIdentity,
    checkpoint_identity: &WorthQueryEvidenceIdentity,
) -> Result<WorthQueryEvidenceIdentity, crate::evidence_identity::WorthQueryEvidenceIdentityDerivationDenial> {
    Ok(worth_query_evidence_identity(WorthQueryEvidenceScope::RuntimeStateSnapshot)
        .field_shape(
            WorthQueryEvidenceTag::new("identity_family"),
            "worth_query_runtime_async_result_state_v1",
        )
        .field_shape(WorthQueryEvidenceTag::new("kind"), kind.as_str())
        .field_evidence_identity(WorthQueryEvidenceTag::new("causality"), causality_identity)
        .field_evidence_identity(WorthQueryEvidenceTag::new("basis"), basis_identity)
        .field_evidence_identity(
            WorthQueryEvidenceTag::new("checkpoint"),
            checkpoint_identity,
        )
        .seal()?)
}

#[cfg(test)]
pub(crate) fn runtime_async_causality_from_label(label: &str) -> Result<WorthQueryEvidenceIdentity, crate::evidence_identity::WorthQueryEvidenceIdentityDerivationDenial> {
    Ok(runtime_async_causality_identity(&runtime_async_causality_label_identity(label)?)?)
}

#[cfg(test)]
pub(crate) fn runtime_async_checkpoint_label_identity(label: &str) -> Result<WorthQueryEvidenceIdentity, crate::evidence_identity::WorthQueryEvidenceIdentityDerivationDenial> {
    Ok(worth_query_evidence_identity(WorthQueryEvidenceScope::LowerRuntimeBoundaryEvidence)
        .field_shape(
            WorthQueryEvidenceTag::new("identity_family"),
            "worth_query_runtime_async_checkpoint_label_v1",
        )
        .field_shape(WorthQueryEvidenceTag::new("label"), label)
        .seal()?)
}
