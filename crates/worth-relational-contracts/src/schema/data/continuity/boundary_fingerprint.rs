use serde::{Deserialize, Serialize};
use worth_foundational::facade::CanonicalDigestId;

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct SchemaBoundaryFingerprint(CanonicalDigestId);

impl SchemaBoundaryFingerprint {
    pub const ZERO: Self = Self::new([0; 32]);

    pub const fn new(bytes: [u8; 32]) -> Self {
        Self(CanonicalDigestId::new(bytes))
    }

    pub const fn bytes(&self) -> &[u8; 32] {
        self.0.bytes()
    }
}

impl std::fmt::Debug for SchemaBoundaryFingerprint {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_tuple("SchemaBoundaryFingerprint")
            .field(self.bytes())
            .finish()
    }
}
