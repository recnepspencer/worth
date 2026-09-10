mod retained_charge;

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

use crate::data::node::NodeContract;
use crate::logic::transaction::{
    AspectMergePolicyBinding, ConflictIsolationPolicyName, ConflictPolicyName, DeletionPolicyName,
    IdentityMatcherName, MergeStrategyName, SourceOnlyPolicyName,
};

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
    default_merge_strategy_name: Option<MergeStrategyName>,
    default_conflict_policy_name: Option<ConflictPolicyName>,
    default_identity_matcher_name: Option<IdentityMatcherName>,
    default_source_only_policy_name: Option<SourceOnlyPolicyName>,
    default_deletion_policy_name: Option<DeletionPolicyName>,
    default_conflict_isolation_policy_name: Option<ConflictIsolationPolicyName>,
    default_aspect_merge_policy_bindings: Vec<AspectMergePolicyBinding>,
    digest: String,
}

impl SignalSchemaDescriptor {
    pub fn new(
        id: SignalSchemaId,
        semantic_name: SignalSchemaName,
        version: SignalSchemaVersion,
        default_contract: NodeContract,
    ) -> Self {
        Self::new_with_merge_semantics(
            id,
            semantic_name,
            version,
            default_contract,
            None,
            None,
            None,
            None,
            None,
        )
    }

    pub fn new_with_merge_strategy(
        id: SignalSchemaId,
        semantic_name: SignalSchemaName,
        version: SignalSchemaVersion,
        default_contract: NodeContract,
        default_merge_strategy_name: Option<MergeStrategyName>,
    ) -> Self {
        Self::new_with_merge_semantics(
            id,
            semantic_name,
            version,
            default_contract,
            default_merge_strategy_name,
            None,
            None,
            None,
            None,
        )
    }

    pub fn new_with_merge_semantics(
        id: SignalSchemaId,
        semantic_name: SignalSchemaName,
        version: SignalSchemaVersion,
        default_contract: NodeContract,
        default_merge_strategy_name: Option<MergeStrategyName>,
        default_conflict_policy_name: Option<ConflictPolicyName>,
        default_identity_matcher_name: Option<IdentityMatcherName>,
        default_source_only_policy_name: Option<SourceOnlyPolicyName>,
        default_deletion_policy_name: Option<DeletionPolicyName>,
    ) -> Self {
        Self::new_with_merge_semantics_and_isolation(
            id,
            semantic_name,
            version,
            default_contract,
            default_merge_strategy_name,
            default_conflict_policy_name,
            default_identity_matcher_name,
            default_source_only_policy_name,
            default_deletion_policy_name,
            None,
        )
    }

    pub fn new_with_merge_semantics_and_isolation(
        id: SignalSchemaId,
        semantic_name: SignalSchemaName,
        version: SignalSchemaVersion,
        default_contract: NodeContract,
        default_merge_strategy_name: Option<MergeStrategyName>,
        default_conflict_policy_name: Option<ConflictPolicyName>,
        default_identity_matcher_name: Option<IdentityMatcherName>,
        default_source_only_policy_name: Option<SourceOnlyPolicyName>,
        default_deletion_policy_name: Option<DeletionPolicyName>,
        default_conflict_isolation_policy_name: Option<ConflictIsolationPolicyName>,
    ) -> Self {
        Self::new_with_merge_semantics_and_aspects_and_isolation(
            id,
            semantic_name,
            version,
            default_contract,
            default_merge_strategy_name,
            default_conflict_policy_name,
            default_identity_matcher_name,
            default_source_only_policy_name,
            default_deletion_policy_name,
            default_conflict_isolation_policy_name,
            Vec::new(),
        )
    }

    pub fn new_with_merge_semantics_and_aspects(
        id: SignalSchemaId,
        semantic_name: SignalSchemaName,
        version: SignalSchemaVersion,
        default_contract: NodeContract,
        default_merge_strategy_name: Option<MergeStrategyName>,
        default_conflict_policy_name: Option<ConflictPolicyName>,
        default_identity_matcher_name: Option<IdentityMatcherName>,
        default_source_only_policy_name: Option<SourceOnlyPolicyName>,
        default_deletion_policy_name: Option<DeletionPolicyName>,
        default_aspect_merge_policy_bindings: Vec<AspectMergePolicyBinding>,
    ) -> Self {
        Self::new_with_merge_semantics_and_aspects_and_isolation(
            id,
            semantic_name,
            version,
            default_contract,
            default_merge_strategy_name,
            default_conflict_policy_name,
            default_identity_matcher_name,
            default_source_only_policy_name,
            default_deletion_policy_name,
            None,
            default_aspect_merge_policy_bindings,
        )
    }

