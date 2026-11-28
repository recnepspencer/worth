mod retained_charge;

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

use crate::data::node::NodeContract;

use super::SignalSchemaBinding;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct SignalSchemaId(pub u32);

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct SignalSchemaName(String);

impl SignalSchemaName {
    pub fn new(name: impl Into<String>) -> Self {
        Self(name.into())
    }

    pub fn as_str(&self) -> &str {
        self.0.as_str()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct SignalSchemaVersion {
    pub major: u16,
    pub minor: u16,
}

impl SignalSchemaVersion {
    pub fn new(major: u16, minor: u16) -> Self {
        Self { major, minor }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SignalSchemaDescriptor {
    id: SignalSchemaId,
    semantic_name: SignalSchemaName,
    version: SignalSchemaVersion,
    default_contract: NodeContract,
    digest: String,
}

impl SignalSchemaDescriptor {
    pub fn new(
        id: SignalSchemaId,
        semantic_name: SignalSchemaName,
        version: SignalSchemaVersion,
        default_contract: NodeContract,
    ) -> Self {
        let digest = schema_descriptor_digest(&id, &semantic_name, &version, &default_contract);
        Self {
            id,
            semantic_name,
            version,
            default_contract,
            digest,
        }
    }

    pub fn id(&self) -> SignalSchemaId {
        self.id
    }

    pub fn semantic_name(&self) -> &SignalSchemaName {
        &self.semantic_name
    }

    pub fn version(&self) -> SignalSchemaVersion {
        self.version
    }

    pub fn default_contract(&self) -> &NodeContract {
        &self.default_contract
    }

    pub fn digest(&self) -> &str {
        &self.digest
    }

    pub fn binding(&self) -> SignalSchemaBinding {
        SignalSchemaBinding::new(
            self.id,
            self.semantic_name.clone(),
            self.version,
            self.digest.clone(),
        )
    }
}

fn schema_descriptor_digest(
    id: &SignalSchemaId,
    semantic_name: &SignalSchemaName,
    version: &SignalSchemaVersion,
    default_contract: &NodeContract,
) -> String {
    let canonical = serde_json::json!({
        "id": id.0,
        "semantic_name": semantic_name.as_str(),
        "version": {
            "major": version.major,
            "minor": version.minor,
        },
        "default_contract": default_contract,
        "default_merge_strategy_name": null,
        "default_conflict_policy_name": null,
        "default_identity_matcher_name": null,
        "default_source_only_policy_name": null,
        "default_deletion_policy_name": null,
        "default_conflict_isolation_policy_name": null,
        "default_aspect_merge_policy_bindings": [],
    });
    let bytes = serde_json::to_vec(&canonical).expect("signal schema descriptor serialization");
    let digest = Sha256::digest(bytes);
    digest.iter().map(|byte| format!("{byte:02x}")).collect()
}
