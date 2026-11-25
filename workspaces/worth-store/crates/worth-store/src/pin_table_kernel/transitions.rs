#[cfg(not(verus_keep_ghost))]
use super::VecSetExt;
use super::{PinLifecycleState, PinRecord, PinTableKernel, PinTransitionError};
#[cfg(verus_keep_ghost)]
use vstd::prelude::*;

verus_method! {
    /// Acquire a fresh `Active` pin on `root_id`, optionally linked to an active parent pin.
    pub fn acquire_pin(
        &mut self,
        pin_id: u64,
        root_id: u64,
        parent_pin_id: Option<u64>,
    ) -> (result: Result<(), PinTransitionError>);
    requires {
        old(self).spec_well_formed(),
    }
    ensures {
        final(self).spec_well_formed(),
        match result {
            Ok(()) => final(self).spec_root_protected(root_id)
                && (forall|r: u64| old(self).spec_root_protected(r)
                    ==> final(self).spec_root_protected(r)),
            Err(_) => final(self).spec_entries() == old(self).spec_entries(),
        },
    }
    body_with_ok_proof {
        if self.find_index(pin_id).is_some() {
            return Err(PinTransitionError::DuplicatePinId);
        }

        if let Some(parent_id) = parent_pin_id {
            let parent_idx = match self.find_index(parent_id) {
                Some(idx) => idx,
                None => return Err(PinTransitionError::ParentPinNotActive),
            };
            let parent = self.entries[parent_idx];
            if !matches!(parent.state, PinLifecycleState::Active) {
                return Err(PinTransitionError::ParentPinNotActive);
            }
            if parent.live_child_pins == u64::MAX {
                return Err(PinTransitionError::CounterOverflow);
            }
            let updated_parent = PinRecord {
                live_child_pins: parent.live_child_pins + 1,
                ..parent
            };
            self.entries.set(parent_idx, updated_parent);
        }

        let record = PinRecord {
            pin_id,
            root_id,
            parent_pin_id,
            state: PinLifecycleState::Active,
            active_reader_loans: 0,
            live_child_pins: 0,
        };
        self.entries.push(record);
    }
    ok_proof {
        let last = (self.entries@.len() - 1) as int;
        assert(Self::spec_entry_protects(self.entries@[last], root_id));
        assert forall|r: u64| old(self).spec_root_protected(r)
            implies self.spec_root_protected(r) by {
            let i = choose|i: int| 0 <= i < old(self).entries@.len()
                && Self::spec_entry_protects(old(self).entries@[i], r);
            assert(0 <= i < self.entries@.len());
            assert(Self::spec_entry_protects(self.entries@[i], r));
        };
    }
    ok_value { Ok(()) }
}

verus_method! {
    /// Admit an in-process reader loan against an `Active` pin (`Expiring` fences new loans).
    pub fn acquire_reader_loan(&mut self, pin_id: u64) -> (result: Result<(), PinTransitionError>);
    requires {
        old(self).spec_well_formed(),
    }
    ensures {
        final(self).spec_well_formed(),
        forall|r: u64| old(self).spec_root_protected(r) ==> final(self).spec_root_protected(r),
    }
    body_with_ok_proof {
        let idx = match self.find_index(pin_id) {
            Some(idx) => idx,
            None => return Err(PinTransitionError::PinNotFound),
        };
        let entry = self.entries[idx];
        if !matches!(entry.state, PinLifecycleState::Active) {
            return Err(PinTransitionError::LeaseNotActive);
        }
        if entry.active_reader_loans == u64::MAX {
            return Err(PinTransitionError::CounterOverflow);
        }
        let updated = PinRecord {
            active_reader_loans: entry.active_reader_loans + 1,
            ..entry
        };
        self.entries.set(idx, updated);
    }
    ok_proof {
        assert forall|r: u64| old(self).spec_root_protected(r)
            implies self.spec_root_protected(r) by {
            let i = choose|i: int| 0 <= i < old(self).entries@.len()
                && Self::spec_entry_protects(old(self).entries@[i], r);
            assert(Self::spec_entry_protects(self.entries@[i], r));
        };
    }
    ok_value { Ok(()) }
}

