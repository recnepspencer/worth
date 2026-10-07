use crate::data::core_profile::StableHashValue;
use crate::data::handle::NodeId;
use crate::data::node::ContextRequirement;
use crate::data::output::PartitionSubscription;
use crate::data::proof::PartitionScopeSet;

use super::identity::{PersistentCorrespondenceEvidence, ReuseSemanticRegionIdentity};

fn stable_hash_seed() -> StableHashValue {
    0xcbf29ce484222325_u64 as StableHashValue
}

fn hash_u64(mut hash: StableHashValue, value: u64) -> StableHashValue {
    for byte in value.to_le_bytes() {
        hash ^= byte as StableHashValue;
        hash = hash.wrapping_mul(0x100000001b3_u64 as StableHashValue);
    }
    hash
}

fn hash_bool(hash: StableHashValue, value: bool) -> StableHashValue {
    hash_u64(hash, u64::from(value))
}

fn hash_str(mut hash: StableHashValue, value: &str) -> StableHashValue {
    for byte in value.as_bytes() {
        hash ^= *byte as StableHashValue;
        hash = hash.wrapping_mul(0x100000001b3_u64 as StableHashValue);
    }
    hash
}

pub(super) fn stable_semantic_region_digest(
    region: &ReuseSemanticRegionIdentity,
) -> StableHashValue {
    stable_semantic_region_digest_from_parts(
        region.node,
        region.partitioned_output,
        &region.partition_scope,
        region.required_context,
    )
}

pub(crate) fn stable_semantic_region_digest_from_parts(
    node: NodeId,
    partitioned_output: bool,
    partition_scope: &[PartitionSubscription],
    required_context: ContextRequirement,
) -> StableHashValue {
    let mut hash = stable_hash_seed();
    hash = hash_u64(hash, node.index() as u64);
    hash = hash_u64(hash, node.generation() as u64);
    hash = hash_bool(hash, partitioned_output);
    for scope in partition_scope {
        hash = hash_partition_subscription(hash, scope);
    }
    hash_context_requirement(hash, required_context)
}

pub(super) fn stable_partition_scope_digest(scopes: &PartitionScopeSet) -> StableHashValue {
    stable_partition_scope_digest_from_slice(scopes.as_slice())
}

pub(crate) fn stable_partition_scope_digest_from_slice(
    scopes: &[PartitionSubscription],
) -> StableHashValue {
    scopes.iter().fold(stable_hash_seed(), |hash, scope| {
        hash_partition_subscription(hash, scope)
    })
}

pub(crate) fn stable_persistent_correspondence_digest(
    evidence: &PersistentCorrespondenceEvidence,
) -> StableHashValue {
    let (tag, value) = match evidence {
        PersistentCorrespondenceEvidence::HostSuppliedKey(value) => (1_u64, value.as_str()),
        PersistentCorrespondenceEvidence::ContractDeclaredBasis(value) => (2_u64, value.as_str()),
        PersistentCorrespondenceEvidence::LineageBackedMapping(value) => (3_u64, value.as_str()),
        PersistentCorrespondenceEvidence::RegionIdentityBasis(value) => (4_u64, value.as_str()),
    };
    hash_str(hash_u64(stable_hash_seed(), tag), value)
}

fn hash_partition_subscription(
    mut hash: StableHashValue,
    scope: &PartitionSubscription,
) -> StableHashValue {
    hash = hash_u64(hash, scope.path().depth() as u64);
    for segment in scope.path().segments() {
        hash = hash_u64(hash, segment.len() as u64);
        hash = hash_str(hash, segment);
    }
    hash_u64(hash, scope.coverage() as u64)
}

fn hash_context_requirement(
    hash: StableHashValue,
    requirement: ContextRequirement,
) -> StableHashValue {
    let tag = match requirement {
        ContextRequirement::None => 0_u64,
        ContextRequirement::DomainContext => 1_u64,
        ContextRequirement::RelationalSnapshot => 2_u64,
    };
    hash_u64(hash, tag)
}
