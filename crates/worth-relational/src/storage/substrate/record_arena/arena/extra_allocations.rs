//! Physical extra ownership across current and historical edges, never authority.
//! Delta membership proves exact set difference `A(self) \ A(previous)` across
//! changed slots, unchanged slots, historical metadata, and sibling arenas.
use super::{RecordArena, RecordKind};
use crate::storage::substrate::{
    HistoricalMetadata, SharedColumn, StorageAllocation, StorageAllocationObservation,
    StorageAllocationVisitor,
};
use std::collections::{BTreeMap, BTreeSet};

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

/// Exact set-difference collector for Extra carriers and nested persistent
/// aspect allocations across one or both arenas of a partition.
#[derive(Default)]
pub(crate) struct ExtraDeltaCollector {
    candidates: BTreeMap<u64, StorageAllocationObservation>,
    unresolved: BTreeSet<u64>,
    in_previous: BTreeSet<u64>,
}

impl ExtraDeltaCollector {
    pub(crate) fn has_unresolved(&self) -> bool {
        !self.unresolved.is_empty()
    }

    pub(crate) fn emit(self, visitor: &mut dyn StorageAllocationVisitor) {
        for (id, observation) in self.candidates {
            if !self.in_previous.contains(&id) {
                visitor.visit(observation);
            }
        }
    }

    fn record(&mut self, allocation: StorageAllocationObservation) -> bool {
        let is_new = match self.candidates.entry(allocation.id) {
            std::collections::btree_map::Entry::Vacant(entry) => {
                entry.insert(allocation);
                true
            }
            std::collections::btree_map::Entry::Occupied(_) => false,
        };
        if is_new && !allocation.unique {
            self.unresolved.insert(allocation.id);
        }
        is_new
    }
}

struct CandidateVisitor<'a>(&'a mut ExtraDeltaCollector);
impl StorageAllocationVisitor for CandidateVisitor<'_> {
    fn visit(&mut self, allocation: StorageAllocationObservation) -> bool {
        self.0.record(allocation)
    }
}

