//! Real Phase 3 group naming world: every step runs
//! `AggregateGroupNamingAuthority::transition` over a proposed partition of
//! member names and records the committed transition.

use std::collections::BTreeSet;

use worth_foundational::facade::AggregateIdentity;

use crate::aggregate_identity::{
    AggregateContinuityRule, AggregateGroupMembers, AggregateGroupNamingAuthority,
    AggregateGroupNamingDenial, AggregateGroupScheme, AggregateGroupSchemeId,
    AggregateGroupTransition, AggregateMemberRole, AggregatePriorGroup,
};
use crate::graph_member::NativeMemberKey;
use crate::identity::data::{EntityId, PartitionId};
use crate::lineage::data::PersistentNameIdentity;

/// The synthetic persistent name of group member `n`.
pub(super) fn member(n: u8) -> PersistentNameIdentity {
    let mut bytes = [0x6b; 32];
    bytes[0] = n;
    PersistentNameIdentity::from_minted_digest(bytes)
}

/// Live and retired group names of one scheme across committed transitions.
pub(super) struct GroupWorld {
    scheme: AggregateGroupScheme,
    prior: Vec<AggregatePriorGroup>,
    retired: BTreeSet<AggregateIdentity>,
}

impl GroupWorld {
    pub(super) fn new() -> Self {
        let id = AggregateGroupSchemeId::new("cad.loop.v1").expect("valid scheme id");
        Self {
            scheme: AggregateGroupScheme::new(id, AggregateContinuityRule::Anchor),
            prior: Vec::new(),
            retired: BTreeSet::new(),
        }
    }

    /// Names the next complete partition. Each inner slice is one group's
    /// members; the least member anchors a fresh name.
    pub(super) fn step(
        &mut self,
        groups: &[&[u8]],
    ) -> Result<AggregateGroupTransition, AggregateGroupNamingDenial> {
        let mut proposed = Vec::with_capacity(groups.len());
        for group in groups {
            let mut members = AggregateGroupMembers::new();
            for &n in *group {
                let carrier =
                    NativeMemberKey::Entity(EntityId::new(PartitionId::main(), u64::from(n), 1));
                members.insert(member(n), carrier, AggregateMemberRole::Ordinary)?;
            }
            proposed.push(members);
        }
        let transition = AggregateGroupNamingAuthority::transition(
            &self.scheme,
            &self.prior,
            &self.retired,
            proposed,
        )?;
        for assignment in &transition.assignments {
            self.retired.remove(&assignment.identity);
        }
        self.retired.extend(transition.retired.iter().copied());
        self.prior = transition
            .assignments
            .iter()
            .map(|assignment| AggregatePriorGroup {
                identity: assignment.identity,
                anchor: assignment.anchor,
                sources: assignment.sources.clone(),
                members: assignment.members.clone(),
            })
            .collect();
        Ok(transition)
    }

    /// The live group whose members include `n`.
    pub(super) fn group_of(&self, n: u8) -> AggregateIdentity {
        self.prior
            .iter()
            .find(|group| group.members.get(&member(n)).is_some())
            .map(|group| group.identity)
            .expect("member belongs to a live group")
    }

    pub(super) fn live_groups(&self) -> Vec<AggregateIdentity> {
        self.prior.iter().map(|group| group.identity).collect()
    }
}
