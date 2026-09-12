use std::num::NonZeroU32;

/// Hard bounds on simultaneous reader acquisitions and distinct protected roots.
/// Sessions share their parent's acquisition; a separate reader charges a slot.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PhysicalReadProtectionPolicy {
    acquisitions: NonZeroU32,
    roots: NonZeroU32,
}

impl PhysicalReadProtectionPolicy {
    pub const fn new(acquisitions: NonZeroU32, roots: NonZeroU32) -> Self {
        Self {
            acquisitions,
            roots,
        }
    }

    pub const fn acquisitions(self) -> NonZeroU32 {
        self.acquisitions
    }

    pub const fn roots(self) -> NonZeroU32 {
        self.roots
    }
}

impl Default for PhysicalReadProtectionPolicy {
    fn default() -> Self {
        Self::new(NonZeroU32::new(64).unwrap(), NonZeroU32::new(16).unwrap())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PhysicalReadProtectionDenial {
    /// Every admitted acquisition slot is held by a reader or descendant session.
    ProtectionLimit,
    /// Capturing another distinct root would exceed the admitted root bound.
    RetainedRootLimit,
    /// Bounded metadata could not be allocated during Store admission.
    MetadataUnavailable,
    /// This Store incarnation has stopped issuing read authority.
    Revoked,
}