struct MembershipResolver<'a> {
    seen: &'a mut BTreeSet<u64>,
    collector: &'a mut ExtraDeltaCollector,
}
impl StorageAllocationVisitor for MembershipResolver<'_> {
    fn visit(&mut self, allocation: StorageAllocationObservation) -> bool {
        if self.collector.unresolved.is_empty() {
            return false;
        }
        if !self.seen.insert(allocation.id) {
            return false;
        }
        if self.collector.unresolved.remove(&allocation.id) {
            self.collector.in_previous.insert(allocation.id);
        }
        !self.collector.unresolved.is_empty()
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

struct ChangedCarrier<K: RecordKind> {
    extra: StorageAllocation<K::Extra>,
    prior_hint: Option<StorageAllocation<K::Extra>>,
    changed_refs: usize,
}

#[cfg(test)]
thread_local! {
    static EXTRA_DELTA_RESOLVE_SCANS: std::cell::Cell<u64> = const { std::cell::Cell::new(0) };
}

#[cfg(test)]
pub(crate) fn extra_delta_resolve_scans_on_current_thread() -> u64 {
    EXTRA_DELTA_RESOLVE_SCANS.with(std::cell::Cell::get)
}

impl<K: RecordKind> RecordArena<K> {
    pub(crate) fn collect_changed_extra_allocations(
        &self,
        previous: &Self,
        previous_watermark: Option<u64>,
        collector: &mut ExtraDeltaCollector,
    ) {
        let mut carriers: BTreeMap<u64, ChangedCarrier<K>> = BTreeMap::new();
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
                    let id = extra.id();
                    let entry = carriers.entry(id).or_insert_with(|| ChangedCarrier {
                        prior_hint: prior_slot_extra(previous, physical),
                        extra,
                        changed_refs: 0,
                    });
                    entry.changed_refs += 1;
                }
            });
        self.metadata_history.visit_changed_values(
            &previous.metadata_history,
            &mut |physical, current, prior| {
                let Some(history) = current else {
                    return;
                };
                let empty = SharedColumn::default();
                history.visit_changed_values(prior.unwrap_or(&empty), &mut |_, meta, _| {
                    let Some(meta) = meta else {
                        return;
                    };
                    let extra = K::metadata_extra(meta);
                    if !previously_retained(previous, physical, extra, Some(meta.effective_at())) {
                        let id = extra.id();
                        let entry = carriers.entry(id).or_insert_with(|| ChangedCarrier {
                            prior_hint: prior_slot_extra(previous, physical),
                            extra: extra.clone(),
                            changed_refs: 0,
                        });
                        entry.changed_refs += 1;
                    }
                });
            },
        );
        for carrier in carriers.into_values() {
            // An Extra born strictly after `previous` was sealed cannot exist
            // anywhere in `previous`, even while a borrowed transaction overlay
            // or mutation log retains additional outer handles to it.
            let born_after_previous = previous_watermark
                .is_some_and(|watermark| K::extra_birth_seq(&carrier.extra) >= watermark);
            let all_in_self =
                born_after_previous || carrier.extra.strong_count() == carrier.changed_refs + 1;
            let mut carrier_obs = carrier.extra.observation(all_in_self);
            carrier_obs.unique = all_in_self;
            let mut candidate_visitor = CandidateVisitor(collector);
            if candidate_visitor.visit(carrier_obs) {
                K::visit_new_extra_allocations(
                    &carrier.extra,
                    carrier.prior_hint.as_deref(),
                    all_in_self,
                    &mut candidate_visitor,
                );
            }
        }
    }

    pub(crate) fn resolve_extra_allocations(
        &self,
        seen: &mut BTreeSet<u64>,
        collector: &mut ExtraDeltaCollector,
    ) {
        if !collector.has_unresolved() {
            return;
        }
        #[cfg(test)]
        EXTRA_DELTA_RESOLVE_SCANS.with(|count| count.set(count.get() + 1));
        let mut resolver = MembershipResolver { seen, collector };
        self.extra.visit_allocations(
            false,
            &mut IgnoreColumns,
            &mut |extra, allocation, _| {
                if !resolver.collector.has_unresolved() {
                    return;
                }
                if allocation.visit(&mut resolver) {
                    K::visit_extra_allocations(extra, false, &mut resolver);
                }
            },
        );
        if !resolver.collector.has_unresolved() {
            return;
        }
        self.metadata_history.visit_allocations(
            false,
            &mut IgnoreColumns,
            &mut |history, _, _| {
                if !resolver.collector.has_unresolved() {
                    return;
                }
                history.visit_allocations(false, &mut IgnoreColumns, &mut |meta, _, _| {
                    if !resolver.collector.has_unresolved() {
                        return;
                    }
                    let extra = K::metadata_extra(meta);
                    if resolver.visit(extra.observation(false)) {
                        K::visit_extra_allocations(extra, false, &mut resolver);
                    }
                });
            },
        );
    }

    #[cfg(test)]
    pub(super) fn visit_new_extra_allocations(
        &self,
        previous: &Self,
        visitor: &mut dyn StorageAllocationVisitor,
    ) {
        let mut collector = ExtraDeltaCollector::default();
        self.collect_changed_extra_allocations(previous, None, &mut collector);
        if collector.has_unresolved() {
            let mut seen = BTreeSet::new();
            previous.resolve_extra_allocations(&mut seen, &mut collector);
        }
        collector.emit(visitor);
    }
}

/// Descends through `SharedColumn` pages without recording page IDs so
/// `resolve_extra_allocations` visits only `Extra` carriers and nested values.
struct IgnoreColumns;
impl StorageAllocationVisitor for IgnoreColumns {
    fn visit(&mut self, _: StorageAllocationObservation) -> bool {
        true
    }
}
