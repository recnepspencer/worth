use serde::{de::Error as DeError, Deserialize, Deserializer, Serialize};

use crate::commit_strategies::data::canonical_digest::commit_strategy_descriptor_digest;
use crate::commit_strategies::data::strategy_id::CommitStrategyId;

use super::read_contract::{CommitStrategyVersion, StrategyReadContract};
use super::semantic_names::{
    CommitStrategyFamilyName, CommitStrategySemanticName, PersistentArtifactName,
    StrategyInputSchemaName, StrategyInputSchemaVersion, StrategyIntentName,
    StrategyOutputSchemaName,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct CommitStrategyDescriptorDigest(pub [u8; 32]);

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct CommitStrategyDescriptor {
    id: CommitStrategyId,
    semantic_name: CommitStrategySemanticName,
    family_name: CommitStrategyFamilyName,
    version: CommitStrategyVersion,
    intent_name: StrategyIntentName,
    input_schema_name: StrategyInputSchemaName,
    input_schema_version: StrategyInputSchemaVersion,
    output_schema_name: StrategyOutputSchemaName,
    read_contract: StrategyReadContract,
    artifact_name: PersistentArtifactName,
    digest: CommitStrategyDescriptorDigest,
}

impl CommitStrategyDescriptor {
    pub fn new(
        id: CommitStrategyId,
        semantic_name: CommitStrategySemanticName,
        family_name: CommitStrategyFamilyName,
        version: CommitStrategyVersion,
        intent_name: StrategyIntentName,
        input_schema_name: StrategyInputSchemaName,
        input_schema_version: StrategyInputSchemaVersion,
        output_schema_name: StrategyOutputSchemaName,
        read_contract: StrategyReadContract,
        artifact_name: PersistentArtifactName,
    ) -> Self {
        let digest = commit_strategy_descriptor_digest(
            id,
            &semantic_name,
            &family_name,
            version,
            &intent_name,
            &input_schema_name,
            input_schema_version,
            &output_schema_name,
            &read_contract,
            &artifact_name,
        );
        Self {
            id,
            semantic_name,
            family_name,
            version,
            intent_name,
            input_schema_name,
            input_schema_version,
            output_schema_name,
            read_contract,
            artifact_name,
            digest,
        }
    }

    pub fn id(&self) -> CommitStrategyId {
        self.id
    }

    pub fn semantic_name(&self) -> &CommitStrategySemanticName {
        &self.semantic_name
    }

    pub fn family_name(&self) -> &CommitStrategyFamilyName {
        &self.family_name
    }

    pub fn version(&self) -> CommitStrategyVersion {
        self.version
    }

    pub fn intent_name(&self) -> &StrategyIntentName {
        &self.intent_name
    }

    pub fn input_schema_name(&self) -> &StrategyInputSchemaName {
        &self.input_schema_name
    }

    pub fn input_schema_version(&self) -> StrategyInputSchemaVersion {
        self.input_schema_version
    }

    pub fn output_schema_name(&self) -> &StrategyOutputSchemaName {
        &self.output_schema_name
    }

    pub fn read_contract(&self) -> &StrategyReadContract {
        &self.read_contract
    }

    pub fn artifact_name(&self) -> &PersistentArtifactName {
        &self.artifact_name
    }

    pub fn digest(&self) -> CommitStrategyDescriptorDigest {
        self.digest
    }
}

impl<'de> Deserialize<'de> for CommitStrategyDescriptor {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let raw = RawCommitStrategyDescriptor::deserialize(deserializer)?;
        let expected = commit_strategy_descriptor_digest(
            raw.id,
            &raw.semantic_name,
            &raw.family_name,
            raw.version,
            &raw.intent_name,
            &raw.input_schema_name,
            raw.input_schema_version,
            &raw.output_schema_name,
            &raw.read_contract,
            &raw.artifact_name,
        );
        if raw.digest != expected {
            return Err(D::Error::custom(
                "commit strategy descriptor digest does not match descriptor contents",
            ));
        }
        Ok(raw.into_descriptor())
    }
}

