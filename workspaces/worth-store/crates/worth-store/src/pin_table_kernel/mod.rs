#![allow(unexpected_cfgs)]
// Wired into Store's `reclaim_guard` in Phase M4.1.
#![allow(dead_code)]

//! Pure physical root-pin transition kernel for Store Milestone 4 (`M4.0` / `M4.1`).
//!
//! Compiled directly inside `worth-store` under plain `cargo` with **zero `vstd`
//! dependency**, while `tools/verus/run_verus.py check` verifies the exact same
//! production function bodies directly against formal pre/postconditions
//! (`StoreGCPinSafety` and `LeaseExpiryNoReaderLoss`).

#[macro_use]
mod macros;
#[cfg(all(test, not(verus_keep_ghost)))]
mod tests;
mod transitions;

#[cfg(verus_keep_ghost)]
use vstd::prelude::*;

verus_items! {
    /// Lifecycle state of one durable root-pin lease (`Active -> Expiring -> Expired`).
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub(crate) enum PinLifecycleState {
        Active,
        Expiring,
        Expired,
    }

    /// Typed transition refusal from the pure pin-table kernel.
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub(crate) enum PinTransitionError {
        DuplicatePinId,
        PinNotFound,
        ParentPinNotActive,
        LeaseNotActive,
        CounterOverflow,
    }

    /// Pure physical record for one durable pin entry (zero semantic/branch vocabulary).
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub(crate) struct PinRecord {
        pub(crate) pin_id: u64,
        pub(crate) root_id: u64,
        pub(crate) parent_pin_id: Option<u64>,
        pub(crate) state: PinLifecycleState,
        pub(crate) active_reader_loans: u64,
        pub(crate) live_child_pins: u64,
    }

    /// Pure executable transition kernel for Store's durable `PinTable`.
    #[derive(Debug, Clone, PartialEq, Eq, Default)]
    pub(crate) struct PinTableKernel {
        entries: Vec<PinRecord>,
    }
}

#[cfg(verus_keep_ghost)]
verus! {
    impl PinTableKernel {
        pub(crate) closed spec fn spec_len(&self) -> int {
            self.entries@.len() as int
        }

        spec fn spec_entry_protects(entry: PinRecord, root_id: u64) -> bool {
            entry.root_id == root_id && entry.state != PinLifecycleState::Expired
        }

        pub(crate) closed spec fn spec_root_protected(&self, root_id: u64) -> bool {
            exists|i: int| 0 <= i < self.entries@.len()
                && Self::spec_entry_protects(self.entries@[i], root_id)
        }

        pub(crate) closed spec fn spec_has_pin_id(&self, pin_id: u64) -> bool {
            exists|i: int| 0 <= i < self.entries@.len()
                && self.entries@[i].pin_id == pin_id
        }

        pub(crate) closed spec fn spec_well_formed(&self) -> bool {
            (forall|i: int, j: int|
                0 <= i < self.entries@.len() && 0 <= j < self.entries@.len() && i != j
                ==> self.entries@[i].pin_id != self.entries@[j].pin_id)
            && (forall|i: int|
                0 <= i < self.entries@.len() && self.entries@[i].state == PinLifecycleState::Expired
                ==> self.entries@[i].active_reader_loans == 0
                    && self.entries@[i].live_child_pins == 0)
        }

        pub(crate) closed spec fn spec_entry_at(&self, idx: int) -> PinRecord {
            self.entries@[idx]
        }

        pub(crate) closed spec fn spec_entries(&self) -> Seq<PinRecord> {
            self.entries@
        }
    }
}

verus_method! {
    /// Construct an empty pin table.
    pub(crate) fn new() -> (result: Self);
    ensures {
        result.spec_len() == 0,
        result.spec_well_formed(),
    }
    body {
        PinTableKernel { entries: Vec::new() }
    }
}

verus_method! {
    /// Number of tracked pin entries.
    pub(crate) fn len(&self) -> (result: usize);
    ensures {
        result as int == self.spec_len(),
    }
    body {
        self.entries.len()
    }
}

verus_method! {
    /// Whether the table currently holds zero pin entries.
    pub(crate) fn is_empty(&self) -> (result: bool);
    ensures {
        result == (self.spec_len() == 0),
    }
    body {
        self.entries.is_empty()
    }
}

verus_scan_method! {
    /// Locate an entry index by `pin_id`.
    pub(crate) fn find_index(&self, pin_id: u64) -> (result: Option<usize>);
    ensures {
        match result {
            Some(idx) => (idx as int) < self.spec_len()
                && self.spec_entry_at(idx as int).pin_id == pin_id,
            None => !self.spec_has_pin_id(pin_id),
        },
    }
    init {
        let mut i: usize = 0;
    }
    while (i < self.entries.len())
    invariant {
        0 <= i <= self.entries@.len(),
        forall|k: int| 0 <= k < i ==> self.entries@[k].pin_id != pin_id,
    }
    decreases { self.entries@.len() - i, }
    step {
        if self.entries[i].pin_id == pin_id {
            return Some(i);
        }
        i += 1;
    }
    after {
        None
    }
}

verus_scan_method! {
    /// Query whether `root_id` is currently protected by any non-expired pin.
    pub(crate) fn is_root_protected(&self, root_id: u64) -> (result: bool);
    ensures {
        result == self.spec_root_protected(root_id),
    }
    init {
        let mut i: usize = 0;
    }
    while (i < self.entries.len())
    invariant {
        0 <= i <= self.entries@.len(),
        forall|k: int| 0 <= k < i
            ==> !Self::spec_entry_protects(self.entries@[k], root_id),
    }
    decreases { self.entries@.len() - i, }
    step {
        let entry = self.entries[i];
        let is_expired = matches!(entry.state, PinLifecycleState::Expired);
        if entry.root_id == root_id && !is_expired {
            return true;
        }
        i += 1;
    }
    after {
        false
    }
}

#[cfg(not(verus_keep_ghost))]
trait VecSetExt<T> {
    fn set(&mut self, index: usize, value: T);
}

#[cfg(not(verus_keep_ghost))]
impl<T> VecSetExt<T> for Vec<T> {
    #[inline]
    fn set(&mut self, index: usize, value: T) {
        self[index] = value;
    }
}
