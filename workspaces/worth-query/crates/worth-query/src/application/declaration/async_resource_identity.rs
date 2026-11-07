use crate::evidence_identity::WorthQueryEvidenceIdentityDerivationDenial as IdentityDerivationDenial;
use crate::{WorthQueryEvidenceIdentity, WorthQueryEvidenceScope, WorthQueryEvidenceTag};

use super::{
    WorthQueryAsyncFailurePosture, WorthQueryAsyncLoadingPosture,
    WorthQueryAsyncRequestIdentityPart, WorthQueryAsyncSourceFamily,
};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorthQueryAsyncResourceRequestIdentity {
    source_family: WorthQueryAsyncSourceFamily,
    loading_posture: WorthQueryAsyncLoadingPosture,
    failure_posture: WorthQueryAsyncFailurePosture,
    request_identity: Vec<WorthQueryAsyncRequestIdentityPart>,
    evidence_identity: WorthQueryEvidenceIdentity,
}

impl WorthQueryAsyncResourceRequestIdentity {
    pub fn declare(
        source_family: WorthQueryAsyncSourceFamily,
        loading_posture: WorthQueryAsyncLoadingPosture,
        failure_posture: WorthQueryAsyncFailurePosture,
        request_identity: Vec<WorthQueryAsyncRequestIdentityPart>,
    ) -> Result<Self, WorthQueryAsyncResourceRequestIdentityError> {
        let request_identity = admit_identity_parts(request_identity)?;
        let encoded_parts = request_identity
            .iter()
            .map(WorthQueryAsyncRequestIdentityPart::evidence_component)
            .collect::<Vec<_>>();
        let evidence_identity = WorthQueryEvidenceIdentity::compose(
            WorthQueryEvidenceScope::AsyncResourceRequestIdentity,
        )
        .field_shape(
            WorthQueryEvidenceTag::new("source_family"),
            source_family.as_str(),
        )
        .field_shape(
            WorthQueryEvidenceTag::new("loading_posture"),
            loading_posture.as_str(),
        )
        .field_shape(
            WorthQueryEvidenceTag::new("failure_posture"),
            failure_posture.as_str(),
        )
        .field_value_sequence(
            WorthQueryEvidenceTag::new("request_identity"),
            &encoded_parts,
        )
        .seal()?;

        Ok(Self {
            source_family,
            loading_posture,
            failure_posture,
            request_identity,
            evidence_identity,
        })
    }

    pub fn source_family(&self) -> WorthQueryAsyncSourceFamily {
        self.source_family
    }

    pub fn loading_posture(&self) -> WorthQueryAsyncLoadingPosture {
        self.loading_posture
    }

    pub fn failure_posture(&self) -> WorthQueryAsyncFailurePosture {
        self.failure_posture
    }

    pub fn request_identity(&self) -> &[WorthQueryAsyncRequestIdentityPart] {
        &self.request_identity
    }

    pub fn canonical_identity(&self) -> &str {
        self.evidence_identity.terminal_projection_for_reporting()
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum WorthQueryAsyncResourceRequestIdentityError {
    EvidenceIdentityDerivationDenied(IdentityDerivationDenial),
    MissingIdentityParts,
    BlankIdentityKey,
    BlankIdentityValue { key: String },
    DuplicateIdentityKey { key: String },
}

fn admit_identity_parts(
    mut parts: Vec<WorthQueryAsyncRequestIdentityPart>,
) -> Result<Vec<WorthQueryAsyncRequestIdentityPart>, WorthQueryAsyncResourceRequestIdentityError> {
    if parts.is_empty() {
        return Err(WorthQueryAsyncResourceRequestIdentityError::MissingIdentityParts);
    }
    parts.sort_by(|left, right| left.key().cmp(right.key()));
    let mut prior_key: Option<&str> = None;
    for part in &parts {
        let key = part.key().trim();
        if key.is_empty() {
            return Err(WorthQueryAsyncResourceRequestIdentityError::BlankIdentityKey);
        }
        if part.has_blank_text_value() {
            return Err(
                WorthQueryAsyncResourceRequestIdentityError::BlankIdentityValue {
                    key: key.to_string(),
                },
            );
        }
        if prior_key == Some(key) {
            return Err(
                WorthQueryAsyncResourceRequestIdentityError::DuplicateIdentityKey {
                    key: key.to_string(),
                },
            );
        }
        prior_key = Some(key);
    }
    Ok(parts)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn identity_is_order_independent_and_semantic_drift_sensitive() {
        let left = identity(vec![part("source", "users"), part("selector", "owner")]);
        let right = identity(vec![part("selector", "owner"), part("source", "users")]);
        let changed = identity(vec![part("source", "users"), part("selector", "approver")]);

        assert_eq!(left.canonical_identity(), right.canonical_identity());
        assert_ne!(left.canonical_identity(), changed.canonical_identity());
    }

    #[test]
    fn ambiguous_or_blank_identity_parts_are_denied() {
        assert_eq!(
            WorthQueryAsyncResourceRequestIdentity::declare(
                WorthQueryAsyncSourceFamily::HostResource,
                WorthQueryAsyncLoadingPosture::Blocking,
                WorthQueryAsyncFailurePosture::FailClosed,
                vec![part("source", "users"), part("source", "other")],
            ),
            Err(
                WorthQueryAsyncResourceRequestIdentityError::DuplicateIdentityKey {
                    key: "source".to_string(),
                }
            )
        );
        assert!(matches!(
            WorthQueryAsyncResourceRequestIdentity::declare(
                WorthQueryAsyncSourceFamily::HostResource,
                WorthQueryAsyncLoadingPosture::Blocking,
                WorthQueryAsyncFailurePosture::FailClosed,
                vec![part("source", " ")],
            ),
            Err(WorthQueryAsyncResourceRequestIdentityError::BlankIdentityValue { .. })
        ));
    }

    fn identity(
        parts: Vec<WorthQueryAsyncRequestIdentityPart>,
    ) -> WorthQueryAsyncResourceRequestIdentity {
        WorthQueryAsyncResourceRequestIdentity::declare(
            WorthQueryAsyncSourceFamily::HostResource,
            WorthQueryAsyncLoadingPosture::Blocking,
            WorthQueryAsyncFailurePosture::FailClosed,
            parts,
        )
        .expect("identity should admit")
    }

    fn part(key: &str, value: &str) -> WorthQueryAsyncRequestIdentityPart {
        WorthQueryAsyncRequestIdentityPart::text(key, value)
    }
}

impl From<IdentityDerivationDenial> for WorthQueryAsyncResourceRequestIdentityError {
    fn from(cause: IdentityDerivationDenial) -> Self {
        Self::EvidenceIdentityDerivationDenied(cause)
    }
}