#[derive(Deserialize)]
struct RawCommitStrategyDescriptor {
    id: CommitStrategyId,
    semantic_name: CommitStrategySemanticName,
    family_name: CommitStrategyFamilyName,
    version: CommitStrategyVersion,
    intent_name: StrategyIntentName,
    input_schema_name: StrategyInputSchemaName,
    input_schema_version: StrategyInputSchemaVersion,
    output_schema_name: StrategyOutputSchemaName,
    read_contract: StrategyReadContract,
    artifact_name: PersistentArtifactName,
    digest: CommitStrategyDescriptorDigest,
}

impl RawCommitStrategyDescriptor {
    fn into_descriptor(self) -> CommitStrategyDescriptor {
        CommitStrategyDescriptor {
            id: self.id,
            semantic_name: self.semantic_name,
            family_name: self.family_name,
            version: self.version,
            intent_name: self.intent_name,
            input_schema_name: self.input_schema_name,
            input_schema_version: self.input_schema_version,
            output_schema_name: self.output_schema_name,
            read_contract: self.read_contract,
            artifact_name: self.artifact_name,
            digest: self.digest,
        }
    }
}

impl Serialize for CommitStrategyDescriptorDigest {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        worth_foundational::CanonicalDigestId::new(self.0).serialize(serializer)
    }
}
impl<'de> Deserialize<'de> for CommitStrategyDescriptorDigest {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        worth_foundational::CanonicalDigestId::deserialize(deserializer)
            .map(|digest| Self(*digest.bytes()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::commit_strategies::data::{
        StrategyPacketContract, StrategyReadCostClass, StrategyReadLocalityClass,
        StrategyReadScopeClass, StrategyTraversalBasis,
    };

    fn descriptor() -> CommitStrategyDescriptor {
        CommitStrategyDescriptor::new(
            CommitStrategyId(11),
            CommitStrategySemanticName::new("strategy.intent.reconcile"),
            CommitStrategyFamilyName::new("strategy.intent"),
            CommitStrategyVersion::new(1, 0),
            StrategyIntentName::new("reconcile.desired.state"),
            StrategyInputSchemaName::new("intent.input.v1"),
            StrategyInputSchemaVersion(1),
            StrategyOutputSchemaName::new("intent.output.v1"),
            StrategyReadContract {
                scope_class: StrategyReadScopeClass::ExplicitTargetsOnly,
                locality_class: StrategyReadLocalityClass::SinglePartition,
                traversal_basis: StrategyTraversalBasis::NoTraversal,
                packet_contract: StrategyPacketContract::ProjectionOnly,
                cost_class: StrategyReadCostClass::ORequestedSurface,
            },
            PersistentArtifactName::new("strategy.intent.reconcile"),
        )
    }

    #[test]
    fn descriptor_constructor_preserves_verified_digest() {
        let descriptor = descriptor();
        let recomputed = CommitStrategyDescriptor::new(
            descriptor.id(),
            descriptor.semantic_name().clone(),
            descriptor.family_name().clone(),
            descriptor.version(),
            descriptor.intent_name().clone(),
            descriptor.input_schema_name().clone(),
            descriptor.input_schema_version(),
            descriptor.output_schema_name().clone(),
            descriptor.read_contract().clone(),
            descriptor.artifact_name().clone(),
        );

        assert_eq!(descriptor.digest(), recomputed.digest());
    }

    #[test]
    fn descriptor_digest_drift_is_detectable_with_typed_fixture() {
        let mut forged = descriptor();
        let original_digest = forged.digest();
        forged.semantic_name = CommitStrategySemanticName::new("strategy.intent.tampered");
        let recomputed = CommitStrategyDescriptor::new(
            forged.id(),
            forged.semantic_name().clone(),
            forged.family_name().clone(),
            forged.version(),
            forged.intent_name().clone(),
            forged.input_schema_name().clone(),
            forged.input_schema_version(),
            forged.output_schema_name().clone(),
            forged.read_contract().clone(),
            forged.artifact_name().clone(),
        );

        assert_ne!(original_digest, recomputed.digest());
        assert_eq!(forged.digest(), original_digest);
    }
}
