use serde::{Deserialize, Serialize};
use worth_foundational::facade::AspectValue;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct AuthoritativeFieldComparisonKey {
    canonical_value_bytes: Vec<u8>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum AuthoritativeFieldComparisonKeyDecodeDenialKind {
    InvalidEncoding,
    NonCanonicalEncoding,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AuthoritativeFieldComparisonKeyDecodeDenial {
    kind: AuthoritativeFieldComparisonKeyDecodeDenialKind,
    detail: String,
}

impl AuthoritativeFieldComparisonKeyDecodeDenial {
    fn new(
        kind: AuthoritativeFieldComparisonKeyDecodeDenialKind,
        detail: impl Into<String>,
    ) -> Self {
        Self {
            kind,
            detail: detail.into(),
        }
    }

    pub const fn kind(&self) -> AuthoritativeFieldComparisonKeyDecodeDenialKind {
        self.kind
    }

    pub fn detail(&self) -> &str {
        &self.detail
    }
}

impl std::fmt::Display for AuthoritativeFieldComparisonKeyDecodeDenial {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(&self.detail)
    }
}

impl std::error::Error for AuthoritativeFieldComparisonKeyDecodeDenial {}

impl AuthoritativeFieldComparisonKey {
    /// Backing allocation needed by the native canonical comparison encoding.
    /// A caller can admit this before constructing a key.
    pub fn required_encoded_capacity_bytes(value: &AspectValue) -> Option<u64> {
        crate::aspect_wire::encoded_aspect_value_len(value)
            .and_then(|bytes| u64::try_from(bytes).ok())
    }

    pub fn from_aspect_value(value: &AspectValue) -> Self {
        Self {
            canonical_value_bytes: canonical_aspect_value_bytes(value),
        }
    }

    pub fn canonical_value_bytes(&self) -> &[u8] {
        &self.canonical_value_bytes
    }

    pub fn value_from_canonical_bytes(
        bytes: &[u8],
    ) -> Result<AspectValue, AuthoritativeFieldComparisonKeyDecodeDenial> {
        let value = crate::aspect_wire::decode_aspect_value(bytes).map_err(|error| {
            AuthoritativeFieldComparisonKeyDecodeDenial::new(
                AuthoritativeFieldComparisonKeyDecodeDenialKind::InvalidEncoding,
                error.to_string(),
            )
        })?;
        if crate::aspect_wire::encode_aspect_value(&value) != bytes {
            return Err(AuthoritativeFieldComparisonKeyDecodeDenial::new(
                AuthoritativeFieldComparisonKeyDecodeDenialKind::NonCanonicalEncoding,
                "field comparison key is not canonical",
            ));
        }
        Ok(value)
    }

    pub fn owned_allocation_capacity_bytes(&self) -> u64 {
        self.canonical_value_bytes.capacity() as u64
    }
}

pub fn authoritative_aspect_value_field_comparison_key(
    value: &AspectValue,
) -> AuthoritativeFieldComparisonKey {
    AuthoritativeFieldComparisonKey::from_aspect_value(value)
}

fn canonical_aspect_value_bytes(value: &AspectValue) -> Vec<u8> {
    crate::aspect_wire::encode_aspect_value(value)
}

#[cfg(test)]
mod tests {
    use worth_foundational::facade::{AspectValue, CanonicalF64, InternedString};

    use super::{AuthoritativeFieldComparisonKey, AuthoritativeFieldComparisonKeyDecodeDenialKind};

    #[test]
    fn comparison_key_preserves_aspect_value_family_even_when_display_collides() {
        let int_key = AuthoritativeFieldComparisonKey::from_aspect_value(&AspectValue::Int64(1));
        let string_key = AuthoritativeFieldComparisonKey::from_aspect_value(&AspectValue::String(
            InternedString::Raw("1".to_string()),
        ));

        assert_ne!(int_key, string_key);
        assert_ne!(
            int_key.canonical_value_bytes(),
            string_key.canonical_value_bytes()
        );
    }

    #[test]
    fn comparison_key_reports_owned_capacity_instead_of_initialized_length() {
        let mut canonical_value_bytes = Vec::with_capacity(128);
        canonical_value_bytes.extend_from_slice(b"small");
        let key = AuthoritativeFieldComparisonKey {
            canonical_value_bytes,
        };

        assert_eq!(key.canonical_value_bytes().len(), 5);
        assert_eq!(key.owned_allocation_capacity_bytes(), 128);
        assert_ne!(
            key.owned_allocation_capacity_bytes(),
            key.canonical_value_bytes().len() as u64,
            "initialized length cannot stand in for an owned allocation's capacity"
        );
    }

    #[test]
    fn signed_zero_has_distinct_native_index_keys() {
        let negative = AuthoritativeFieldComparisonKey::from_aspect_value(&AspectValue::Float64(
            CanonicalF64::from_f64(-0.0),
        ));
        let positive = AuthoritativeFieldComparisonKey::from_aspect_value(&AspectValue::Float64(
            CanonicalF64::from_f64(0.0),
        ));
        assert_ne!(negative, positive);
    }

    #[test]
    fn native_key_capacity_is_known_before_encoding_and_round_trips_values() {
        let values = [
            AspectValue::Null,
            AspectValue::Float64(CanonicalF64::from_f64(-0.0)),
            AspectValue::String(InternedString::Raw("scoped-key".to_owned())),
        ];
        for value in values {
            let required = AuthoritativeFieldComparisonKey::required_encoded_capacity_bytes(&value)
                .expect("canonical value has an exact encoded bound");
            let key = AuthoritativeFieldComparisonKey::from_aspect_value(&value);
            assert_eq!(key.owned_allocation_capacity_bytes(), required);
            assert_eq!(
                AuthoritativeFieldComparisonKey::value_from_canonical_bytes(
                    key.canonical_value_bytes()
                )
                .expect("native value decodes"),
                value,
            );
        }
        let denial = AuthoritativeFieldComparisonKey::value_from_canonical_bytes(&[0xff])
            .expect_err("unknown native tags must be rejected");
        assert_eq!(
            denial.kind(),
            AuthoritativeFieldComparisonKeyDecodeDenialKind::InvalidEncoding
        );
        assert!(!denial.detail().is_empty());
    }
}
