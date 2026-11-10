use crate::evidence_identity::WorthQueryEvidenceIdentityDerivationDenial as IdentityDerivationDenial;
use crate::evidence_identity::WorthQueryEvidenceIdentityEncoder;
use crate::projection_consumption::ProjectionFactRequest;
use crate::{WorthQueryEvidenceIdentity, WorthQueryEvidenceScope, WorthQueryEvidenceTag};

pub(crate) fn domain_capability_scope_encoder(
    identity_family: &str,
) -> WorthQueryEvidenceIdentityEncoder {
    WorthQueryEvidenceIdentity::compose(WorthQueryEvidenceScope::DomainCapabilityIdentity)
        .field_shape(
            WorthQueryEvidenceTag::new("identity_family"),
            identity_family,
        )
}

pub(crate) fn domain_capability_certification_scope_encoder(
    identity_family: &str,
) -> WorthQueryEvidenceIdentityEncoder {
    WorthQueryEvidenceIdentity::compose(
        WorthQueryEvidenceScope::DomainCapabilityCertificationIdentity,
    )
    .field_shape(
        WorthQueryEvidenceTag::new("identity_family"),
        identity_family,
    )
}

pub(crate) fn seal(
    encoder: WorthQueryEvidenceIdentityEncoder,
) -> Result<String, IdentityDerivationDenial> {
    Ok(encoder.seal()?.as_str().to_string())
}

pub(crate) fn compose_certification_sequence_digest(
    identity_family: &str,
    tag: &'static str,
    values: impl IntoIterator<Item = impl AsRef<str>>,
) -> Result<String, IdentityDerivationDenial> {
    seal(
        domain_capability_certification_scope_encoder(identity_family)
            .field_value_sequence(WorthQueryEvidenceTag::new(tag), values),
    )
}

pub(crate) fn compose_fact_request_entry_digest(
    request: &ProjectionFactRequest,
) -> Result<String, IdentityDerivationDenial> {
    let mut encoder = domain_capability_scope_encoder("projection_fact_request_entry_v1")
        .field_shape(WorthQueryEvidenceTag::new("kind"), request.kind().as_str());
    if let Some(field) = request.field_path() {
        encoder = encoder.field_shape(
            WorthQueryEvidenceTag::new("field"),
            field.terminal_projection_for_boundary(),
        );
    }
    seal(encoder)
}
