use std::sync::Arc;

use serde::{Deserialize, Serialize};

mod canonical_bytes;

use super::{
    CommitStrategyDescriptorDigest, CommitStrategyId, CommitStrategySemanticName,
    PersistentArtifactName, StrategyInputSchemaName, StrategyInputSchemaVersion,
};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum StrategyRequestOrigin {
    Api,
    Harness,
    Replay,
    Test,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct StrategyCallerProvenance {
    pub request_origin: StrategyRequestOrigin,
    pub actor_identity: Option<String>,
    pub correlation_id: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NativeStrategyCommitRequest {
    strategy_name: CommitStrategySemanticName,
    input_bytes: Arc<[u8]>,
    caller_provenance: StrategyCallerProvenance,
}

impl NativeStrategyCommitRequest {
    pub fn from_native_canonical_bytes(
        strategy_name: CommitStrategySemanticName,
        input_bytes: impl Into<Arc<[u8]>>,
        caller_provenance: StrategyCallerProvenance,
    ) -> Self {
        Self {
            strategy_name,
            input_bytes: input_bytes.into(),
            caller_provenance,
        }
    }

    pub fn strategy_name(&self) -> &CommitStrategySemanticName {
        &self.strategy_name
    }

    pub fn input_bytes(&self) -> &[u8] {
        &self.input_bytes
    }

    pub fn caller_provenance(&self) -> &StrategyCallerProvenance {
        &self.caller_provenance
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct CanonicalStrategyInputDigest(pub [u8; 32]);

#[derive(Clone, Serialize, Deserialize)]
pub struct CanonicalStrategyInputArtifact {
    schema_name: StrategyInputSchemaName,
    schema_version: StrategyInputSchemaVersion,
    canonical_bytes: Arc<[u8]>,
    digest: CanonicalStrategyInputDigest,
    artifact_name: PersistentArtifactName,
    // Last: original bytes drop before their cold readmission admission.
    #[serde(skip)]
    canonical_bytes_memory: Option<Arc<worth_execution::ExecutionMemoryReservation>>,
}

impl CanonicalStrategyInputArtifact {
    pub(crate) fn new(
        schema_name: StrategyInputSchemaName,
        schema_version: StrategyInputSchemaVersion,
        canonical_bytes: Arc<[u8]>,
        digest: CanonicalStrategyInputDigest,
        artifact_name: PersistentArtifactName,
    ) -> Self {
        Self {
            schema_name,
            schema_version,
            canonical_bytes,
            digest,
            artifact_name,
            canonical_bytes_memory: None,
        }
    }

    pub fn schema_name(&self) -> &StrategyInputSchemaName {
        &self.schema_name
    }

    pub fn schema_version(&self) -> StrategyInputSchemaVersion {
        self.schema_version
    }

    pub fn canonical_bytes(&self) -> &[u8] {
        &self.canonical_bytes
    }

    pub fn digest(&self) -> CanonicalStrategyInputDigest {
        self.digest
    }

    pub fn artifact_name(&self) -> &PersistentArtifactName {
        &self.artifact_name
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CanonicalStrategyCommitRequest {
    strategy_id: CommitStrategyId,
    descriptor_digest: CommitStrategyDescriptorDigest,
    canonical_input: CanonicalStrategyInputArtifact,
    caller_provenance: StrategyCallerProvenance,
}

impl CanonicalStrategyCommitRequest {
    pub(crate) fn new(
        strategy_id: CommitStrategyId,
        descriptor_digest: CommitStrategyDescriptorDigest,
        canonical_input: CanonicalStrategyInputArtifact,
        caller_provenance: StrategyCallerProvenance,
    ) -> Self {
        Self {
            strategy_id,
            descriptor_digest,
            canonical_input,
            caller_provenance,
        }
    }

    pub fn strategy_id(&self) -> CommitStrategyId {
        self.strategy_id
    }

    pub fn descriptor_digest(&self) -> CommitStrategyDescriptorDigest {
        self.descriptor_digest
    }

    pub fn canonical_input(&self) -> &CanonicalStrategyInputArtifact {
        &self.canonical_input
    }

    pub fn caller_provenance(&self) -> &StrategyCallerProvenance {
        &self.caller_provenance
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum StrategyCommitRequestError {
    UnknownStrategyName {
        strategy_name: CommitStrategySemanticName,
    },
}
