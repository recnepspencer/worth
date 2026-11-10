use crate::evidence_identity::WorthQueryEvidenceIdentityDerivationDenial as IdentityDerivationDenial;
use crate::evidence_identity::WorthQueryEvidenceIdentityEncoder;
use crate::{WorthQueryEvidenceIdentity, WorthQueryEvidenceScope, WorthQueryEvidenceTag};

pub(crate) fn consumption_scope_encoder(
    identity_family: &str,
) -> WorthQueryEvidenceIdentityEncoder {
    WorthQueryEvidenceIdentity::compose(WorthQueryEvidenceScope::ProjectionConsumptionIdentity)
        .field_shape(
            WorthQueryEvidenceTag::new("identity_family"),
            identity_family,
        )
}

pub(crate) fn certification_scope_encoder(
    identity_family: &str,
) -> WorthQueryEvidenceIdentityEncoder {
    WorthQueryEvidenceIdentity::compose(
        WorthQueryEvidenceScope::ProjectionConsumptionCertificationIdentity,
    )
    .field_shape(
        WorthQueryEvidenceTag::new("identity_family"),
        identity_family,
    )
}

pub(crate) fn scope_encoder(identity_family: &str) -> WorthQueryEvidenceIdentityEncoder {
    consumption_scope_encoder(identity_family)
}

pub(crate) fn seal(
    encoder: WorthQueryEvidenceIdentityEncoder,
) -> Result<String, IdentityDerivationDenial> {
    Ok(encoder.seal()?.as_str().to_string())
}

pub(crate) fn compose_sequence_digest(
    identity_family: &str,
    tag: &'static str,
    values: impl IntoIterator<Item = impl AsRef<str>>,
) -> Result<String, IdentityDerivationDenial> {
    seal(
        consumption_scope_encoder(identity_family)
            .field_value_sequence(WorthQueryEvidenceTag::new(tag), values),
    )
}

pub(crate) fn compose_certification_sequence_digest(
    identity_family: &str,
    tag: &'static str,
    values: impl IntoIterator<Item = impl AsRef<str>>,
) -> Result<String, IdentityDerivationDenial> {
    seal(
        certification_scope_encoder(identity_family)
            .field_value_sequence(WorthQueryEvidenceTag::new(tag), values),
    )
}

pub(crate) fn compose_labeled_entry_digest(
    identity_family: &'static str,
    entries: &[(&'static str, &str)],
) -> Result<String, IdentityDerivationDenial> {
    let mut encoder = certification_scope_encoder(identity_family);
    for (tag, value) in entries {
        encoder = encoder.field_shape(WorthQueryEvidenceTag::new(tag), *value);
    }
    seal(encoder)
}
