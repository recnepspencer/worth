#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum UiAllocationInvalidationFamily {
    TextContentChange,
    QueryMeasurementFactChange,
    ContentExtentChange,
    ResizePreviewDelta,
    DurableLocalResizeChange,
    ViewportExtentChange,
    ScrollExtentObservation,
    ScrollOwnedExtentChange,
    PortalAnchorMovement,
    HostMeasurementResultReplacement,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct UiAllocationInvalidationIntent {
    family: UiAllocationInvalidationFamily,
    ingress_ref: UiAllocationFrameIngressRef,
}

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub(crate) struct UiAllocationFrameIngressRef {
    epoch: crate::runtime::UiAllocationFrameEpoch,
    ordinal: u16,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum UiAllocationInvalidationReferenceDenial {
    FrameEpochMismatch {
        plan: crate::runtime::UiAllocationFrameEpoch,
        invalidation: crate::runtime::UiAllocationFrameEpoch,
    },
    ForeignPlanInvalidation {
        plan: crate::runtime::UiAllocationFrameEpoch,
        invalidation: crate::runtime::UiAllocationFrameEpoch,
        ordinal: u16,
    },
    MissingCanonicalIngress {
        ordinal: u16,
        ingress_count: u16,
    },
}

impl UiAllocationInvalidationIntent {
    pub(crate) fn new(
        family: UiAllocationInvalidationFamily,
        ingress_ref: UiAllocationFrameIngressRef,
    ) -> Self {
        Self {
            family,
            ingress_ref,
        }
    }

    pub fn family(&self) -> UiAllocationInvalidationFamily {
        self.family
    }
    pub(crate) fn ingress_ref(&self) -> UiAllocationFrameIngressRef {
        self.ingress_ref
    }
}

impl UiAllocationFrameIngressRef {
    pub(super) fn mint(epoch: crate::runtime::UiAllocationFrameEpoch, index: usize) -> Self {
        Self {
            epoch,
            ordinal: u16::try_from(index)
                .expect("a frame carries at most ALLOCATION_FRAME_SOURCE_COUNT ingress"),
        }
    }

    pub(crate) fn epoch(self) -> crate::runtime::UiAllocationFrameEpoch {
        self.epoch
    }
    pub(crate) fn ordinal(self) -> u16 {
        self.ordinal
    }
}
