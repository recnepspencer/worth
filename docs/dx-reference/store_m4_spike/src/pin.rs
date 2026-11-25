use crate::{AppError, HistoricalReadDenial, PhysicalRootPointer};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct PinId(u64);

impl PinId {
    pub const fn raw(self) -> u64 {
        self.0
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HomeRootManifest {
    pub product_map_root: Option<PhysicalRootPointer>,
    native_root: PhysicalRootPointer,
    world_root: PhysicalRootPointer,
    signal_root: PhysicalRootPointer,
    _sealed: (),
}

impl HomeRootManifest {
    pub const fn native_root(&self) -> PhysicalRootPointer {
        self.native_root
    }

    pub const fn world_root(&self) -> PhysicalRootPointer {
        self.world_root
    }

    pub const fn signal_root(&self) -> PhysicalRootPointer {
        self.signal_root
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PinScope(u64);

impl PinScope {
    pub const fn new(id: u64) -> Self {
        Self(id)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NestedDropPolicy {
    RefuseIfPinnedByChild,
    RetainUntilChildrenDrain,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PinSpec {
    pub scope: PinScope,
    pub parent_pin: Option<PinId>,
    pub nested_drop_policy: NestedDropPolicy,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LeaseState {
    Active,
    Expiring,
    Expired,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct LeaseTransition {
    state: LeaseState,
}

impl LeaseTransition {
    pub const fn new(state: LeaseState) -> Self {
        Self { state }
    }

    pub const fn state(self) -> LeaseState {
        self.state
    }
}

pub struct PinnedFrontierLease {
    pin_id: PinId,
    manifest: HomeRootManifest,
}

impl PinnedFrontierLease {
    pub fn demo(pin_id: u64) -> Self {
        Self {
            pin_id: PinId(pin_id),
            manifest: HomeRootManifest {
                product_map_root: Some(PhysicalRootPointer(pin_id)),
                native_root: PhysicalRootPointer(1),
                world_root: PhysicalRootPointer(2),
                signal_root: PhysicalRootPointer(3),
                _sealed: (),
            },
        }
    }

    pub const fn pin_id(&self) -> PinId {
        self.pin_id
    }

    pub const fn manifest(&self) -> &HomeRootManifest {
        &self.manifest
    }

    pub fn read_pinned_manifest(&self) -> Result<HomeRootManifest, HistoricalReadDenial> {
        Ok(self.manifest.clone())
    }
}

pub trait RuntimeWorldRetentionPort {
    fn capture_and_pin_frontier(&self, spec: PinSpec) -> Result<PinnedFrontierLease, AppError>;
    fn expire_lease(&self, pin_id: PinId) -> Result<LeaseTransition, AppError>;
}