    pub fn new_with_merge_semantics_and_aspects_and_isolation(
        id: SignalSchemaId,
        semantic_name: SignalSchemaName,
        version: SignalSchemaVersion,
        default_contract: NodeContract,
        default_merge_strategy_name: Option<MergeStrategyName>,
        default_conflict_policy_name: Option<ConflictPolicyName>,
        default_identity_matcher_name: Option<IdentityMatcherName>,
        default_source_only_policy_name: Option<SourceOnlyPolicyName>,
        default_deletion_policy_name: Option<DeletionPolicyName>,
        default_conflict_isolation_policy_name: Option<ConflictIsolationPolicyName>,
        mut default_aspect_merge_policy_bindings: Vec<AspectMergePolicyBinding>,
    ) -> Self {
        default_aspect_merge_policy_bindings.sort_by_key(|binding| binding.aspect.id());
        let digest = schema_descriptor_digest(
            &id,
            &semantic_name,
            &version,
            &default_contract,
            default_merge_strategy_name.as_ref(),
            default_conflict_policy_name.as_ref(),
            default_identity_matcher_name.as_ref(),
            default_source_only_policy_name.as_ref(),
            default_deletion_policy_name.as_ref(),
            default_conflict_isolation_policy_name.as_ref(),
            &default_aspect_merge_policy_bindings,
        );
        Self {
            id,
            semantic_name,
            version,
            default_contract,
            default_merge_strategy_name,
            default_conflict_policy_name,
            default_identity_matcher_name,
            default_source_only_policy_name,
            default_deletion_policy_name,
            default_conflict_isolation_policy_name,
            default_aspect_merge_policy_bindings,
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

    pub fn default_merge_strategy_name(&self) -> Option<&MergeStrategyName> {
        self.default_merge_strategy_name.as_ref()
    }

    pub fn default_conflict_policy_name(&self) -> Option<&ConflictPolicyName> {
        self.default_conflict_policy_name.as_ref()
    }

    pub fn default_identity_matcher_name(&self) -> Option<&IdentityMatcherName> {
        self.default_identity_matcher_name.as_ref()
    }

    pub fn default_source_only_policy_name(&self) -> Option<&SourceOnlyPolicyName> {
        self.default_source_only_policy_name.as_ref()
    }

    pub fn default_deletion_policy_name(&self) -> Option<&DeletionPolicyName> {
        self.default_deletion_policy_name.as_ref()
    }

    pub fn default_conflict_isolation_policy_name(&self) -> Option<&ConflictIsolationPolicyName> {
        self.default_conflict_isolation_policy_name.as_ref()
    }

    pub fn default_aspect_merge_policy_bindings(&self) -> &[AspectMergePolicyBinding] {
        &self.default_aspect_merge_policy_bindings
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
    default_merge_strategy_name: Option<&MergeStrategyName>,
    default_conflict_policy_name: Option<&ConflictPolicyName>,
    default_identity_matcher_name: Option<&IdentityMatcherName>,
    default_source_only_policy_name: Option<&SourceOnlyPolicyName>,
    default_deletion_policy_name: Option<&DeletionPolicyName>,
    default_conflict_isolation_policy_name: Option<&ConflictIsolationPolicyName>,
    default_aspect_merge_policy_bindings: &[AspectMergePolicyBinding],
) -> String {
    let canonical = serde_json::json!({
        "id": id.0,
        "semantic_name": semantic_name.as_str(),
        "version": {
            "major": version.major,
            "minor": version.minor,
        },
        "default_contract": default_contract,
        "default_merge_strategy_name": default_merge_strategy_name.map(MergeStrategyName::as_str),
        "default_conflict_policy_name": default_conflict_policy_name.map(ConflictPolicyName::as_str),
        "default_identity_matcher_name": default_identity_matcher_name.map(IdentityMatcherName::as_str),
        "default_source_only_policy_name": default_source_only_policy_name.map(SourceOnlyPolicyName::as_str),
        "default_deletion_policy_name": default_deletion_policy_name.map(DeletionPolicyName::as_str),
        "default_conflict_isolation_policy_name": default_conflict_isolation_policy_name.map(ConflictIsolationPolicyName::as_str),
        "default_aspect_merge_policy_bindings": default_aspect_merge_policy_bindings.iter().map(|binding| serde_json::json!({
            "aspect": binding.aspect.id(),
            "policy_name": binding.policy_name.as_str(),
        })).collect::<Vec<_>>(),
    });
    let bytes = serde_json::to_vec(&canonical).expect("signal schema descriptor serialization");
    let digest = Sha256::digest(bytes);
    digest.iter().map(|byte| format!("{byte:02x}")).collect()
}
