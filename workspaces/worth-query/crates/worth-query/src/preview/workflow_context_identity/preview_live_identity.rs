use crate::evidence_identity::WorthQueryEvidenceIdentityDerivationDenial as IdentityDerivationDenial;
use crate::{WorthQueryEvidenceIdentity, WorthQueryEvidenceScope, WorthQueryEvidenceTag};

pub(in crate::preview) fn compose_preview_live_admission_digest(
    preview_binding_digest: &str,
    live_subscription_digest: &str,
    live_family: &str,
) -> Result<String, IdentityDerivationDenial> {
    Ok(
        WorthQueryEvidenceIdentity::compose(WorthQueryEvidenceScope::WorkflowContextBinding)
            .field_shape(
                WorthQueryEvidenceTag::new("identity_family"),
                "worth_query_preview_live_admission_v1",
            )
            .field_shape(
                WorthQueryEvidenceTag::new("preview_binding"),
                preview_binding_digest,
            )
            .field_shape(
                WorthQueryEvidenceTag::new("live_subscription"),
                live_subscription_digest,
            )
            .field_shape(WorthQueryEvidenceTag::new("live_family"), live_family)
            .seal()?
            .as_str()
            .to_string(),
    )
}
