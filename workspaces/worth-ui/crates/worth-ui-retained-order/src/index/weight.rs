use std::hash::Hash;

use super::BoundedOrderIndex;

impl<Identity> BoundedOrderIndex<Identity>
where
    Identity: Copy + Eq + Hash,
{
    pub(crate) fn weight(&self, identity: Identity) -> Option<u32> {
        self.cost.identity_lookup();
        self.identities
            .get(&identity)
            .map(|node| self.nodes[*node].weight)
    }

    pub(crate) fn update_weight(&mut self, identity: Identity, weight: u32) -> bool {
        self.cost.identity_lookup();
        let Some(mut node) = self.identities.get(&identity).copied() else {
            return false;
        };
        self.nodes[node].weight = weight;
        loop {
            self.cost.node_touch();
            self.refresh(node);
            let Some(parent) = self.nodes[node].parent else {
                break;
            };
            node = parent;
        }
        true
    }

    pub(crate) fn first_with_weight_at_least(&self, minimum: u32) -> Option<Identity> {
        self.cost.identity_lookup();
        let mut current = self.root?;
        loop {
            self.cost.node_touch();
            if self.nodes[current]
                .left
                .is_some_and(|left| self.nodes[left].subtree_max_weight >= minimum)
            {
                current = self.nodes[current].left.expect("tested left child");
                continue;
            }
            if self.nodes[current].weight >= minimum {
                return self.nodes[current].identity;
            }
            if self.nodes[current]
                .right
                .is_none_or(|right| self.nodes[right].subtree_max_weight < minimum)
            {
                return None;
            }
            current = self.nodes[current].right?;
        }
    }
}
