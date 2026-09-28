//! Which Scroll region a gesture over one mounted occurrence addresses, and
//! why.
//!
//! An occurrence that owns a region keeps a gesture for that region, even
//! when it is in turn content of an outer one. An occurrence that owns none
//! travels with the region laid out around it. Both answers name a region
//! owner, so each carries which it is: owning a region and being content of
//! one cannot be mistaken for each other.

use worth_ui_host_contract::UiMountedInstanceIdentity;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum UiAddressedScrollOwner {
    /// The occurrence owns the region the gesture addresses.
    OwnsRegion(UiMountedInstanceIdentity),
    /// The occurrence owns no region and travels as content of this owner's.
    ContentOf(UiMountedInstanceIdentity),
}

impl UiAddressedScrollOwner {
    /// The region owner the gesture addresses.
    pub(crate) const fn owner(self) -> UiMountedInstanceIdentity {
        match self {
            Self::OwnsRegion(owner) | Self::ContentOf(owner) => owner,
        }
    }
}
