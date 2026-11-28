//! One kernel-backed pin table shared by the durable physical pins and the
//! volatile (memory Home) pins. Every lifecycle transition goes through the
//! verified `PinTableKernel`; this core adds only per-pin attachments, the
//! nested drop policy and monotone identity issuance.
use std::collections::BTreeMap;

use super::{RootPinDenial, RootPinDropPolicy, RootPinId, RootPinState};
use crate::pin_table_kernel::{PinLifecycleState, PinRecord, PinTableKernel, PinTransitionError};

/// The next step an expiry request would take, decided without effect.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum RootPinExpiry {
    /// `Active -> Expiring`: fences new loans while retaining the root.
    BeginExpiring,
    /// `Expiring -> Expired`: no loan and no live child remain.
    Complete,
    /// Already `Expiring` with loans or live children outstanding.
    Draining,
    /// Already `Expired`.
    Expired,
}

struct PinAttachment<T> {
    policy: RootPinDropPolicy,
    value: T,
}

pub(crate) struct RootPinCore<T> {
    kernel: PinTableKernel,
    attachments: BTreeMap<u64, PinAttachment<T>>,
    next_pin: u64,
}

impl<T> Default for RootPinCore<T> {
    fn default() -> Self {
        Self {
            kernel: PinTableKernel::new(),
            attachments: BTreeMap::new(),
            next_pin: 1,
        }
    }
}

impl<T> RootPinCore<T> {
    /// Issue the next monotone pin identity and acquire it on `root`.
    pub(crate) fn acquire(
        &mut self,
        root: u64,
        parent: Option<RootPinId>,
        policy: RootPinDropPolicy,
        value: T,
    ) -> Result<RootPinId, RootPinDenial> {
        let pin = self.next_pin;
        let next = pin
            .checked_add(1)
            .ok_or(RootPinDenial::PinIdentityExhausted)?;
        self.acquire_exact(RootPinId(pin), root, parent, policy, value)?;
        self.next_pin = next;
        Ok(RootPinId(pin))
    }

    /// Re-acquire a recorded pin identity during restoration.
    pub(crate) fn acquire_exact(
        &mut self,
        pin: RootPinId,
        root: u64,
        parent: Option<RootPinId>,
        policy: RootPinDropPolicy,
        value: T,
    ) -> Result<(), RootPinDenial> {
        self.kernel
            .acquire_pin(pin.0, root, parent.map(|parent| parent.0))
            .map_err(denial)?;
        self.attachments
            .insert(pin.0, PinAttachment { policy, value });
        self.next_pin = self.next_pin.max(pin.0.saturating_add(1));
        Ok(())
    }

    pub(crate) fn state(&self, pin: RootPinId) -> Option<RootPinState> {
        self.kernel.record(pin.0).map(|record| state(record.state))
    }

    pub(crate) fn root(&self, pin: RootPinId) -> Option<u64> {
        self.kernel.record(pin.0).map(|record| record.root_id)
    }

    pub(crate) fn parent(&self, pin: RootPinId) -> Option<RootPinId> {
        self.kernel
            .record(pin.0)
            .and_then(|record| record.parent_pin_id)
            .map(RootPinId)
    }

    /// The attachment of a pin that still protects its root.
    pub(crate) fn live_attachment(&self, pin: RootPinId) -> Option<&T> {
        let record = self.kernel.record(pin.0)?;
        (record.state != PinLifecycleState::Expired)
            .then(|| self.attachments.get(&pin.0).map(|entry| &entry.value))
            .flatten()
    }

    /// Admit one reader loan; only `Active` pins admit loans.
    pub(crate) fn loan(&mut self, pin: RootPinId) -> Result<&T, RootPinDenial> {
        self.kernel.acquire_reader_loan(pin.0).map_err(denial)?;
        match self.attachments.get(&pin.0) {
            Some(entry) => Ok(&entry.value),
            None => {
                let _ = self.kernel.release_reader_loan(pin.0);
                Err(RootPinDenial::PinNotFound)
            }
        }
    }

    pub(crate) fn release_loan(&mut self, pin: RootPinId) {
        let _ = self.kernel.release_reader_loan(pin.0);
    }

