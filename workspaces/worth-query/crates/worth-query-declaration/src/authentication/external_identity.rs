use worth_foundational::facade::{AspectValue, InternedString, ScalarAspectType};

use crate::application_schema::{
    ApplicationIdentityScalarValueBinding, ApplicationReadableScalarValueBinding,
    ApplicationScalarValueBinding, ApplicationValueDecodeAvailable, ApplicationValueDecodeDenial,
    ApplicationValueEncodeDenial, ApplicationValueIsIdentity,
    ApplicationValueSignedAggregateUnavailable, ApplicationValueValidationDenial,
};

const MAX_ISSUER_BYTES: usize = 2_048;
const MAX_SUBJECT_BYTES: usize = 1_024;

/// Stable external identity carried across authentication adapters.
///
/// Issuer and subject remain separate typed components. The Foundational value
/// conversion uses a length-delimited encoding so one equality index can
/// resolve the pair without a lossy digest or issuer-wide scan.
#[derive(Clone, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct WorthQueryExternalPrincipalIdentity {
    issuer: String,
    subject: String,
}
crate::worth_query_portable_type!(
    WorthQueryExternalPrincipalIdentity => "worth.query.external_principal_identity.v1"
);

impl std::fmt::Debug for WorthQueryExternalPrincipalIdentity {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("WorthQueryExternalPrincipalIdentity")
            .finish_non_exhaustive()
    }
}

impl WorthQueryExternalPrincipalIdentity {
    pub fn new(
        issuer: impl Into<String>,
        subject: impl Into<String>,
    ) -> Result<Self, WorthQueryExternalPrincipalIdentityDenial> {
        let issuer = issuer.into();
        let subject = subject.into();
        validate_component("issuer", &issuer, MAX_ISSUER_BYTES)?;
        validate_component("subject", &subject, MAX_SUBJECT_BYTES)?;
        Ok(Self { issuer, subject })
    }

    pub fn issuer(&self) -> &str {
        &self.issuer
    }

    pub fn subject(&self) -> &str {
        &self.subject
    }

    fn into_index_value(self) -> String {
        format!(
            "{}:{}{}:{}",
            self.issuer.len(),
            self.issuer,
            self.subject.len(),
            self.subject
        )
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct WorthQueryExternalPrincipalIdentityBinding;

impl ApplicationScalarValueBinding for WorthQueryExternalPrincipalIdentityBinding {
    type Value = WorthQueryExternalPrincipalIdentity;
    type Unit = ();
    type Decode = ApplicationValueDecodeAvailable;
    type Identity = ApplicationValueIsIdentity;
    type SignedAggregate = ApplicationValueSignedAggregateUnavailable;

    const IDENTITY_NAME: &'static str = "worth.query.external_principal_identity.v1";
    const SCALAR_FAMILY: ScalarAspectType = ScalarAspectType::String;

    fn validate(_: &Self::Value) -> Result<(), ApplicationValueValidationDenial> {
        Ok(())
    }

    fn encode(value: &Self::Value) -> Result<AspectValue, ApplicationValueEncodeDenial> {
        Ok(AspectValue::String(InternedString::from(
            value.clone().into_index_value(),
        )))
    }
}

impl ApplicationReadableScalarValueBinding for WorthQueryExternalPrincipalIdentityBinding {
    fn decode(value: &AspectValue) -> Result<Self::Value, ApplicationValueDecodeDenial> {
        let AspectValue::String(InternedString::Raw(encoded)) = value else {
            return Err(ApplicationValueDecodeDenial::ScalarFamilyMismatch {
                binding_identity: Self::IDENTITY,
                expected: Self::SCALAR_FAMILY,
                observed: value.value_family(),
            });
        };
        decode_index_value(encoded).ok_or(ApplicationValueDecodeDenial::CodecRejected {
            binding_identity: Self::IDENTITY,
        })
    }
}

impl ApplicationIdentityScalarValueBinding for WorthQueryExternalPrincipalIdentityBinding {}

fn decode_index_value(encoded: &str) -> Option<WorthQueryExternalPrincipalIdentity> {
    let mut offset = 0;
    let issuer = decode_index_component(encoded, &mut offset)?;
    let subject = decode_index_component(encoded, &mut offset)?;
    (offset == encoded.len())
        .then(|| WorthQueryExternalPrincipalIdentity::new(issuer, subject).ok())
        .flatten()
}

fn decode_index_component<'a>(encoded: &'a str, offset: &mut usize) -> Option<&'a str> {
    let suffix = encoded.get(*offset..)?;
    let delimiter = suffix.find(':')?;
    let length = suffix.get(..delimiter)?.parse::<usize>().ok()?;
    let start = offset.checked_add(delimiter)?.checked_add(1)?;
    let end = start.checked_add(length)?;
    let component = encoded.get(start..end)?;
    *offset = end;
    Some(component)
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WorthQueryExternalPrincipalIdentityDenialKind {
    Empty,
    SurroundingWhitespace,
    ControlCharacter,
    TooLong,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorthQueryExternalPrincipalIdentityDenial {
    component: &'static str,
    kind: WorthQueryExternalPrincipalIdentityDenialKind,
}

impl WorthQueryExternalPrincipalIdentityDenial {
    pub const fn component(&self) -> &'static str {
        self.component
    }

    pub const fn kind(&self) -> WorthQueryExternalPrincipalIdentityDenialKind {
        self.kind
    }
}

impl std::fmt::Display for WorthQueryExternalPrincipalIdentityDenial {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            formatter,
            "external principal {} denied: {:?}",
            self.component, self.kind
        )
    }
}

