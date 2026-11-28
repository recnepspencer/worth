//! Native graph-law meaning distinct from physical edge deletion/uniqueness.
use crate::config::data::CrossContextPolicy;
use crate::graph_member::NativeMemberDomain;
use crate::internal_port::graph_member::domain::NativeMemberDomainOwnerPort as _;
use serde::{Deserialize, Serialize};

pub use worth_foundational::RelationCardinality as DerivedRelationCardinality;

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
        self.cardinality.is_valid()
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

    pub(crate) fn clone_allocation_bytes(
        &self,
        checkpoint: &mut dyn FnMut() -> Option<()>,
    ) -> Option<usize> {
        let Some(minimum) = &self.reaches_at_least else {
            return Some(0);
        };
        minimum.target_domains.iter().try_fold(
            minimum
                .target_domains
                .len()
                .checked_mul(size_of::<NativeMemberDomain>())?,
            |bytes, domain| {
                checkpoint()?;
                bytes.checked_add(domain.name_bytes()?)
            },
        )
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
