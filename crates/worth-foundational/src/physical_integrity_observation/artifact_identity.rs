use serde::{Deserialize, Serialize};

const MAX_PHYSICAL_ARTIFACT_IDENTITY_BYTES: usize = 512;

/// Stable descriptive identity for one persisted artifact or framed range.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct PhysicalArtifactIdentity(Box<str>);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PhysicalArtifactIdentityDenial {
    Empty,
    TooLong,
    ContainsControlCharacter,
}

impl PhysicalArtifactIdentity {
    pub fn new(value: impl Into<Box<str>>) -> Result<Self, PhysicalArtifactIdentityDenial> {
        let value = value.into();
        if value.is_empty() {
            return Err(PhysicalArtifactIdentityDenial::Empty);
        }
        if value.len() > MAX_PHYSICAL_ARTIFACT_IDENTITY_BYTES {
            return Err(PhysicalArtifactIdentityDenial::TooLong);
        }
        if value.chars().any(char::is_control) {
            return Err(PhysicalArtifactIdentityDenial::ContainsControlCharacter);
        }
        Ok(Self(value))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}