verus_method! {
    /// Release one in-process reader loan from `pin_id`.
    pub fn release_reader_loan(&mut self, pin_id: u64) -> (result: Result<(), PinTransitionError>);
    requires {
        old(self).spec_well_formed(),
    }
    ensures {
        final(self).spec_well_formed(),
        forall|r: u64| old(self).spec_root_protected(r) ==> final(self).spec_root_protected(r),
    }
    body_with_ok_proof {
        let idx = match self.find_index(pin_id) {
            Some(idx) => idx,
            None => return Err(PinTransitionError::PinNotFound),
        };
        let entry = self.entries[idx];
        if entry.active_reader_loans > 0 {
            let updated = PinRecord {
                active_reader_loans: entry.active_reader_loans - 1,
                ..entry
            };
            self.entries.set(idx, updated);
        }
    }
    ok_proof {
        assert forall|r: u64| old(self).spec_root_protected(r)
            implies self.spec_root_protected(r) by {
            let i = choose|i: int| 0 <= i < old(self).entries@.len()
                && Self::spec_entry_protects(old(self).entries@[i], r);
            assert(Self::spec_entry_protects(self.entries@[i], r));
        };
    }
    ok_value { Ok(()) }
}

verus_method! {
    /// Advance `pin_id` along `Active -> Expiring -> Expired`.
    ///
    /// Enforces `LeaseExpiryNoReaderLoss`: never reaches `Expired` while
    /// `active_reader_loans > 0 || live_child_pins > 0`.
    pub fn expire_lease(&mut self, pin_id: u64) -> (result: Result<PinLifecycleState, PinTransitionError>);
    requires {
        old(self).spec_well_formed(),
    }
    ensures {
        final(self).spec_well_formed(),
        match result {
            Ok(PinLifecycleState::Expiring) => exists|i: int|
                0 <= i < final(self).spec_len()
                && final(self).spec_entry_at(i).pin_id == pin_id
                && final(self).spec_root_protected(final(self).spec_entry_at(i).root_id),
            Ok(PinLifecycleState::Expired) => exists|i: int|
                0 <= i < final(self).spec_len()
                && final(self).spec_entry_at(i).pin_id == pin_id
                && final(self).spec_entry_at(i).active_reader_loans == 0
                && final(self).spec_entry_at(i).live_child_pins == 0,
            _ => true,
        },
    }
    body_with_ok_proof {
        let idx = match self.find_index(pin_id) {
            Some(idx) => idx,
            None => return Err(PinTransitionError::PinNotFound),
        };
        let entry = self.entries[idx];
        let next_state = match entry.state {
            PinLifecycleState::Active => {
                let updated = PinRecord {
                    state: PinLifecycleState::Expiring,
                    ..entry
                };
                self.entries.set(idx, updated);
                PinLifecycleState::Expiring
            }
            PinLifecycleState::Expiring => {
                if entry.active_reader_loans == 0 && entry.live_child_pins == 0 {
                    let updated = PinRecord {
                        state: PinLifecycleState::Expired,
                        ..entry
                    };
                    self.entries.set(idx, updated);
                    if let Some(parent_id) = entry.parent_pin_id {
                        if let Some(parent_idx) = self.find_index(parent_id) {
                            if parent_idx != idx {
                                let parent = self.entries[parent_idx];
                                if parent.live_child_pins > 0 {
                                    let updated_parent = PinRecord {
                                        live_child_pins: parent.live_child_pins - 1,
                                        ..parent
                                    };
                                    self.entries.set(parent_idx, updated_parent);
                                }
                            }
                        }
                    }
                    PinLifecycleState::Expired
                } else {
                    PinLifecycleState::Expiring
                }
            }
            PinLifecycleState::Expired => PinLifecycleState::Expired,
        };
    }
    ok_proof {
        assert(self.spec_entry_at(idx as int).pin_id == pin_id);
        if matches!(next_state, PinLifecycleState::Expiring) {
            assert(Self::spec_entry_protects(self.entries@[idx as int], entry.root_id));
        }
    }
    ok_value { Ok(next_state) }
}