impl std::error::Error for WorthQueryExternalPrincipalIdentityDenial {}

fn validate_component(
    component: &'static str,
    value: &str,
    maximum_bytes: usize,
) -> Result<(), WorthQueryExternalPrincipalIdentityDenial> {
    let kind = if value.is_empty() {
        Some(WorthQueryExternalPrincipalIdentityDenialKind::Empty)
    } else if value.trim() != value {
        Some(WorthQueryExternalPrincipalIdentityDenialKind::SurroundingWhitespace)
    } else if value.chars().any(char::is_control) {
        Some(WorthQueryExternalPrincipalIdentityDenialKind::ControlCharacter)
    } else if value.len() > maximum_bytes {
        Some(WorthQueryExternalPrincipalIdentityDenialKind::TooLong)
    } else {
        None
    };
    match kind {
        Some(kind) => Err(WorthQueryExternalPrincipalIdentityDenial { component, kind }),
        None => Ok(()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn length_delimited_pairs_do_not_alias() {
        let left = WorthQueryExternalPrincipalIdentity::new("ab", "c")
            .unwrap()
            .into_index_value();
        let right = WorthQueryExternalPrincipalIdentity::new("a", "bc")
            .unwrap()
            .into_index_value();
        assert_ne!(left, right);
    }

    #[test]
    fn binding_round_trips_the_exact_issuer_and_subject() {
        let identity =
            WorthQueryExternalPrincipalIdentity::new("https://issuer.example", "subject-123")
                .unwrap();
        let encoded = WorthQueryExternalPrincipalIdentityBinding::encode(&identity).unwrap();
        assert_eq!(
            WorthQueryExternalPrincipalIdentityBinding::decode(&encoded).unwrap(),
            identity
        );
    }

    #[test]
    fn invalid_components_fail_before_index_encoding() {
        for (value, expected) in [
            ("", WorthQueryExternalPrincipalIdentityDenialKind::Empty),
            (
                " issuer",
                WorthQueryExternalPrincipalIdentityDenialKind::SurroundingWhitespace,
            ),
            (
                "issuer\n",
                WorthQueryExternalPrincipalIdentityDenialKind::SurroundingWhitespace,
            ),
        ] {
            assert_eq!(
                WorthQueryExternalPrincipalIdentity::new(value, "subject")
                    .unwrap_err()
                    .kind(),
                expected
            );
        }
    }

    #[test]
    fn control_characters_and_overlength_components_fail_at_the_typed_boundary() {
        for (issuer, subject, component, kind) in [
            (
                "issuer\u{0001}".to_string(),
                "subject".to_string(),
                "issuer",
                WorthQueryExternalPrincipalIdentityDenialKind::ControlCharacter,
            ),
            (
                "issuer".to_string(),
                "subject\u{007f}".to_string(),
                "subject",
                WorthQueryExternalPrincipalIdentityDenialKind::ControlCharacter,
            ),
            (
                "i".repeat(MAX_ISSUER_BYTES + 1),
                "subject".to_string(),
                "issuer",
                WorthQueryExternalPrincipalIdentityDenialKind::TooLong,
            ),
            (
                "issuer".to_string(),
                "s".repeat(MAX_SUBJECT_BYTES + 1),
                "subject",
                WorthQueryExternalPrincipalIdentityDenialKind::TooLong,
            ),
        ] {
            let denial = WorthQueryExternalPrincipalIdentity::new(issuer, subject).unwrap_err();
            assert_eq!(denial.component(), component);
            assert_eq!(denial.kind(), kind);
        }
    }

    #[test]
    fn debug_output_discloses_neither_issuer_nor_subject() {
        let identity =
            WorthQueryExternalPrincipalIdentity::new("https://issuer.example", "subject-123")
                .unwrap();
        let debug = format!("{identity:?}");
        assert!(!debug.contains("https://issuer.example"));
        assert!(!debug.contains("subject-123"));
    }
}
