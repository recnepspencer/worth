#[cfg(not(verus_keep_ghost))]
use super::VecSetExt;
use super::{PinLifecycleState, PinRecord, PinTableKernel, PinTransitionError};
#[cfg(verus_keep_ghost)]
use vstd::prelude::*;

verus_method! {
    /// Acquire a fresh `Active` pin on `root_id`, optionally linked to an active parent pin.
    pub(crate) fn acquire_pin(
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
            Ok(()) => final(self).spec_len() == old(self).spec_len() + 1
                && final(self).spec_entry_at(old(self).spec_len()) == (PinRecord {
                    pin_id,
                    root_id,
                    parent_pin_id,
                    state: PinLifecycleState::Active,
                    active_reader_loans: 0,
                    live_child_pins: 0,
                })
                && final(self).spec_root_protected(root_id)
                && (forall|r: u64| old(self).spec_root_protected(r)
                    ==> final(self).spec_root_protected(r))
                && (match parent_pin_id {
                    Some(pid) => exists|p: int| 0 <= p < old(self).spec_len()
                        && old(self).spec_entry_at(p).pin_id == pid
                        && old(self).spec_entry_at(p).state == PinLifecycleState::Active
                        && final(self).spec_entry_at(p) == (PinRecord {
                            live_child_pins: (old(self).spec_entry_at(p).live_child_pins + 1) as u64,
                            ..old(self).spec_entry_at(p)
                        })
                        && (forall|k: int| 0 <= k < old(self).spec_len() && k != p
                            ==> final(self).spec_entry_at(k) == old(self).spec_entry_at(k)),
                    None => forall|k: int| 0 <= k < old(self).spec_len()
                        ==> final(self).spec_entry_at(k) == old(self).spec_entry_at(k),
                }),
            Err(_) => final(self).spec_entries() == old(self).spec_entries(),
        },
    }
    body_with_ok_proof {
        if self.find_index(pin_id).is_some() {
            return Err(PinTransitionError::DuplicatePinId);
        }

        let _parent_idx_opt = match parent_pin_id {
            Some(parent_id) => {
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
                Some(parent_idx)
            }
            None => None,
        };

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
        assert(self.spec_entry_at(last) == record);
        assert(Self::spec_entry_protects(self.entries@[last], root_id));
        if let Some(p_idx) = _parent_idx_opt {
            let p = p_idx as int;
            assert(0 <= p < old(self).spec_len());
            assert(self.spec_entry_at(p) == (PinRecord {
                live_child_pins: (old(self).spec_entry_at(p).live_child_pins + 1) as u64,
                ..old(self).spec_entry_at(p)
            }));
        }
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
    pub(crate) fn acquire_reader_loan(&mut self, pin_id: u64) -> (result: Result<(), PinTransitionError>);
    requires {
        old(self).spec_well_formed(),
    }
    ensures {
        final(self).spec_well_formed(),
        forall|r: u64| old(self).spec_root_protected(r) ==> final(self).spec_root_protected(r),
        match result {
            Ok(()) => final(self).spec_len() == old(self).spec_len()
                && (exists|idx: int| 0 <= idx < old(self).spec_len()
                    && old(self).spec_entry_at(idx).pin_id == pin_id
                    && old(self).spec_entry_at(idx).state == PinLifecycleState::Active
                    && final(self).spec_entry_at(idx) == (PinRecord {
                        active_reader_loans: (old(self).spec_entry_at(idx).active_reader_loans + 1) as u64,
                        ..old(self).spec_entry_at(idx)
                    })
                    && (forall|k: int| 0 <= k < old(self).spec_len() && k != idx
                        ==> final(self).spec_entry_at(k) == old(self).spec_entry_at(k))),
            Err(_) => final(self).spec_entries() == old(self).spec_entries(),
        },
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
        assert(old(self).spec_entry_at(idx as int).pin_id == pin_id);
        assert(self.spec_entry_at(idx as int) == updated);
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
    pub(crate) fn release_reader_loan(&mut self, pin_id: u64) -> (result: Result<(), PinTransitionError>);
    requires {
        old(self).spec_well_formed(),
    }
    ensures {
        final(self).spec_well_formed(),
        forall|r: u64| old(self).spec_root_protected(r) ==> final(self).spec_root_protected(r),
        match result {
            Ok(()) => final(self).spec_len() == old(self).spec_len()
                && (exists|idx: int| 0 <= idx < old(self).spec_len()
                    && old(self).spec_entry_at(idx).pin_id == pin_id
                    && final(self).spec_entry_at(idx) == (PinRecord {
                        active_reader_loans: if old(self).spec_entry_at(idx).active_reader_loans > 0 {
                            (old(self).spec_entry_at(idx).active_reader_loans - 1) as u64
                        } else {
                            0
                        },
                        ..old(self).spec_entry_at(idx)
                    })
                    && (forall|k: int| 0 <= k < old(self).spec_len() && k != idx
                        ==> final(self).spec_entry_at(k) == old(self).spec_entry_at(k))),
            Err(_) => final(self).spec_entries() == old(self).spec_entries(),
        },
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
        assert(old(self).spec_entry_at(idx as int).pin_id == pin_id);
        assert(self.spec_entry_at(idx as int) == (PinRecord {
            active_reader_loans: if old(self).spec_entry_at(idx as int).active_reader_loans > 0 {
                (old(self).spec_entry_at(idx as int).active_reader_loans - 1) as u64
            } else {
                0
            },
            ..old(self).spec_entry_at(idx as int)
        }));
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
    /// `active_reader_loans > 0 || live_child_pins > 0`, and preserves every
    /// unrelated entry (decrementing only the linked parent's `live_child_pins`
    /// by exactly 1 upon `Expiring -> Expired`).
    pub(crate) fn expire_lease(&mut self, pin_id: u64) -> (result: Result<PinLifecycleState, PinTransitionError>);
    requires {
        old(self).spec_well_formed(),
    }
    ensures {
        final(self).spec_well_formed(),
        match result {
            Ok(next_state) => final(self).spec_len() == old(self).spec_len()
                && (exists|idx: int| 0 <= idx < old(self).spec_len()
                    && old(self).spec_entry_at(idx).pin_id == pin_id
                    && final(self).spec_entry_at(idx) == (PinRecord {
                        state: next_state,
                        ..old(self).spec_entry_at(idx)
                    })
                    && (match old(self).spec_entry_at(idx).state {
                        PinLifecycleState::Active => next_state == PinLifecycleState::Expiring,
                        PinLifecycleState::Expiring =>
                            if old(self).spec_entry_at(idx).active_reader_loans == 0
                                && old(self).spec_entry_at(idx).live_child_pins == 0
                            {
                                next_state == PinLifecycleState::Expired
                            } else {
                                next_state == PinLifecycleState::Expiring
                            },
                        PinLifecycleState::Expired => next_state == PinLifecycleState::Expired,
                    })
                    && (next_state == PinLifecycleState::Expiring
                        ==> final(self).spec_root_protected(final(self).spec_entry_at(idx).root_id))
                    && (next_state == PinLifecycleState::Expired
                        ==> final(self).spec_entry_at(idx).active_reader_loans == 0
                            && final(self).spec_entry_at(idx).live_child_pins == 0)
                    && (forall|k: int| 0 <= k < old(self).spec_len() && k != idx ==>
                        if old(self).spec_entry_at(idx).state == PinLifecycleState::Expiring
                            && next_state == PinLifecycleState::Expired
                            && old(self).spec_entry_at(idx).parent_pin_id
                                == Some(old(self).spec_entry_at(k).pin_id)
                        {
                            final(self).spec_entry_at(k) == (PinRecord {
                                live_child_pins: if old(self).spec_entry_at(k).live_child_pins > 0 {
                                    (old(self).spec_entry_at(k).live_child_pins - 1) as u64
                                } else {
                                    0
                                },
                                ..old(self).spec_entry_at(k)
                            })
                        } else {
                            final(self).spec_entry_at(k) == old(self).spec_entry_at(k)
                        })),
            Err(_) => final(self).spec_entries() == old(self).spec_entries(),
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
                    let parent_idx_opt = match entry.parent_pin_id {
                        Some(parent_id) => self.find_index(parent_id),
                        None => None,
                    };
                    match parent_idx_opt {
                        Some(parent_idx) if parent_idx != idx => {
                            let parent = self.entries[parent_idx];
                            if parent.live_child_pins > 0 {
                                let updated_parent = PinRecord {
                                    live_child_pins: parent.live_child_pins - 1,
                                    ..parent
                                };
                                self.entries.set(parent_idx, updated_parent);
                            }
                        }
                        _ => {}
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
        assert(old(self).spec_entry_at(idx as int).pin_id == pin_id);
        assert(self.spec_entry_at(idx as int) == (PinRecord {
            state: next_state,
            ..old(self).spec_entry_at(idx as int)
        }));
        if matches!(next_state, PinLifecycleState::Expiring) {
            assert(Self::spec_entry_protects(self.entries@[idx as int], entry.root_id));
        }
    }
    ok_value { Ok(next_state) }
}
