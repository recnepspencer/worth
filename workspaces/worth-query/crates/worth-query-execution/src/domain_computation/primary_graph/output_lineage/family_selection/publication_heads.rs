//! Native publication order across a family's distinct mutation bindings.
use super::super::RecordedOutput;
use std::{cmp::Reverse, collections::BTreeMap};
use worth_relational::facade::identity::EntityId;

type Member<'a> = (Option<[u8; 32]>, &'a str, EntityId);
struct Head<'a> {
    position: (Reverse<usize>, u64),
    recorded: &'a RecordedOutput,
    ambiguous: bool,
}

#[derive(Default)]
pub(super) struct PublicationHeads<'a> {
    members: BTreeMap<Member<'a>, Head<'a>>,
    unbound: Vec<(&'a str, &'a RecordedOutput)>,
}

impl<'a> PublicationHeads<'a> {
    /// One role lookup plus the finite balanced-tree comparison bound.
    /// Storage and selection scale with offered heads, never older history.
    pub(super) fn admission_work(&self) -> usize {
        let levels = usize::BITS as usize - self.members.len().max(1).leading_zeros() as usize;
        1 + 11 * levels
    }

    pub(super) fn insert(
        &mut self,
        partition: Option<[u8; 32]>,
        role: &'a str,
        depth: usize,
        recorded: &'a RecordedOutput,
    ) {
        let Some(entity) = recorded.correspondence.publication_entity_for_role(role) else {
            self.unbound.push((role, recorded));
            return;
        };
        let position = (Reverse(depth), recorded.settlement_identity.address().1);
        match self.members.entry((partition, role, entity)) {
            std::collections::btree_map::Entry::Vacant(entry) => {
                entry.insert(Head {
                    position,
                    recorded,
                    ambiguous: false,
                });
            }
            std::collections::btree_map::Entry::Occupied(mut entry) => {
                let head = entry.get_mut();
                match position.cmp(&head.position) {
                    std::cmp::Ordering::Greater => {
                        *head = Head {
                            position,
                            recorded,
                            ambiguous: false,
                        }
                    }
                    std::cmp::Ordering::Equal => {
                        head.ambiguous |=
                            recorded.settlement_identity != head.recorded.settlement_identity;
                    }
                    std::cmp::Ordering::Less => {}
                }
            }
        }
    }

    pub(super) fn finish(self) -> (Vec<(&'a str, &'a RecordedOutput)>, bool) {
        let mut heads = self.unbound;
        let mut ambiguous = false;
        for ((_, role, _), head) in self.members {
            ambiguous |= head.ambiguous;
            if !head.ambiguous {
                heads.push((role, head.recorded));
            }
        }
        (heads, ambiguous)
    }
}
