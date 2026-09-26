#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct UiResizeLogicalExtent(u32);

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum UiResizeLogicalExtentDenial {
    NotFinite,
    Negative,
    OutOfRange,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct UiResizePreviewSample {
    target: crate::graph::UiGraphNodeIdentity,
    extent: UiResizeLogicalExtent,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct UiDurableResizeCommitIntent {
    authority: crate::runtime::WorthUiAdmittedDurableResizeInput,
    extent: UiResizeLogicalExtent,
}

impl UiResizeLogicalExtent {
    pub fn try_from_logical_pixels(value: f32) -> Result<Self, UiResizeLogicalExtentDenial> {
        if !value.is_finite() {
            return Err(UiResizeLogicalExtentDenial::NotFinite);
        }
        if value < 0.0 {
            return Err(UiResizeLogicalExtentDenial::Negative);
        }
        crate::whole_number::whole_u32(f64::from((value * 64.0).round()))
            .map(Self)
            .ok_or(UiResizeLogicalExtentDenial::OutOfRange)
    }
    pub fn subpixels(self) -> u32 {
        self.0
    }
}

impl UiResizePreviewSample {
    pub fn new(target: crate::graph::UiGraphNodeIdentity, extent: UiResizeLogicalExtent) -> Self {
        Self { target, extent }
    }
    pub fn target(self) -> crate::graph::UiGraphNodeIdentity {
        self.target
    }
    pub fn extent(self) -> UiResizeLogicalExtent {
        self.extent
    }
}

impl UiDurableResizeCommitIntent {
    pub fn terminal(
        authority: crate::runtime::WorthUiAdmittedDurableResizeInput,
        extent: UiResizeLogicalExtent,
    ) -> Self {
        Self { authority, extent }
    }
    pub fn authority(&self) -> &crate::runtime::WorthUiAdmittedDurableResizeInput {
        &self.authority
    }
    pub fn extent(&self) -> UiResizeLogicalExtent {
        self.extent
    }
}
