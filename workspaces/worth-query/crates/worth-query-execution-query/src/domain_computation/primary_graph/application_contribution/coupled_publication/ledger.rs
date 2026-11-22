//! Qualified use (PR C §1.2 R2): the retained derivations of one assembly
//! round and the touch keys their proofs read. Marking is append time and
//! keyed; every marked derivation is compared exactly at that same append,
//! so the append that invalidates it is authenticated, never inferred.
use crate::domain_computation::primary_graph::application_attempt::fact::touch_key::{
    FactTouchKey, TouchProbe,
};
use crate::domain_computation::primary_graph::provider::ParentRead;
use worth_execution::{ExecutionMemoryReservation, ExecutionOrderedMap, ExecutionRequest};

/// One derivation qualified use watches: a verified parent the assembly
/// retained, or a producer part (`writer`, its index in the writer roster).
pub(super) struct Derivation {
    pub(super) read: ParentRead,
    pub(super) writer: Option<usize>,
    /// The applied position its retained support was observed at.
    pub(super) position: usize,
    /// A wave task owns it and can derive it again.
    pub(super) claimed: bool,
    pub(super) superseded: bool,
    /// The last member that marked it, so one append compares it once.
    marked_by: Option<usize>,
}

impl Derivation {
    pub(super) fn new(read: ParentRead, position: usize) -> Self {
        Self::part(read, position, None)
    }

    pub(super) fn part(read: ParentRead, position: usize, writer: Option<usize>) -> Self {
        Self {
            read,
            writer,
            position,
            claimed: false,
            superseded: false,
            marked_by: None,
        }
    }
}

#[derive(Default)]
pub(super) struct DerivationLedger {
    /// In registration order, which is read order.
    entries: Vec<Derivation>,
    _entries_memory: Option<ExecutionMemoryReservation>,
    index: ExecutionOrderedMap<(FactTouchKey, usize), ()>,
    conservative: Vec<usize>,
    _conservative_memory: Option<ExecutionMemoryReservation>,
}

/// A request stop while keeping the ledger.
pub(super) struct LedgerStop(pub(super) worth_execution::ExecutionOrderedMapDenial);

impl DerivationLedger {
    pub(super) fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    /// Register a retained derivation with the keys of its proofs (`None` for
    /// a conservative proof). Returns the index entries inserted.
    pub(super) fn retain(
        &mut self,
        execution: &ExecutionRequest<'_, '_>,
        derivation: Derivation,
        keys: impl IntoIterator<Item = Option<Vec<FactTouchKey>>>,
    ) -> Result<u64, LedgerStop> {
        let id = self.entries.len();
        let mut conservative = false;
        let mut inserted = 0u64;
        for fact in keys {
            match fact {
                Some(keys) => {
                    for key in keys {
                        inserted += 1;
                        self.index
                            .insert_cloned_native_for_request(&(key, id), &(), *execution)
                            .map_err(LedgerStop)?;
                    }
                }
                None => conservative = true,
            }
        }
        if conservative {
            funded_push(
                &mut self.conservative,
                &mut self._conservative_memory,
                id,
                execution,
            )?;
        }
        funded_push(
            &mut self.entries,
            &mut self._entries_memory,
            derivation,
            execution,
        )?;
        Ok(inserted)
    }

    /// The entry retained as `read`, by binary search over read order.
    pub(super) fn find(&self, read: ParentRead) -> Option<usize> {
        self.entries
            .binary_search_by_key(&read.0, |entry| entry.read.0)
            .ok()
    }

    /// The work of one `find`.
    pub(super) fn find_work(&self) -> u64 {
        self.entries.len().max(1).ilog2() as u64 + 1
    }

    /// The derivation `read` names, if registered and still claimed.
    pub(super) fn claimed(&self, read: ParentRead) -> bool {
        self.find(read).is_some_and(|id| self.entries[id].claimed)
    }

    /// A wave task owns the derivation retained as `read`.
    pub(super) fn claim(&mut self, read: ParentRead) {
        if let Some(id) = self.find(read) {
            self.entries[id].claimed = true;
        }
    }

    /// The live derivations older than `member` whose proof keys the member's
    /// certified touches intersect, each once. Returns them and the work.
    pub(super) fn mark(
        &mut self,
        probes: impl IntoIterator<Item = TouchProbe>,
        member: usize,
    ) -> (Vec<usize>, u64) {
        let mut hits = Vec::new();
        let mut work = 0u64;
        let mut marked_conservative = false;
        for probe in probes {
            match probe {
                TouchProbe::Nothing => {}
                TouchProbe::Everything => {
                    for id in 0..self.entries.len() {
                        work += 1;
                        self.hit(id, member, &mut hits);
                    }
                    return (hits, work);
                }
                TouchProbe::Keys(keys) => {
                    for key in keys {
                        work += self.index.len().max(1).ilog2() as u64 + 1;
                        let Some(mut rank) = self
                            .index
                            .first_at_or_after(&(key.clone(), 0))
                            .and_then(|(first, ())| self.index.rank_of(first))
                        else {
                            continue;
                        };
                        while let Some((&(ref found, id), ())) = self.index.entry_at(rank) {
                            if *found != key {
                                break;
                            }
                            work += 1;
                            self.hit(id, member, &mut hits);
                            rank += 1;
                        }
                    }
                    if !marked_conservative {
                        marked_conservative = true;
                        for slot in 0..self.conservative.len() {
                            work += 1;
                            let id = self.conservative[slot];
                            self.hit(id, member, &mut hits);
                        }
                    }
                }
            }
        }
        (hits, work)
    }

    fn hit(&mut self, id: usize, member: usize, hits: &mut Vec<usize>) {
        let entry = &mut self.entries[id];
        if entry.position >= member || entry.superseded || entry.marked_by == Some(member) {
            return;
        }
        entry.marked_by = Some(member);
        hits.push(id);
    }

    pub(super) fn entry(&self, id: usize) -> &Derivation {
        &self.entries[id]
    }

    pub(super) fn supersede(&mut self, id: usize) {
        self.entries[id].superseded = true;
    }
}

fn funded_push<T>(
    vec: &mut Vec<T>,
    memory: &mut Option<ExecutionMemoryReservation>,
    item: T,
    execution: &ExecutionRequest<'_, '_>,
) -> Result<(), LedgerStop> {
    if vec.len() == vec.capacity() {
        let next = vec.capacity().saturating_mul(2).max(4);
        let bytes = u64::try_from(next.saturating_mul(size_of::<T>())).unwrap_or(u64::MAX);
        let reserved = execution
            .allocation_policy()
            .reserve_memory(bytes)
            .map_err(|cause| {
                LedgerStop(worth_execution::ExecutionOrderedMapDenial::Allocation(
                    cause,
                ))
            })?;
        let mut replacement = Vec::new();
        if replacement.try_reserve_exact(next).is_err() || replacement.capacity() != next {
            return Err(LedgerStop(
                worth_execution::ExecutionOrderedMapDenial::CapacityOverflow,
            ));
        }
        replacement.append(vec);
        *vec = replacement;
        *memory = reserved;
    }
    vec.push(item);
    Ok(())
}