    /// Decide the next expiry step without effect, honouring the drop policy.
    pub(crate) fn expiry(&self, pin: RootPinId) -> Result<RootPinExpiry, RootPinDenial> {
        let record = self
            .kernel
            .record(pin.0)
            .ok_or(RootPinDenial::PinNotFound)?;
        let policy = self
            .attachments
            .get(&pin.0)
            .map_or(RootPinDropPolicy::RetainUntilChildrenDrain, |entry| {
                entry.policy
            });
        Ok(match record.state {
            PinLifecycleState::Active => {
                if policy == RootPinDropPolicy::RefuseIfPinnedByChild && record.live_child_pins > 0
                {
                    return Err(RootPinDenial::PinnedByChild);
                }
                RootPinExpiry::BeginExpiring
            }
            PinLifecycleState::Expiring if drained(record) => RootPinExpiry::Complete,
            PinLifecycleState::Expiring => RootPinExpiry::Draining,
            PinLifecycleState::Expired => RootPinExpiry::Expired,
        })
    }

    /// Apply one kernel expiry step and return the resulting state.
    pub(crate) fn expire(&mut self, pin: RootPinId) -> Result<RootPinState, RootPinDenial> {
        let next = self.kernel.expire_lease(pin.0).map_err(denial)?;
        if next == PinLifecycleState::Expired {
            self.attachments.remove(&pin.0);
        }
        Ok(state(next))
    }

    /// Whether any non-expired pin holds exactly `root`.
    pub(crate) fn protects_root(&self, root: u64) -> bool {
        self.kernel.is_root_protected(root)
    }

    /// Whether any non-expired pin holds a root at or below `root`.
    pub(crate) fn protects_root_at_or_below(&self, root: u64) -> bool {
        self.kernel.is_root_protected_at_or_below(root)
    }

    /// A non-expired pin root at or below `root`, witnessing the verified
    /// kernel's protection answer. Transitive ancestors stay non-expired while
    /// a live child overlays them, so they witness here too.
    pub(crate) fn protected_root_at_or_below(&self, root: u64) -> Option<u64> {
        if !self.kernel.is_root_protected_at_or_below(root) {
            return None;
        }
        let witness = self.attachments.keys().find_map(|pin| {
            self.kernel
                .record(*pin)
                .filter(|record| record.state != PinLifecycleState::Expired)
                .map(|record| record.root_id)
                .filter(|pinned| *pinned <= root)
        });
        // The kernel answered; never weaken it if no attachment names the root.
        Some(witness.unwrap_or(root))
    }

    /// Whether any non-expired pin's attachment satisfies `matches`.
    pub(crate) fn any_live(&self, matches: impl Fn(&T) -> bool) -> bool {
        self.attachments.iter().any(|(pin, entry)| {
            self.kernel
                .record(*pin)
                .is_some_and(|record| record.state != PinLifecycleState::Expired)
                && matches(&entry.value)
        })
    }

    /// Pins whose next expiry step completes `Expiring -> Expired`.
    pub(crate) fn drained(&self) -> Vec<RootPinId> {
        self.attachments
            .keys()
            .map(|pin| RootPinId(*pin))
            .filter(|pin| self.expiry(*pin) == Ok(RootPinExpiry::Complete))
            .collect()
    }

    pub(crate) fn live_pins(&self) -> usize {
        self.attachments.len()
    }

    /// Pin identities that currently protect a root (`Active` or `Expiring`),
    /// in ascending order.
    pub(crate) fn live_pin_ids(&self) -> Vec<RootPinId> {
        self.attachments
            .keys()
            .filter(|pin| {
                self.kernel
                    .record(**pin)
                    .is_some_and(|record| record.state != PinLifecycleState::Expired)
            })
            .map(|pin| RootPinId(*pin))
            .collect()
    }
}

const fn drained(record: PinRecord) -> bool {
    record.active_reader_loans == 0 && record.live_child_pins == 0
}

const fn state(state: PinLifecycleState) -> RootPinState {
    match state {
        PinLifecycleState::Active => RootPinState::Active,
        PinLifecycleState::Expiring => RootPinState::Expiring,
        PinLifecycleState::Expired => RootPinState::Expired,
    }
}

const fn denial(error: PinTransitionError) -> RootPinDenial {
    match error {
        PinTransitionError::DuplicatePinId => RootPinDenial::DuplicatePin,
        PinTransitionError::PinNotFound => RootPinDenial::PinNotFound,
        PinTransitionError::ParentPinNotActive => RootPinDenial::ParentPinNotActive,
        PinTransitionError::LeaseNotActive => RootPinDenial::LeaseExpired,
        PinTransitionError::CounterOverflow => RootPinDenial::CounterOverflow,
    }
}
