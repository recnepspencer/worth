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
    arrangement: super::planning::UiPortalArrangement,
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
        arrangement: super::planning::UiPortalArrangement,
        layer: UiPortalLayerIdentity,
        shielding: super::UiPortalInputShielding,
    ) -> Self {
        Self {
            presentation,
            arrangement,
            layer,
            shielding,
        }
    }

    pub(super) const fn arrangement(self) -> super::planning::UiPortalArrangement {
        self.arrangement
    }

    pub(super) const fn with_arrangement(
        mut self,
        arrangement: super::planning::UiPortalArrangement,
    ) -> Self {
        self.arrangement = arrangement;
        self
    }

    pub(crate) const fn presentation(
        self,
    ) -> worth_ui_host_contract::UiHostObservationPresentationBasis {
        self.presentation
    }
    /// The anchor this placement was fitted to: where the host showed it at
    /// open, then where each accepted frame laid its owner out.
    pub(crate) const fn anchor(self) -> UiPublishedRect {
        self.arrangement.anchor
    }
    pub(crate) const fn bounds(self) -> UiPresentedPortalBounds {
        self.arrangement.bounds
    }
    pub(crate) const fn paint_bounds(self) -> UiPresentedPortalBounds {
        self.arrangement.paint_bounds
    }
    /// The viewport the placement was fitted to, which clips the Portal.
    pub(crate) const fn clip_bounds(self) -> UiPublishedRect {
        self.arrangement.viewport
    }
    #[cfg(test)]
    pub(crate) const fn side(self) -> UiPortalPlacementSide {
        self.arrangement.side
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
