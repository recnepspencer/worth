//! Store-owned root pins (Store Milestone 4, Phase M4.1).
//!
//! A pin is a constant-size protection on one root, with an optional parent
//! pin it overlays. Lifecycle is `Active -> Expiring -> Expired`: `Expiring`
//! refuses new reader loans while still protecting the root, and a pin reaches
//! `Expired` only when no reader loan and no live child pin remain, so a child
//! overlay keeps every ancestor's root protected until it drains. Every
//! transition runs through the verified `PinTableKernel`.
//!
//! The durable pin table lives in the physical runtime
//! ([`crate::physical_runtime::stability::durable_pins::PhysicalRootPins`]); [`VolatileRootPinTable`]
//! enforces the identical state machine for a memory Home's lifetime.

pub(crate) mod core;
#[cfg(test)]
mod tests;

use std::sync::Mutex;

pub(crate) use self::core::{RootPinCore, RootPinExpiry};

/// Opaque, strictly monotone pin identity within one table.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct RootPinId(pub(crate) u64);

impl RootPinId {
    /// The pin's ordinal, for owner-side encoding.
    pub const fn ordinal(self) -> u64 {
        self.0
    }
    /// Rebind an ordinal an owner recorded. Naming a pin confers no pin: every
    /// operation still resolves it against the table.
    pub const fn from_ordinal(ordinal: u64) -> Self {
        Self(ordinal)
    }
}

/// Lifecycle state of one pin.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RootPinState {
    Active,
    /// New reader loans are refused; the root stays protected until every loan
    /// and child pin drains.
    Expiring,
    /// The root is released to reclamation.
    Expired,
}

/// What an explicit drop of a parent pin does while child pins remain.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RootPinDropPolicy {
    /// Refuse the drop while any live child pin overlays this pin.
    RefuseIfPinnedByChild,
    /// Fence new loans now; release the root once children and loans drain.
    RetainUntilChildrenDrain,
}

/// Typed pin refusal. A refusal leaves the table unchanged.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RootPinDenial {
    PinNotFound,
    DuplicatePin,
    /// The named parent pin is missing or no longer `Active`.
    ParentPinNotActive,
    /// The pin is `Expiring` or `Expired`; new loans are fenced.
    LeaseExpired,
    /// `RefuseIfPinnedByChild` and a live child pin remains.
    PinnedByChild,
    CounterOverflow,
    PinIdentityExhausted,
    /// The owner payload exceeds the constant-size pin record.
    PayloadTooLarge,
    /// The durable pin table is not restored; nothing can be pinned yet.
    PinsUnavailable,
    /// A reclamation fence is installed on the current root.
    ReclaimFenced,
    /// The durable pin record could not be written; nothing was pinned.
    StorageUnavailable,
    /// The durable pin record's fate is unknown until reopen. The pin keeps
    /// protecting its root in this process.
    StorageIndeterminate,
}

/// Memory-Home pins: the identical verified state machine, held only for the
/// live Home's lifetime. Roots are owner-chosen opaque tokens.
pub struct VolatileRootPinTable {
    core: Mutex<RootPinCore<Vec<u8>>>,
}

impl Default for VolatileRootPinTable {
    fn default() -> Self {
        Self {
            core: Mutex::new(RootPinCore::default()),
        }
    }
}

impl std::fmt::Debug for VolatileRootPinTable {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("VolatileRootPinTable")
            .field("live_pins", &self.lock().live_pins())
            .finish()
    }
}

impl VolatileRootPinTable {
    fn lock(&self) -> std::sync::MutexGuard<'_, RootPinCore<Vec<u8>>> {
        self.core
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
    }

    /// Pin `root` with an owner payload, optionally overlaying `parent`.
    pub fn pin(
        &self,
        root: u64,
        parent: Option<RootPinId>,
        policy: RootPinDropPolicy,
        payload: Vec<u8>,
    ) -> Result<RootPinId, RootPinDenial> {
        self.lock().acquire(root, parent, policy, payload)
    }

    /// Admit one reader loan and return the pinned owner payload.
    pub fn loan(&self, pin: RootPinId) -> Result<Vec<u8>, RootPinDenial> {
        self.lock().loan(pin).cloned()
    }

    /// Release one loan; a drained `Expiring` pin and its drained `Expiring`
    /// ancestors complete at once.
    pub fn release_loan(&self, pin: RootPinId) {
        let mut core = self.lock();
        core.release_loan(pin);
        settle(&mut core, pin);
    }

    /// Request expiry: `Active -> Expiring`, then complete `Expiring -> Expired`
    /// for this pin and each ancestor as soon as its loans and children drain.
    pub fn expire(&self, pin: RootPinId) -> Result<RootPinState, RootPinDenial> {
        let mut core = self.lock();
        if core.expiry(pin)? == RootPinExpiry::BeginExpiring {
            core.expire(pin)?;
        }
        settle(&mut core, pin);
        core.state(pin).ok_or(RootPinDenial::PinNotFound)
    }

    pub fn state(&self, pin: RootPinId) -> Option<RootPinState> {
        self.lock().state(pin)
    }

    /// The owner payload of a pin that still protects its root.
    pub fn payload(&self, pin: RootPinId) -> Option<Vec<u8>> {
        self.lock().live_attachment(pin).cloned()
    }

    pub fn protects_root(&self, root: u64) -> bool {
        self.lock().protects_root(root)
    }

    /// Pin identities that currently protect a root (`Active` or `Expiring`).
    pub fn live_pin_ids(&self) -> Vec<RootPinId> {
        self.lock().live_pin_ids()
    }
}

/// Complete `pin` and then each ancestor while it is `Expiring` and drained.
fn settle<T>(core: &mut RootPinCore<T>, pin: RootPinId) {
    let mut next = Some(pin);
    while let Some(current) = next {
        if core.expiry(current) != Ok(RootPinExpiry::Complete) || core.expire(current).is_err() {
            return;
        }
        next = core.parent(current);
    }
}
