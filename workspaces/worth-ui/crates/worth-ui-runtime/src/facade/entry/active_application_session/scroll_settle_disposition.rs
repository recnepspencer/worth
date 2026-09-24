//! What one frame's accepted Scroll samples did to the displayed pose, and the
//! typed reasons any part of that settlement was refused.
//!
//! A refused settlement is never silent. Every accepted sample that resolves is
//! still applied; the first refusal is reported alongside, and the same sample
//! comes back next frame, so a reader who asks the session sees the refusal
//! rather than an offset that quietly stopped following the pixels.

/// What one frame's accepted Scroll samples did to the displayed pose.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum UiScrollSettleDisposition {
    /// Every accepted sample this frame reached the displayed pose and the
    /// owner's offset.
    Applied,
    /// Mounted geometry is mid-presentation. The accepted offsets stay true and
    /// the next frame applies them.
    DeferredPresentationInFlight,
    /// A direct-input or layout candidate owns staged geometry. Reconciliation
    /// waits for that publication rather than overwriting its unaccepted pose.
    DeferredPendingGeometry,
    /// Some part of the settlement was refused. Whatever resolved was still
    /// applied; the refusal names the first sample that was not.
    Refused(UiScrollSettleRefusal),
    /// The settle was owed to a surface generation the host no longer shows.
    /// Nothing moved; the next committed tick settles the new generation.
    Superseded,
    /// No Scroll content group has an accepted sample to settle.
    Idle,
}

/// Why part of a frame's accepted-sample settlement did not happen.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum UiScrollSettleRefusal {
    /// Mounted geometry refused the displayed poses outright, for a reason
    /// other than a presentation in flight. No pose moved.
    Geometry(crate::mounting::UiMountedOccurrenceGeometryDenial),
    /// One accepted sample could not be resolved to the owner it settles.
    Resolution(UiAcceptedScrollSettlementDenial),
    /// The poses were applied, but one owner's offset could not follow them.
    WriteBack(UiScrollWriteBackRefusal),
}

/// Why an accepted Scroll-content sample could not be resolved to the region
/// owner and offset it stands for.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum UiAcceptedScrollSettlementDenial {
    /// Scroll is not installed, yet a Scroll-content sample was retained.
    ScrollUnavailable,
    /// The sampled occurrence has no resolvable Scroll ownership chain.
    Ownership(crate::runtime::scroll::UiScrollOwnershipResolutionDenial),
    /// No region owner in the chain has the Motion owner key the sample names.
    OwnerNotInChain { motion_owner_key: u64 },
    /// The owning region has no mounted content box to measure the sample
    /// against.
    GeometryUnavailable,
    /// The owning region has no current allocation to bind an incarnation to.
    IncarnationUnavailable,
    /// The sample sits before the content's rest position, which no
    /// non-negative offset describes.
    SampleBeforeRest,
}

/// Why one applied accepted sample could not be written back into its Scroll
/// owner's offset. The displayed pose is the accepted sample either way; the
/// owner keeps the previous offset, which is still a real offset the next
/// delta can start from.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum UiScrollWriteBackRefusal {
    /// Scroll is not installed, yet a settlement was resolved.
    ScrollUnavailable,
    /// The settling region no longer has a current mounted identity basis.
    RegionBasisUnavailable {
        region_instance: worth_ui_host_contract::UiMountedInstanceIdentity,
    },
    /// The region's current bounds could not be resolved.
    Bounds {
        region_instance: worth_ui_host_contract::UiMountedInstanceIdentity,
        denial: crate::runtime::scroll::UiScrollBoundsResolutionDenial,
    },
    /// Scroll refused the settlement route.
    Route {
        region_instance: worth_ui_host_contract::UiMountedInstanceIdentity,
        denial: crate::runtime::scroll::UiScrollRouteDenial,
    },
}

impl super::super::WorthUiActiveApplicationSession {
    /// What the most recent accepted-sample settlement did, including any
    /// refusal it reported.
    pub fn last_scroll_settle_disposition(&self) -> UiScrollSettleDisposition {
        self.last_scroll_settle_disposition
    }
}
