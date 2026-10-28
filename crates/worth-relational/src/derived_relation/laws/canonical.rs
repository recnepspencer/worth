//! Optional law suffix binds every executable constraint without old-byte drift.
use super::DerivedRelationGraphLaws;
use crate::config::data::CrossContextPolicy;
use sha2::{Digest, Sha256};

pub(crate) fn update_digest(hasher: &mut Sha256, laws: &DerivedRelationGraphLaws) {
    hasher.update(b"\0WORTH_DERIVED_RELATION_LAWS_V1\0");
    hasher.update([
        u8::from(laws.self_edges_allowed),
        u8::from(laws.acyclic),
        u8::from(laws.symmetric),
        u8::from(laws.orbit),
        u8::from(laws.same_partition_endpoints),
    ]);
    hasher.update([match laws.cross_context_policy {
        CrossContextPolicy::AllowExplicit => 1,
        CrossContextPolicy::SchemaControlled => 2,
        CrossContextPolicy::Forbid => 3,
    }]);
    let cardinality = laws.cardinality;
    for value in [
        cardinality.source_min,
        cardinality.source_max,
        cardinality.target_min,
        cardinality.target_max,
        cardinality.pair_min,
        cardinality.pair_max,
    ] {
        match value {
            None => hasher.update([0]),
            Some(value) => {
                hasher.update([1]);
                hasher.update(value.to_le_bytes());
            }
        }
    }
    match &laws.reaches_at_least {
        None => hasher.update([0]),
        Some(minimum) => {
            hasher.update([1]);
            hasher.update(minimum.minimum_reachable_targets.to_le_bytes());
            hasher.update((minimum.target_domains.len() as u64).to_le_bytes());
            for domain in &minimum.target_domains {
                crate::derived_relation::canonical::endpoint(hasher, domain);
            }
        }
    }
}
