use crate::evidence_identity::WorthQueryEvidenceIdentityDerivationDenial as IdentityDerivationDenial;
use crate::evidence_identity::{
    WorthQueryEvidenceIdentity, WorthQueryEvidenceScope, WorthQueryEvidenceTag,
};

pub(super) fn subscription_certification_failure_identity(
    identity_family: &'static str,
    kind: &'static str,
    message: &'static str,
    evidence: &[WorthQueryEvidenceIdentity],
) -> Result<WorthQueryEvidenceIdentity, IdentityDerivationDenial> {
    WorthQueryEvidenceIdentity::compose(WorthQueryEvidenceScope::SubscriptionActivationReceipt)
        .field_shape(
            WorthQueryEvidenceTag::new("identity_family"),
            identity_family,
        )
        .field_shape(WorthQueryEvidenceTag::new("kind"), kind)
        .field_value(WorthQueryEvidenceTag::new("message"), message)
        .field_evidence_identity_sequence(WorthQueryEvidenceTag::new("evidence"), evidence.iter())
        .seal()
}
