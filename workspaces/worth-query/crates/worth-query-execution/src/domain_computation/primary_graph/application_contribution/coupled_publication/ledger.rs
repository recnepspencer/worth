//! Qualified use (PR C §1.2 R2): the retained derivations of one assembly
//! round, the touch keys their proofs read, and which a later append made
//! suspect. Marking is append time and keyed; the exact comparison that
//! decides a reopen happens before the next use (`qualify`).
use crate::domain_computation::primary_graph::application_attempt::fact::touch_key::{
    FactTouchKey, TouchProbe,
};
use crate::domain_computation::primary_graph::provider::ParentRead;
use worth_execution::{ExecutionMemoryReservation, ExecutionOrderedMap, ExecutionRequest};

/// One retained derivation: a Verified parent the assembly retained.
pub(super) struct Derivation {
    pub(super) read: ParentRead,
    /// The applied position its retained support was observed at.
    pub(super) position: usize,
    /// The position of the demand that owes it.
    pub(super) owed: usize,
    pub(super) state: DerivationState,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) enum DerivationState {
    Live,
    /// A later append's touches intersect its proof keys. The first such
    /// append is the candidate invalidator.
    Suspect {
        invalidator: usize,
        owed: usize,
    },
    /// Reopened at its recorded invalidator; its window ends there.
    Superseded,
}

#[derive(Default)]
pub(super) struct DerivationLedger {
    entries: Vec<Derivation>,
    _entries_memory: Option<ExecutionMemoryReservation>,
    index: ExecutionOrderedMap<(FactTouchKey, usize), ()>,
    conservative: Vec<usize>,
    _conservative_memory: Option<ExecutionMemoryReservation>,
    suspects: ExecutionOrderedMap<usize, ()>,
}

/// A request stop while keeping the ledger.
pub(super) struct LedgerStop(pub(super) worth_execution::ExecutionOrderedMapDenial);

impl DerivationLedger {
    pub(super) fn is_quiet(&self) -> bool {
        self.suspects.is_empty()
    }

    pub(super) fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    /// Register a retained derivation with the keys of its proofs (`None` for
    /// a conservative proof).
    pub(super) fn retain(
        &mut self,
        execution: &ExecutionRequest<'_, '_>,
        derivation: Derivation,
        keys: impl IntoIterator<Item = Option<Vec<FactTouchKey>>>,
    ) -> Result<(), LedgerStop> {
        let id = self.entries.len();
        let mut conservative = false;
        for fact in keys {
            match fact {
                Some(keys) => {
                    for key in keys {
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
        )
    }

    /// Mark every live derivation older than `member` whose proof keys the
    /// member's certified touches intersect. Returns the hits charged.
    pub(super) fn mark(
        &mut self,
        execution: &ExecutionRequest<'_, '_>,
        probes: impl IntoIterator<Item = TouchProbe>,
        member: usize,
        owed: usize,
    ) -> Result<u64, LedgerStop> {
        let mut hits = 0u64;
        let mut marked_conservative = false;
        for probe in probes {
            match probe {
                TouchProbe::Nothing => {}
                TouchProbe::Everything => {
                    for id in 0..self.entries.len() {
                        hits += 1;
                        self.suspect(execution, id, member, owed)?;
                    }
                    return Ok(hits);
                }
                TouchProbe::Keys(keys) => {
                    for key in keys {
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
                            hits += 1;
                            self.suspect(execution, id, member, owed)?;
                            rank += 1;
                        }
                    }
                    if !marked_conservative {
                        marked_conservative = true;
                        for slot in 0..self.conservative.len() {
                            hits += 1;
                            let id = self.conservative[slot];
                            self.suspect(execution, id, member, owed)?;
                        }
                    }
                }
            }
        }
        Ok(hits)
    }

    fn suspect(
        &mut self,
        execution: &ExecutionRequest<'_, '_>,
        id: usize,
        member: usize,
        owed: usize,
    ) -> Result<(), LedgerStop> {
        let entry = &mut self.entries[id];
        if entry.position >= member || entry.state != DerivationState::Live {
            return Ok(());
        }
        entry.state = DerivationState::Suspect {
            invalidator: member,
            owed,
        };
        self.suspects
            .insert_cloned_native_for_request(&id, &(), *execution)
            .map_err(LedgerStop)
    }

    /// The suspects in registration order, emptied.
    pub(super) fn take_suspects(&mut self) -> Vec<usize> {
        let suspects = self.suspects.keys().copied().collect();
        self.suspects = ExecutionOrderedMap::default();
        suspects
    }

    pub(super) fn entry(&self, id: usize) -> &Derivation {
        &self.entries[id]
    }

    pub(super) fn settle(&mut self, id: usize) {
        self.entries[id].state = DerivationState::Live;
    }

    pub(super) fn supersede(&mut self, id: usize) {
        self.entries[id].state = DerivationState::Superseded;
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
