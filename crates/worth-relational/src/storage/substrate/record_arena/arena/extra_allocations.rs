//! Physical extra ownership across current and historical edges, never authority.
use super::{RecordArena, RecordKind};
use crate::storage::substrate::{
    HistoricalMetadata, SharedColumn, StorageAllocation, StorageAllocationObservation,
    StorageAllocationVisitor,
};
use std::collections::BTreeSet;

/// Scratch of the existing descriptive allocation walk; not a preallocation quote.
pub(super) struct AllocationUnion<'a> {
    pub(super) visitor: &'a mut dyn StorageAllocationVisitor,
    seen: BTreeSet<u64>,
}
impl<'a> AllocationUnion<'a> {
    pub(super) fn new(visitor: &'a mut dyn StorageAllocationVisitor) -> Self {
        Self {
            visitor,
            seen: BTreeSet::new(),
        }
    }
}
impl StorageAllocationVisitor for AllocationUnion<'_> {
    fn visit(&mut self, allocation: StorageAllocationObservation) -> bool {
        self.seen.insert(allocation.id) && self.visitor.visit(allocation)
    }
}

struct SeenVisitor<'a> {
    seen: &'a mut BTreeSet<u64>,
    visitor: &'a mut dyn StorageAllocationVisitor,
}
impl StorageAllocationVisitor for SeenVisitor<'_> {
    fn visit(&mut self, allocation: StorageAllocationObservation) -> bool {
        self.seen.insert(allocation.id) && self.visitor.visit(allocation)
    }
}

fn previously_retained<K: RecordKind>(
    arena: &RecordArena<K>,
    physical: usize,
    extra: &StorageAllocation<K::Extra>,
    version: Option<crate::identity::data::VersionId>,
) -> bool {
    if arena
        .extra
        .retained_value(physical)
        .is_some_and(|old| old.id() == extra.id())
    {
        return true;
    }
    let (Some(history), Some(version)) = (arena.metadata_history.get(physical), version) else {
        return false;
    };
    let start = history.partition_point(|meta| meta.effective_at() < version);
    let end = history.partition_point(|meta| meta.effective_at() <= version);
    // Several nonconflicting writes in one commit may share a version. Search
    // only that seeked run, never every earlier commit's record history.
    (start..end).any(|index| K::metadata_extra(&history[index]).id() == extra.id())
}

fn prior_slot_extra<K: RecordKind>(
    arena: &RecordArena<K>,
    physical: usize,
) -> Option<StorageAllocation<K::Extra>> {
    if let Some(extra) = arena.extra.retained_value(physical) {
        return Some(extra);
    }
    arena.metadata_history.get(physical).and_then(|history| {
        history
            .len()
            .checked_sub(1)
            .and_then(|index| history.get(index))
            .map(|meta| K::metadata_extra(meta).clone())
    })
}

fn visit_extra<K: RecordKind>(
    extra: &StorageAllocation<K::Extra>,
    previous_extra: Option<&K::Extra>,
    seen: &mut BTreeSet<u64>,
    visitor: &mut dyn StorageAllocationVisitor,
) {
    let mut union = SeenVisitor { seen, visitor };
    if union.visit(extra.observation(false)) {
        K::visit_new_extra_allocations(extra, previous_extra, &mut union);
    }
}

impl<K: RecordKind> RecordArena<K> {
    pub(super) fn visit_new_extra_allocations(
        &self,
        previous: &Self,
        visitor: &mut dyn StorageAllocationVisitor,
    ) {
        let mut seen = BTreeSet::new();
        self.extra
            .visit_changed_values(&previous.extra, &mut |physical, current, _| {
                if current.is_none() {
                    return;
                }
                let extra = self
                    .extra
                    .retained_value(physical)
                    // Proof: the changed value remains in this borrowed column; this shared borrow cannot remove it.
                    .expect("the changed value remains in this borrowed column");
                let version = self
                    .metadata_history
                    .get(physical)
                    .and_then(|history| {
                        history
                            .len()
                            .checked_sub(1)
                            .and_then(|index| history.get(index))
                    })
                    .filter(|meta| K::metadata_extra(meta).id() == extra.id())
                    .map(HistoricalMetadata::effective_at);
                if !previously_retained(previous, physical, &extra, version) {
                    let prior_extra = prior_slot_extra(previous, physical);
                    visit_extra::<K>(&extra, prior_extra.as_deref(), &mut seen, visitor);
                }
            });
        self.metadata_history.visit_changed_values(
            &previous.metadata_history,
            &mut |physical, current, prior| {
                let Some(history) = current else {
                    return;
                };
                let empty = SharedColumn::default();
                let prior_extra = prior_slot_extra(previous, physical);
                history.visit_changed_values(prior.unwrap_or(&empty), &mut |_, meta, _| {
                    let Some(meta) = meta else {
                        return;
                    };
                    let extra = K::metadata_extra(meta);
                    if !previously_retained(previous, physical, extra, Some(meta.effective_at())) {
                        visit_extra::<K>(extra, prior_extra.as_deref(), &mut seen, visitor);
                    }
                });
            },
        );
    }
}
