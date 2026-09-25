use crate::mounting::presentation::UiPublishedRect;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum UiPortalPlacementSide {
    Below,
    Above,
    ViewportFit,
    Centered,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum UiPortalPlacementDenial {
    MissingPresentedAnchor,
    MissingPresentedViewport,
    IncompatibleCoordinateSpace,
    EmptyAnchor,
    InsufficientViewport,
    UnknownParent,
    LayerDepthExhausted,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct UiPortalLayerIdentity {
    portal: super::UiPortalIdentity,
    parent: Option<super::UiPortalIdentity>,
    depth: u16,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct UiPreparedPortalPlacement {
    presentation: worth_ui_host_contract::UiHostObservationPresentationBasis,
    anchor: UiPublishedRect,
    clip_bounds: UiPublishedRect,
    bounds: UiPresentedPortalBounds,
    paint_bounds: UiPresentedPortalBounds,
    side: UiPortalPlacementSide,
    layer: UiPortalLayerIdentity,
    shielding: super::UiPortalInputShielding,
}

/// Where a Portal's committed placement puts it, in viewport space.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct UiPresentedPortalBounds(UiPublishedRect);

#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct UiCommittedPortalPlacement(UiPreparedPortalPlacement);

// Every geometry component is a published rect, which compares by value.
impl Eq for UiPreparedPortalPlacement {}
impl Eq for UiCommittedPortalPlacement {}

impl UiPreparedPortalPlacement {
    pub(crate) fn for_request(
        request: &super::UiPortalServiceRequest,
        parent: Option<UiCommittedPortalPlacement>,
    ) -> Result<Option<Self>, UiPortalPlacementDenial> {
        super::planning::prepare(request, parent)
    }

    pub(super) const fn planned(
        presentation: worth_ui_host_contract::UiHostObservationPresentationBasis,
        anchor: UiPublishedRect,
        clip_bounds: UiPublishedRect,
        bounds: UiPresentedPortalBounds,
        paint_bounds: UiPresentedPortalBounds,
        side: UiPortalPlacementSide,
        layer: UiPortalLayerIdentity,
        shielding: super::UiPortalInputShielding,
    ) -> Self {
        Self {
            presentation,
            anchor,
            clip_bounds,
            bounds,
            paint_bounds,
            side,
            layer,
            shielding,
        }
    }

    pub(crate) const fn presentation(
        self,
    ) -> worth_ui_host_contract::UiHostObservationPresentationBasis {
        self.presentation
    }
    /// Where the host showed the anchor, as the placement committed it.
    pub(crate) const fn anchor(self) -> UiPublishedRect {
        self.anchor
    }
    pub(crate) const fn bounds(self) -> UiPresentedPortalBounds {
        self.bounds
    }
    pub(crate) const fn paint_bounds(self) -> UiPresentedPortalBounds {
        self.paint_bounds
    }
    pub(crate) const fn clip_bounds(self) -> UiPublishedRect {
        self.clip_bounds
    }
    #[cfg(test)]
    pub(crate) const fn side(self) -> UiPortalPlacementSide {
        self.side
    }
    pub(crate) const fn layer(self) -> UiPortalLayerIdentity {
        self.layer
    }
    pub(crate) const fn shielding(self) -> super::UiPortalInputShielding {
        self.shielding
    }

    pub(crate) const fn with_presentation(
        mut self,
        presentation: worth_ui_host_contract::UiHostObservationPresentationBasis,
    ) -> Self {
        self.presentation = presentation;
        self
    }
}

impl UiPresentedPortalBounds {
    pub(super) fn new(x: f32, y: f32, width: f32, height: f32) -> Self {
        Self(
            UiPublishedRect::from_committed_components(
                [x, y, width, height],
                worth_ui_host_contract::UiMountedCoordinateSpace::Viewport,
            )
            .expect("placement arithmetic preserves canonical finite viewport geometry"),
        )
    }

    pub(crate) const fn rect(self) -> UiPublishedRect {
        self.0
    }

    #[cfg(test)]
    pub(crate) fn components(self) -> [f32; 4] {
        self.0.components()
    }
}

impl UiCommittedPortalPlacement {
    pub(crate) const fn from_prepared(prepared: UiPreparedPortalPlacement) -> Self {
        Self(prepared)
    }

    pub(crate) const fn prepared(self) -> UiPreparedPortalPlacement {
        self.0
    }
}

impl UiPortalLayerIdentity {
    pub(super) const fn planned(
        portal: super::UiPortalIdentity,
        parent: Option<super::UiPortalIdentity>,
        depth: u16,
    ) -> Self {
        Self {
            portal,
            parent,
            depth,
        }
    }
    #[cfg(test)]
    pub(crate) const fn portal(self) -> super::UiPortalIdentity {
        self.portal
    }
    pub(crate) const fn parent(self) -> Option<super::UiPortalIdentity> {
        self.parent
    }
    pub(crate) const fn depth(self) -> u16 {
        self.depth
    }
}
