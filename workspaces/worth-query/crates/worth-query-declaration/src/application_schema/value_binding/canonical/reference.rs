//! Complete descriptive codec identity; typed installation remains the authority.
use super::ApplicationCanonicalValueContract;

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct ApplicationCanonicalValueReferenceParts {
    pub protocol: String,
    pub shape: String,
    pub maximum_wire_bytes: Option<u64>,
    pub maximum_string_bytes: Option<u64>,
    pub maximum_sequence_length: Option<u64>,
    pub maximum_depth: Option<u64>,
    pub supports_admission: bool,
}

/// Shape text is the existing ValueShape serializer's complete projection. It
/// is never decoded into a shape or used to construct a value decoder.
#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct ApplicationCanonicalValueReference {
    parts: ApplicationCanonicalValueReferenceParts,
}

impl ApplicationCanonicalValueReference {
    pub fn from_untrusted_parts(parts: ApplicationCanonicalValueReferenceParts) -> Self {
        Self { parts }
    }

    pub fn parts(&self) -> &ApplicationCanonicalValueReferenceParts {
        &self.parts
    }

    pub fn matches_contract(&self, contract: &ApplicationCanonicalValueContract) -> bool {
        self == &contract.portable_reference()
    }
}

impl ApplicationCanonicalValueContract {
    pub fn portable_reference(&self) -> ApplicationCanonicalValueReference {
        let bounds = self.bounds();
        ApplicationCanonicalValueReference {
            parts: ApplicationCanonicalValueReferenceParts {
                protocol: self.wire_protocol().to_owned(),
                shape: serde_json::to_string(self.shape())
                    .expect("descriptive ValueShape contains only serializable grammar"),
                maximum_wire_bytes: bounds.maximum_wire_bytes().map(|value| value as u64),
                maximum_string_bytes: bounds.maximum_string_bytes().map(|value| value as u64),
                maximum_sequence_length: bounds.maximum_sequence_length().map(|value| value as u64),
                maximum_depth: bounds.maximum_depth().map(|value| value as u64),
                supports_admission: self.supports_admission(),
            },
        }
    }
}
