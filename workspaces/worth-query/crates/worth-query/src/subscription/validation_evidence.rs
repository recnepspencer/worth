use crate::evidence_identity::WorthQueryEvidenceIdentityDerivationDenial as IdentityDerivationDenial;
use crate::evidence_identity::{
    WorthQueryEvidenceIdentity, WorthQueryEvidenceScope, WorthQueryEvidenceTag,
};

use super::terminal_projection_label::TerminalProjectionLabel;

pub(crate) fn validation_evidence_identity_label(
    identity: &WorthQueryEvidenceIdentity,
) -> TerminalProjectionLabel {
    TerminalProjectionLabel::from_identity(identity)
}

pub(crate) fn validation_role_evidence_identity(
    role: &'static str,
    identity: &WorthQueryEvidenceIdentity,
) -> WorthQueryEvidenceIdentity {
    WorthQueryEvidenceIdentity::compose(WorthQueryEvidenceScope::SubscriptionActivationReceipt)
        .field_shape(WorthQueryEvidenceTag::new("role"), role)
        .field_evidence_identity(WorthQueryEvidenceTag::new("subject"), identity)
        .seal().expect("closed validation roles encode a literal role and one sealed token or primitive counter")
}

pub(crate) fn validation_shape_role_evidence_identity(
    role: &'static str,
    shape: &str,
) -> Result<WorthQueryEvidenceIdentity, IdentityDerivationDenial> {
    WorthQueryEvidenceIdentity::compose(WorthQueryEvidenceScope::SubscriptionActivationReceipt)
        .field_shape(WorthQueryEvidenceTag::new("role"), role)
        .field_shape(WorthQueryEvidenceTag::new("subject"), shape)
        .seal()
}

pub(crate) fn validation_u64_role_evidence_identity(
    role: &'static str,
    value: u64,
) -> WorthQueryEvidenceIdentity {
    WorthQueryEvidenceIdentity::compose(WorthQueryEvidenceScope::SubscriptionActivationReceipt)
        .field_shape(WorthQueryEvidenceTag::new("role"), role)
        .field_usize(WorthQueryEvidenceTag::new("subject"), value as usize)
        .seal().expect("closed validation roles encode a literal role and one sealed token or primitive counter")
}

pub(crate) fn validation_usize_role_evidence_identity(
    role: &'static str,
    value: usize,
) -> WorthQueryEvidenceIdentity {
    WorthQueryEvidenceIdentity::compose(WorthQueryEvidenceScope::SubscriptionActivationReceipt)
        .field_shape(WorthQueryEvidenceTag::new("role"), role)
        .field_usize(WorthQueryEvidenceTag::new("subject"), value)
        .seal().expect("closed validation roles encode a literal role and one sealed token or primitive counter")
}

#[cfg(test)]
pub(crate) fn validation_label_list_evidence_identity(
    role: &'static str,
    labels: &[String],
) -> Result<WorthQueryEvidenceIdentity, IdentityDerivationDenial> {
    WorthQueryEvidenceIdentity::compose(WorthQueryEvidenceScope::SubscriptionActivationReceipt)
        .field_shape(WorthQueryEvidenceTag::new("role"), role)
        .field_value_sequence(
            WorthQueryEvidenceTag::new("subject"),
            labels.iter().map(String::as_str),
        )
        .seal()
}
