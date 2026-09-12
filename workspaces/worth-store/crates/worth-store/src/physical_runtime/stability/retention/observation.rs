use std::sync::Arc;

use super::super::{PhysicalProtectedRootObservation, PhysicalReadProtectionPolicy};
use super::RootProtectionRegistry;

/// Descriptive live observations. This surface cannot capture or release roots.
#[derive(Clone)]
pub struct PhysicalReadProtectionObserver {
    registry: Arc<RootProtectionRegistry>,
}

impl PhysicalReadProtectionObserver {
    pub(in crate::physical_runtime::stability) fn new(
        registry: Arc<RootProtectionRegistry>,
    ) -> Self {
        Self { registry }
    }

    pub fn snapshot(&self) -> PhysicalReadProtectionObservation {
        self.registry.observation()
    }

    /// Looks up the exact issued root in the live index, without scanning readers.
    /// A zero result is observation only, never retirement or deletion authority.
    pub fn acquisitions_for_root(&self, root: PhysicalProtectedRootObservation) -> u32 {
        self.registry.root_acquisitions(root)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PhysicalReadProtectionObservation {
    policy: PhysicalReadProtectionPolicy,
    live_acquisitions: u32,
    protected_roots: u32,
    acquisitions: u64,
    releases: u64,
    index_probes: u64,
    revoked: bool,
}

impl PhysicalReadProtectionObservation {
    pub(super) const fn new(
        policy: PhysicalReadProtectionPolicy,
        live_acquisitions: u32,
        protected_roots: u32,
        acquisitions: u64,
        releases: u64,
        index_probes: u64,
        revoked: bool,
    ) -> Self {
        Self {
            policy,
            live_acquisitions,
            protected_roots,
            acquisitions,
            releases,
            index_probes,
            revoked,
        }
    }

    pub const fn policy(self) -> PhysicalReadProtectionPolicy {
        self.policy
    }
    pub const fn live_acquisitions(self) -> u32 {
        self.live_acquisitions
    }
    pub const fn protected_roots(self) -> u32 {
        self.protected_roots
    }
    pub const fn acquisitions(self) -> u64 {
        self.acquisitions
    }
    pub const fn releases(self) -> u64 {
        self.releases
    }
    pub const fn index_probes(self) -> u64 {
        self.index_probes
    }
    pub const fn revoked(self) -> bool {
        self.revoked
    }
}
