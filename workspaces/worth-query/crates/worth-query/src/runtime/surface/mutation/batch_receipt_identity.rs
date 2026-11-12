use crate::evidence_identity::WorthQueryEvidenceIdentityDerivationDenial as IdentityDerivationDenial;
use crate::evidence_identity::{
    worth_query_evidence_identity, WorthQueryEvidenceIdentity, WorthQueryEvidenceScope,
    WorthQueryEvidenceTag,
};
use crate::runtime::{
    WorthQueryAspectTouch, WorthQueryDerivedMaterializationTarget, WorthQueryLiveArtifactTarget,
};

pub(super) fn evidence_value_identities(
    role: &'static str,
    values: &[String],
) -> Result<Vec<WorthQueryEvidenceIdentity>, IdentityDerivationDenial> {
    values
        .iter()
        .map(|value| {
            worth_query_evidence_identity(WorthQueryEvidenceScope::BatchWriteReceipt)
                .field_shape(WorthQueryEvidenceTag::new("role"), role)
                .field_value(WorthQueryEvidenceTag::new("value"), value)
                .seal()
        })
        .collect::<Result<Vec<_>, _>>()
}

pub(super) fn terminal_touch_projection_identities(
    role: &'static str,
    touches: &[WorthQueryAspectTouch],
) -> Result<Vec<WorthQueryEvidenceIdentity>, IdentityDerivationDenial> {
    touches
        .iter()
        .map(|touch| {
            worth_query_evidence_identity(WorthQueryEvidenceScope::BatchWriteReceipt)
                .field_shape(WorthQueryEvidenceTag::new("role"), role)
                .field_value(
                    WorthQueryEvidenceTag::new("value"),
                    touch.admitted_touch_digest_part(),
                )
                .seal()
        })
        .collect::<Result<Vec<_>, _>>()
}

pub(in crate::runtime) fn terminal_live_view_ids(
    targets: &[WorthQueryLiveArtifactTarget],
) -> Vec<String> {
    targets
        .iter()
        .map(|target| target.view_name().to_string())
        .collect()
}

pub(in crate::runtime) fn terminal_derived_view_ids(
    targets: &[WorthQueryDerivedMaterializationTarget],
) -> Vec<String> {
    targets
        .iter()
        .map(|target| target.view_name().to_string())
        .collect()
}
