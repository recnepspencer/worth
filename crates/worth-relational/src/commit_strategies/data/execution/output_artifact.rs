use std::sync::Arc;

use serde::{de::Error as DeError, Deserialize, Deserializer, Serialize};
use sha2::{Digest, Sha256};

use crate::commit_strategies::data::{PersistentArtifactName, StrategyOutputSchemaName};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct CanonicalStrategyOutputDigest(pub [u8; 32]);

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct CanonicalStrategyOutputArtifact {
    schema_name: StrategyOutputSchemaName,
    canonical_bytes: Arc<[u8]>,
    digest: CanonicalStrategyOutputDigest,
    artifact_name: PersistentArtifactName,
}

impl CanonicalStrategyOutputArtifact {
    pub fn new(
        schema_name: StrategyOutputSchemaName,
        canonical_bytes: impl Into<Arc<[u8]>>,
        artifact_name: PersistentArtifactName,
    ) -> Self {
        let canonical_bytes = canonical_bytes.into();
        Self {
            schema_name,
            digest: compute_output_digest(&canonical_bytes),
            canonical_bytes,
            artifact_name,
        }
    }

    pub fn schema_name(&self) -> &StrategyOutputSchemaName {
        &self.schema_name
    }

    pub fn canonical_bytes(&self) -> &[u8] {
        &self.canonical_bytes
    }

    pub fn digest(&self) -> CanonicalStrategyOutputDigest {
        self.digest
    }

    pub fn artifact_name(&self) -> &PersistentArtifactName {
        &self.artifact_name
    }
}

impl<'de> Deserialize<'de> for CanonicalStrategyOutputArtifact {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        #[derive(Deserialize)]
        struct RawCanonicalStrategyOutputArtifact {
            schema_name: StrategyOutputSchemaName,
            canonical_bytes: Arc<[u8]>,
            digest: CanonicalStrategyOutputDigest,
            artifact_name: PersistentArtifactName,
        }

        let raw = RawCanonicalStrategyOutputArtifact::deserialize(deserializer)?;
        let expected_digest = compute_output_digest(&raw.canonical_bytes);
        if raw.digest != expected_digest {
            return Err(D::Error::custom(
                "strategy output digest does not match canonical output bytes",
            ));
        }
        Ok(Self {
            schema_name: raw.schema_name,
            canonical_bytes: raw.canonical_bytes,
            digest: raw.digest,
            artifact_name: raw.artifact_name,
        })
    }
}

pub(super) fn compute_output_digest(canonical_bytes: &[u8]) -> CanonicalStrategyOutputDigest {
    CanonicalStrategyOutputDigest(Sha256::digest(canonical_bytes).into())
}

#[cfg(test)]
pub(super) fn forged_output_artifact_for_digest_test(
    artifact: &CanonicalStrategyOutputArtifact,
    canonical_bytes: Arc<[u8]>,
) -> CanonicalStrategyOutputArtifact {
    CanonicalStrategyOutputArtifact {
        schema_name: artifact.schema_name().clone(),
        canonical_bytes,
        digest: artifact.digest(),
        artifact_name: artifact.artifact_name().clone(),
    }
}

impl Serialize for CanonicalStrategyOutputDigest {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        worth_foundational::CanonicalDigestId::new(self.0).serialize(serializer)
    }
}
impl<'de> Deserialize<'de> for CanonicalStrategyOutputDigest {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        worth_foundational::CanonicalDigestId::deserialize(deserializer)
            .map(|digest| Self(*digest.bytes()))
    }
}
