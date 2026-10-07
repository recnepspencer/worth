//! Native publication order across a family's distinct mutation bindings.
use super::super::{partition_index::FamilyPublicationHead, RecordedOutput};
use std::{any::TypeId, cmp::Reverse, collections::BTreeMap};
use worth_relational::facade::identity::EntityId;

type Member<'a> = (Option<[u8; 32]>, &'a str, EntityId);
struct Head<'a> {
    position: (Reverse<usize>, u64),
    binding: TypeId,
    publication: FamilyPublicationHead<'a>,
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
        binding: TypeId,
        depth: usize,
        publication: FamilyPublicationHead<'a>,
    ) {
        let Some(entity) = publication
            .recorded
            .correspondence
            .publication_entity_for_role(role)
        else {
            self.unbound.push((role, publication.recorded));
            return;
        };
        // A stable republication can inherit an older performed settlement.
        // Selection follows the publication's actual retained index position.
        let position = (Reverse(depth), publication.coordinate.generation);
        match self.members.entry((partition, role, entity)) {
            std::collections::btree_map::Entry::Vacant(entry) => {
                entry.insert(Head {
                    position,
                    binding,
                    publication,
                    ambiguous: false,
                });
            }
            std::collections::btree_map::Entry::Occupied(mut entry) => {
                let head = entry.get_mut();
                match position.cmp(&head.position) {
                    std::cmp::Ordering::Greater => {
                        *head = Head {
                            position,
                            binding,
                            publication,
                            ambiguous: false,
                        }
                    }
                    std::cmp::Ordering::Equal => {
                        head.ambiguous |= binding != head.binding
                            || publication.coordinate.occurrence
                                != head.publication.coordinate.occurrence
                            || publication.slot != head.publication.slot;
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
                heads.push((role, head.publication.recorded));
            }
        }
        (heads, ambiguous)
    }
}
