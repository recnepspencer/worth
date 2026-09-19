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
    anchor: worth_ui_host_contract::UiMountedCanonicalBox,
    clip_bounds: worth_ui_host_contract::UiMountedCanonicalBox,
    bounds: UiPresentedPortalBounds,
    paint_bounds: UiPresentedPortalBounds,
    side: UiPortalPlacementSide,
    layer: UiPortalLayerIdentity,
    shielding: super::UiPortalInputShielding,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct UiPresentedPortalBounds(worth_ui_host_contract::UiMountedCanonicalBox);

#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct UiCommittedPortalPlacement(UiPreparedPortalPlacement);

// Every geometry component is sourced from canonical finite boxes and bounded arithmetic.
impl Eq for UiPreparedPortalPlacement {}
impl Eq for UiCommittedPortalPlacement {}
impl Eq for UiPresentedPortalBounds {}

impl UiPreparedPortalPlacement {
    pub(crate) fn for_request(
        request: &super::UiPortalServiceRequest,
        parent: Option<UiCommittedPortalPlacement>,
    ) -> Result<Option<Self>, UiPortalPlacementDenial> {
        super::planning::prepare(request, parent)
    }

    pub(super) const fn planned(
        presentation: worth_ui_host_contract::UiHostObservationPresentationBasis,
        anchor: worth_ui_host_contract::UiMountedCanonicalBox,
        clip_bounds: worth_ui_host_contract::UiMountedCanonicalBox,
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
    pub(crate) const fn anchor(self) -> worth_ui_host_contract::UiMountedCanonicalBox {
        self.anchor
    }
    pub(crate) const fn bounds(self) -> UiPresentedPortalBounds {
        self.bounds
    }
    pub(crate) const fn paint_bounds(self) -> UiPresentedPortalBounds {
        self.paint_bounds
    }
    pub(crate) const fn clip_bounds(self) -> worth_ui_host_contract::UiMountedCanonicalBox {
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
            worth_ui_host_contract::UiMountedCanonicalBox::canonicalize(
                worth_ui_host_contract::UiMountedCanonicalBoxInput {
                    x,
                    y,
                    width,
                    height,
                    coordinate_space: worth_ui_host_contract::UiMountedCoordinateSpace::Viewport,
                },
            )
            .expect("placement arithmetic preserves canonical finite viewport geometry"),
        )
    }

    pub(crate) const fn mounted_box(self) -> worth_ui_host_contract::UiMountedCanonicalBox {
        self.0
    }

    #[cfg(test)]
    pub(crate) fn components(self) -> [f32; 4] {
        [self.0.x(), self.0.y(), self.0.width(), self.0.height()]
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
