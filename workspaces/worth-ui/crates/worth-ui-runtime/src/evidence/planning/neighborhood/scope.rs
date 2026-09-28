use std::sync::Arc;

use crate::declaration::stable_text_digest;
use crate::evidence::{UiAllocationNeighborhood, UiAllocationNeighborhoodClass};
use crate::graph::UiGraphNodeIdentity;

/// Stable admitted neighborhood ownership, deliberately excluding generations.
///
/// Generations describe a particular measurement/planning attempt; they must
/// never select a different committed-receipt slot.
#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct UiAllocationNeighborhoodScope {
    root_graph_node_identity: UiGraphNodeIdentity,
    layout_operator_contract_identity_digest: u64,
    neighborhood_class: UiAllocationNeighborhoodClass,
    /// Shared, so a scope keying several indexes clones without allocating.
    member_identity_digests: Arc<[u64]>,
}

impl UiAllocationNeighborhoodScope {
    pub fn from_neighborhood(neighborhood: &UiAllocationNeighborhood) -> Self {
        let identity = neighborhood.identity();
        // Membership ownership must survive a measurement-basis rebind.  The
        // admitted graph-node/role pair is stable; measurement-derived member
        // digests and dependency-map digests are not.
        let mut member_identity_digests = neighborhood
            .members()
            .iter()
            .map(|member| {
                stable_text_digest("allocation-neighborhood-scope-member")
                    ^ member.graph_node_identity().digest().rotate_left(7)
                    ^ (member.role() as u64).rotate_left(13)
            })
            .collect::<Vec<_>>();
        member_identity_digests.sort_unstable();
        Self {
            root_graph_node_identity: identity.root_graph_node_identity(),
            layout_operator_contract_identity_digest: identity
                .layout_operator_contract_identity_digest(),
            neighborhood_class: identity.neighborhood_class(),
            member_identity_digests: member_identity_digests.into(),
        }
    }

    pub fn root_graph_node_identity(&self) -> UiGraphNodeIdentity {
        self.root_graph_node_identity
    }

    pub fn neighborhood_class(&self) -> UiAllocationNeighborhoodClass {
        self.neighborhood_class
    }

    pub fn member_identity_digests(&self) -> &[u64] {
        &self.member_identity_digests
    }
    pub(crate) fn identity_digest(&self) -> u64 {
        self.member_identity_digests.iter().fold(
            stable_text_digest("allocation-neighborhood-scope")
                ^ self.root_graph_node_identity.digest().rotate_left(7)
                ^ self
                    .layout_operator_contract_identity_digest
                    .rotate_left(17)
                ^ (self.neighborhood_class as u64).rotate_left(29),
            |digest, member| digest.rotate_left(5) ^ member,
        )
    }
}
