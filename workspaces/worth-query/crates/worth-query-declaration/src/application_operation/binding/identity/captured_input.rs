//! The exact serialization that derived an input identity, not a second encoding.
mod scalar_record;

use super::{ApplicationCanonicalIdentity, ApplicationCanonicalWork};
use std::sync::Arc;
use worth_foundational::facade::{AspectValue, InternedString, ScalarAspectType};

/// Read-only captured input data. Only native identity encoding constructs it.
/// It carries no authorization, world observation or currentness authority.
#[derive(Clone, Debug)]
pub struct ApplicationCanonicalInput {
    bytes: Arc<Vec<u8>>,
    value_offset: usize,
}

/// The current selector form reads scalar members of a serialized record.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ApplicationCanonicalInputDenial {
    UnsupportedShape,
    MissingMember,
    DuplicateMember,
    InvalidScalar,
}

/// A borrowed scalar selected from the captured encoding. Native execution
/// reserves its retained representation before materializing text in decision.
pub enum ApplicationCanonicalInputScalar<'a> {
    Inline(AspectValue),
    Text(&'a str),
}

#[cfg(test)]
mod tests {
    use super::super::{canonical_identity, canonical_identity_with_capture, encoder};
    use serde::{ser::SerializeStruct, Serialize, Serializer};
    use std::collections::BTreeMap;
    use worth_foundational::facade::ScalarAspectType;

    #[test]
    fn capture_is_the_final_canonical_stream_without_changing_its_digest() {
        #[derive(Serialize)]
        struct Input {
            members: BTreeMap<String, u64>,
        }
        let input = Input {
            members: BTreeMap::from([("z".to_owned(), 3), ("a".to_owned(), 9)]),
        };
        let ordinary = canonical_identity("domain", "scope", &input).unwrap();
        let captured = canonical_identity_with_capture(
            "domain",
            "scope",
            &input,
            encoder::HashingSink::new().capturing(true),
        )
        .unwrap();
        assert_eq!(ordinary.identity(), captured.canonical.identity());
        let snapshot = captured.capture.unwrap();
        let mut separately_encoded = Vec::new();
        encoder::encode_into(&mut separately_encoded, &input).unwrap();
        assert_eq!(
            &snapshot.bytes[snapshot.value_offset..],
            &separately_encoded
        );
    }

    struct RepeatedMember;
    impl Serialize for RepeatedMember {
        fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
            let mut record = serializer.serialize_struct("RepeatedMember", 2)?;
            record.serialize_field("retention_days", &7_u64)?;
            record.serialize_field("retention_days", &9_u64)?;
            record.end()
        }
    }

    #[test]
    fn a_later_duplicate_selected_member_is_refused() {
        let captured = canonical_identity_with_capture(
            "domain",
            "scope",
            &RepeatedMember,
            encoder::HashingSink::new().capturing(true),
        )
        .unwrap()
        .capture
        .unwrap();
        assert!(matches!(
            captured.scalar_member("retention_days", ScalarAspectType::UInt64),
            Err(super::ApplicationCanonicalInputDenial::DuplicateMember)
        ));
    }

    #[test]
    fn a_float_payload_that_native_storage_would_normalize_is_refused() {
        #[derive(Serialize)]
        struct Input {
            measure: f32,
        }
        let input = Input {
            measure: f32::from_bits(0x7fc0_1234),
        };
        let captured = canonical_identity_with_capture(
            "domain",
            "scope",
            &input,
            encoder::HashingSink::new().capturing(true),
        )
        .unwrap()
        .capture
        .unwrap();
        assert!(matches!(
            captured.scalar_member("measure", ScalarAspectType::Float32),
            Err(super::ApplicationCanonicalInputDenial::InvalidScalar)
        ));
    }
}
impl ApplicationCanonicalInputScalar<'_> {
    pub fn owned_allocation_capacity_bytes(&self) -> usize {
        match self {
            Self::Inline(value) => value.owned_allocation_capacity_bytes(),
            Self::Text(value) => value.len(),
        }
    }
    pub fn into_value(self) -> AspectValue {
        match self {
            Self::Inline(value) => value,
            Self::Text(value) => {
                AspectValue::String(InternedString::Raw(String::from(Box::<str>::from(value))))
            }
        }
    }
}

pub(super) struct EncodedInputIdentity {
    pub(super) canonical: ApplicationCanonicalIdentity,
    pub(super) capture: Option<ApplicationCanonicalInput>,
}
impl EncodedInputIdentity {
    pub(super) fn identity(&self) -> [u8; 32] {
        self.canonical.identity()
    }
    pub(super) fn work(&self) -> ApplicationCanonicalWork {
        self.canonical.work()
    }
}

impl ApplicationCanonicalInput {
    pub fn encoded_byte_length(&self) -> usize {
        self.bytes.len() - self.value_offset
    }
    pub(super) fn new(bytes: Vec<u8>, value_offset: usize) -> Self {
        Self {
            bytes: Arc::new(bytes),
            value_offset,
        }
    }

    /// Selects from the original serialized record, never from the live Rust
    /// object. Unsupported or ambiguous representations deny, not coerce.
    pub fn scalar_member(
        &self,
        member: &str,
        family: ScalarAspectType,
    ) -> Result<ApplicationCanonicalInputScalar<'_>, ApplicationCanonicalInputDenial> {
        scalar_record::select(&self.bytes[self.value_offset..], member, family)
    }
}
