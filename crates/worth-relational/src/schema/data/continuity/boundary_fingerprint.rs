use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct SchemaBoundaryFingerprint(pub [u8; 32]);

impl SchemaBoundaryFingerprint {
    pub const ZERO: Self = Self([0; 32]);

    pub fn new(bytes: [u8; 32]) -> Self {
        Self(bytes)
    }
}

impl Serialize for SchemaBoundaryFingerprint {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        worth_foundational::CanonicalDigestId::new(self.0).serialize(serializer)
    }
}
impl<'de> Deserialize<'de> for SchemaBoundaryFingerprint {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        worth_foundational::CanonicalDigestId::deserialize(deserializer)
            .map(|digest| Self(*digest.bytes()))
    }
}
