//! Native graph-law meaning distinct from physical edge deletion/uniqueness.
use crate::config::data::CrossContextPolicy;
use crate::graph_member::NativeMemberDomain;
use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq, Ord, PartialOrd, Serialize, Deserialize)]
pub struct DerivedRelationCardinality {
    pub source_min: Option<u64>,
    pub source_max: Option<u64>,
    pub target_min: Option<u64>,
    pub target_max: Option<u64>,
    pub pair_min: Option<u64>,
    pub pair_max: Option<u64>,
}
impl DerivedRelationCardinality {
    pub(super) fn valid(&self) -> bool {
        [self.source_min, self.target_min, self.pair_min]
            .into_iter()
            .flatten()
            .all(|n| n > 0)
            && self.pair_max != Some(0)
            && [
                (self.source_min, self.source_max),
                (self.target_min, self.target_max),
                (self.pair_min, self.pair_max),
            ]
            .into_iter()
            .all(|(minimum, maximum)| match (minimum, maximum) {
                (Some(a), Some(b)) => a <= b,
                _ => true,
            })
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Serialize, Deserialize)]
pub struct DerivedRelationMinimumReachability {
    pub minimum_reachable_targets: u32,
    pub target_domains: Vec<NativeMemberDomain>,
}

/// Counts endpoint pairs, never supporting physical path multiplicity.
#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Serialize, Deserialize)]
pub struct DerivedRelationGraphLaws {
    pub self_edges_allowed: bool,
    pub cross_context_policy: CrossContextPolicy,
    pub cardinality: DerivedRelationCardinality,
    pub acyclic: bool,
    pub symmetric: bool,
    pub orbit: bool,
    pub same_partition_endpoints: bool,
    pub reaches_at_least: Option<DerivedRelationMinimumReachability>,
}
impl DerivedRelationGraphLaws {
    pub(crate) fn valid(&self, from: &NativeMemberDomain, to: &NativeMemberDomain) -> bool {
        self.cardinality.valid()
            && (!self.orbit || from == to)
            && self.reaches_at_least.as_ref().is_none_or(|minimum| {
                minimum.minimum_reachable_targets > 0
                    && !minimum.target_domains.is_empty()
                    && minimum
                        .target_domains
                        .windows(2)
                        .all(|pair| pair[0] < pair[1])
            })
    }
    pub(crate) fn owned_bytes(&self) -> usize {
        self.reaches_at_least.as_ref().map_or(0, |minimum| {
            minimum
                .target_domains
                .capacity()
                .saturating_mul(size_of::<NativeMemberDomain>())
                .saturating_add(
                    minimum
                        .target_domains
                        .iter()
                        .map(NativeMemberDomain::owned_bytes)
                        .sum::<usize>(),
                )
        })
    }
}
