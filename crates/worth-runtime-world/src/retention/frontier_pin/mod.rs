//! Whole-Home frontier pins (Store Milestone 4, Phase M4.1).
//!
//! World defines the narrow retention port; the Home's physical owner
//! implements it over Store's durable pin table, and a memory Home implements
//! it over the identical verified state machine for its lifetime. World never
//! imports Store: a pin is an opaque [`PinId`] plus a sealed
//! [`HomeRootManifest`].
mod manifest;
#[cfg(test)]
mod tests;

pub use manifest::{
    HomeFrontierComponents, HomeRootManifest, PhysicalRootPointer, HOME_ROOT_MANIFEST_VERSION,
};

/// Opaque pin identity, strictly monotone within one Home.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct PinId(u64);

impl PinId {
    /// Issued only by the Home's retention owner.
    pub const fn from_retention_owner(ordinal: u64) -> Self {
        Self(ordinal)
    }
    pub const fn ordinal(self) -> u64 {
        self.0
    }
}

/// Caller-chosen scope recorded with a pin (e.g. a fork or session identity).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct PinScope(u64);

impl PinScope {
    pub const fn new(id: u64) -> Self {
        Self(id)
    }
    pub const fn get(self) -> u64 {
        self.0
    }
}

/// What dropping a parent pin does while child overlays remain.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NestedDropPolicy {
    /// Refuse the drop while any live child pin overlays this pin.
    RefuseIfPinnedByChild,
    /// Fence new loans now; keep every ancestor root reachable until each
    /// child and grandchild drains, then release.
    RetainUntilChildrenDrain,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PinSpec {
    pub scope: PinScope,
    pub parent_pin: Option<PinId>,
    pub nested_drop_policy: NestedDropPolicy,
}

/// `BranchLease` lifecycle: `Active -> Expiring -> Expired`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LeaseState {
    Active,
    /// New reader loans are refused; the pinned roots and parent overlay
    /// edges stay retained until every loan and child pin drains.
    Expiring,
    /// The pin is released to Store's reclamation horizon.
    Expired,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct LeaseTransition {
    pin: PinId,
    state: LeaseState,
}

impl LeaseTransition {
    pub const fn new(pin: PinId, state: LeaseState) -> Self {
        Self { pin, state }
    }
    pub const fn pin(self) -> PinId {
        self.pin
    }
    pub const fn state(self) -> LeaseState {
        self.state
    }
}

/// Typed retention refusal. A refusal writes no durable pin.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RetentionPortDenial {
    PinNotFound,
    /// The named parent pin is missing or no longer `Active`.
    ParentPinNotActive,
    /// The pin is `Expiring` or `Expired`; new loans are fenced.
    LeaseExpired,
    /// `RefuseIfPinnedByChild` and a live child pin remains.
    PinnedByChild,
    /// Store cannot pin right now (reclaim fence, unrestored pins, exhaustion).
    StorageUnavailable,
    /// The durable pin entry's fate is unknown until reopen; the root stays
    /// protected in this process.
    StorageIndeterminate,
    /// The recorded frontier failed its seal.
    CorruptManifest,
}

/// Owner custody of one pinned frontier. Dropping the lease drops its pin
/// under the pin's [`NestedDropPolicy`].
pub trait FrontierPinCustody: std::fmt::Debug + Send + Sync {
    /// Read the pinned frontier through a fresh reader loan.
    fn read_pinned_manifest(&self) -> Result<HomeRootManifest, RetentionPortDenial>;
    /// Called exactly once when the lease is dropped.
    fn release(&self);
}

/// One pinned whole-Home frontier.
#[derive(Debug)]
pub struct PinnedFrontierLease {
    pin_id: PinId,
    manifest: HomeRootManifest,
    custody: Option<Box<dyn FrontierPinCustody>>,
}

impl PinnedFrontierLease {
    /// Assembled only by the Home's retention owner.
    pub fn from_retention_owner(
        pin_id: PinId,
        manifest: HomeRootManifest,
        custody: Box<dyn FrontierPinCustody>,
    ) -> Self {
        Self {
            pin_id,
            manifest,
            custody: Some(custody),
        }
    }
    pub const fn pin_id(&self) -> PinId {
        self.pin_id
    }
    pub const fn manifest(&self) -> &HomeRootManifest {
        &self.manifest
    }
    /// Re-read the pinned frontier under a reader loan; `LeaseExpired` once
    /// this pin is `Expiring`.
    pub fn read_pinned_manifest(&self) -> Result<HomeRootManifest, RetentionPortDenial> {
        match &self.custody {
            Some(custody) => custody.read_pinned_manifest(),
            None => Err(RetentionPortDenial::PinNotFound),
        }
    }

    /// Bind this pin to a durable catalog entry (a named branch, archived
    /// branch or tag): the pin then outlives this handle and stays until it is
    /// expired by id through the retention port.
    pub fn into_catalog_pin(mut self) -> (PinId, HomeRootManifest) {
        self.custody = None;
        (self.pin_id, self.manifest.clone())
    }
}

impl Drop for PinnedFrontierLease {
    fn drop(&mut self) {
        if let Some(custody) = self.custody.take() {
            custody.release();
        }
    }
}

/// Narrow whole-Home retention port (Platform DX `W1.1` extends this shape).
pub trait RuntimeWorldRetentionPort: Send + Sync {
    /// O(1) capture of the current settled frontier under Store's reclaim fence.
    fn capture_and_pin_frontier(
        &self,
        spec: PinSpec,
    ) -> Result<PinnedFrontierLease, RetentionPortDenial>;
    /// `Active -> Expiring` now; `Expiring -> Expired` once loans and child
    /// pins drain.
    fn expire_lease(&self, pin: PinId) -> Result<LeaseTransition, RetentionPortDenial>;
    fn lease_state(&self, pin: PinId) -> Option<LeaseState>;
    /// The recorded frontier of a pin that still protects its roots, without
    /// taking a loan (recovery and catalog rebinding).
    fn pinned_manifest(&self, pin: PinId) -> Result<HomeRootManifest, RetentionPortDenial>;
    /// Maintenance: complete every drained `Expiring` pin. Returns how many
    /// reached `Expired`.
    fn settle_drained(&self) -> Result<usize, RetentionPortDenial>;
}

use worth_execution::report::Disposition::*;

crate::denial_report::world_denial_report!(RetentionPortDenial, "retention", {
    Self::PinNotFound => ("pin_not_found", NotFound, Final, "The requested frontier pin is unavailable."),
    Self::ParentPinNotActive => ("parent_pin_not_active", Conflict, CorrectInput, "The parent pin must be active."),
    Self::LeaseExpired => ("lease_expired", Conflict, Final, "The frontier lease no longer admits reader loans."),
    Self::PinnedByChild => ("pinned_by_child", Conflict, Final, "An active child pin prevents release."),
    Self::StorageUnavailable => ("storage_unavailable", Unavailable, Final, "The storage owner cannot admit a frontier pin."),
    Self::StorageIndeterminate => ("storage_indeterminate", Indeterminate, Escalate, "Reopen the Home to determine whether the frontier pin was recorded."),
    Self::CorruptManifest => ("corrupt_manifest", RelationLaw, Escalate, "The recorded frontier manifest failed its seal."),
});

impl std::error::Error for RetentionPortDenial {}
