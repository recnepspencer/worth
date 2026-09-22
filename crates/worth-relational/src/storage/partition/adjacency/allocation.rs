use super::{AdjacencySet, RelationSet};
use crate::storage::substrate::{visit_inline_value, StorageAllocationVisitor};

impl AdjacencySet {
    pub(crate) fn visit_new_cache_allocations(
        &self,
        previous: &Self,
        visitor: &mut dyn StorageAllocationVisitor,
    ) {
        let entries = self.entries();
        let previous = previous.entries();
        let empty = Default::default();
        for (kinds, old) in [
            (&entries.current_by_kind, &previous.current_by_kind),
            (&entries.historical_by_kind, &previous.historical_by_kind),
        ] {
            if let Some(kinds) = kinds {
                kinds.visit_new_allocations(
                    old.as_ref().unwrap_or(&empty),
                    visitor,
                    &mut |relations: &RelationSet, previous, allocation, visitor| {
                        if visitor.visit(allocation) {
                            if let Some(previous) = previous {
                                relations.visit_new_allocations(
                                    previous,
                                    visitor,
                                    &mut crate::storage::substrate::visit_new_inline_value,
                                );
                            } else {
                                relations.visit_allocations(
                                    false,
                                    visitor,
                                    &mut visit_inline_value,
                                );
                            }
                        }
                    },
                );
            }
        }
        if let Some(revisions) = &entries.structural_revision_by_kind {
            revisions.visit_new_allocations(
                previous
                    .structural_revision_by_kind
                    .as_ref()
                    .unwrap_or(&Default::default()),
                visitor,
                &mut crate::storage::substrate::visit_new_inline_value,
            );
        }
    }

    pub(crate) fn visit_new_authoritative_allocations(
        &self,
        previous: &Self,
        visitor: &mut dyn StorageAllocationVisitor,
    ) {
        self.entries().current.visit_new_allocations(
            &previous.entries().current,
            visitor,
            &mut crate::storage::substrate::visit_new_inline_value,
        );
    }
    pub(crate) fn visit_authoritative_allocations(
        &self,
        unique: bool,
        visitor: &mut dyn StorageAllocationVisitor,
    ) {
        self.entries()
            .current
            .visit_allocations(unique, visitor, &mut visit_inline_value);
    }

    pub(crate) fn visit_cache_allocations(&self, visitor: &mut dyn StorageAllocationVisitor) {
        let entries = self.entries();
        for kinds in [&entries.current_by_kind, &entries.historical_by_kind]
            .into_iter()
            .flatten()
        {
            kinds.visit_allocations(
                false,
                visitor,
                &mut |relations: &RelationSet, allocation, visitor| {
                    if visitor.visit(allocation) {
                        relations.visit_allocations(false, visitor, &mut visit_inline_value);
                    }
                },
            );
        }
        if let Some(revisions) = &entries.structural_revision_by_kind {
            revisions.visit_allocations(false, visitor, &mut visit_inline_value);
        }
    }
}
