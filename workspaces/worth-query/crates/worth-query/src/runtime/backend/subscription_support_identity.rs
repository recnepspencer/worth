use crate::evidence_identity::WorthQueryEvidenceIdentityDerivationDenial as IdentityDerivationDenial;
use crate::evidence_identity::{
    WorthQueryEvidenceIdentity, WorthQueryEvidenceScope, WorthQueryEvidenceTag,
};

pub fn runtime_subscription_support_evidence_identity(
    support_label: &str,
) -> Result<WorthQueryEvidenceIdentity, IdentityDerivationDenial> {
    WorthQueryEvidenceIdentity::compose(WorthQueryEvidenceScope::SubscriptionActivationReceipt)
        .field_shape(
            WorthQueryEvidenceTag::new("identity_family"),
            "runtime_subscription_activation_support_evidence_v1",
        )
        .field_shape(WorthQueryEvidenceTag::new("support_label"), support_label)
        .seal()
}
