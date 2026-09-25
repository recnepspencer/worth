use crate::mounting::presentation::UiPublishedRect;

pub(super) fn prepare(
    request: &super::UiPortalServiceRequest,
    parent: Option<super::UiCommittedPortalPlacement>,
) -> Result<Option<super::UiPreparedPortalPlacement>, super::UiPortalPlacementDenial> {
    if !matches!(
        request.operation(),
        super::request::UiPortalServiceOperation::Open
    ) {
        return Ok(None);
    }
    let anchor = request
        .presented_anchor()
        .ok_or(super::UiPortalPlacementDenial::MissingPresentedAnchor)?;
    let policy = request
        .placement_geometry()
        .ok_or(super::UiPortalPlacementDenial::MissingPresentedAnchor)?;
    // The Portal commits its placement from where the host showed the anchor.
    let anchor_rect = anchor.bounds().adopted();
    let viewport = request
        .presented_viewport()
        .ok_or(super::UiPortalPlacementDenial::MissingPresentedViewport)?;
    if viewport.presentation() != anchor.presentation() {
        return Err(super::UiPortalPlacementDenial::IncompatibleCoordinateSpace);
    }
    let arrangement = arrange(
        anchor_rect,
        viewport.bounds(),
        policy,
        request.content_bounds(),
    )?;
    let depth = match (request.parent(), parent) {
        (None, _) => 0,
        (Some(_), None) => return Err(super::UiPortalPlacementDenial::UnknownParent),
        (Some(_), Some(parent)) => parent
            .prepared()
            .layer()
            .depth()
            .checked_add(1)
            .ok_or(super::UiPortalPlacementDenial::LayerDepthExhausted)?,
    };
    Ok(Some(super::UiPreparedPortalPlacement::planned(
        anchor.presentation(),
        arrangement,
        super::UiPortalLayerIdentity::planned(request.portal(), request.parent(), depth),
        request.shielding(),
    )))
}

/// Where a Portal sits for one anchor and viewport, and the inputs that put
/// it there, so a later frame can place it again by the same arithmetic.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) struct UiPortalArrangement {
    pub(super) anchor: UiPublishedRect,
    pub(super) viewport: UiPublishedRect,
    pub(super) policy: crate::declaration::UiDeclaredPortalPlacementGeometry,
    pub(super) content: Option<super::UiPortalContentBounds>,
    pub(super) bounds: super::UiPresentedPortalBounds,
    pub(super) paint_bounds: super::UiPresentedPortalBounds,
    pub(super) side: super::UiPortalPlacementSide,
}

impl super::UiPreparedPortalPlacement {
    /// Places this Portal again where a successor frame lays out its anchor
    /// and viewport, by the arithmetic that opened it. Its presentation basis,
    /// layer, and shielding carry over; the accepted frame rebinds the basis.
    pub(crate) fn succeeded(
        self,
        anchor: UiPublishedRect,
        viewport: UiPublishedRect,
    ) -> Result<Self, super::UiPortalPlacementDenial> {
        let arrangement = self.arrangement();
        let successor = arrange(anchor, viewport, arrangement.policy, arrangement.content)?;
        Ok(self.with_arrangement(successor))
    }
}

/// Centers a centered Portal in the viewport. Otherwise fits the Portal
/// below its anchor, flips it above when above has its full height, or more
/// room than below, fills the viewport when neither side has any, and else
/// stays below at the height left there; horizontally it clamps inside the
/// margin.
/// Placement is arithmetic within published truth: every input is committed.
fn arrange(
    anchor_rect: UiPublishedRect,
    viewport_rect: UiPublishedRect,
    policy: crate::declaration::UiDeclaredPortalPlacementGeometry,
    content: Option<super::UiPortalContentBounds>,
) -> Result<UiPortalArrangement, super::UiPortalPlacementDenial> {
    if anchor_rect.coordinate_space() != worth_ui_host_contract::UiMountedCoordinateSpace::Viewport
        || viewport_rect.coordinate_space()
            != worth_ui_host_contract::UiMountedCoordinateSpace::Viewport
    {
        return Err(super::UiPortalPlacementDenial::IncompatibleCoordinateSpace);
    }
    if !anchor_rect.has_area() {
        return Err(super::UiPortalPlacementDenial::EmptyAnchor);
    }
    let [anchor_x, anchor_y, _, anchor_height] = anchor_rect.components();
    let [viewport_x, viewport_y, viewport_width, viewport_height] = viewport_rect.components();
    let margin = f32::from(policy.viewport_margin());
    let gap = f32::from(policy.anchor_gap());
    let width = f32::from(policy.preferred_width()).min(viewport_width - margin * 2.0);
    let viewport_top = viewport_y + margin;
    let viewport_bottom = viewport_y + viewport_height - margin;
    let below = viewport_bottom - (anchor_y + anchor_height + gap);
    let above = anchor_y - gap - viewport_top;
    let desired_height = f32::from(policy.maximum_height());
    let (side, available_height) = if policy.centered() {
        (
            super::UiPortalPlacementSide::Centered,
            viewport_height - margin * 2.0,
        )
    } else if below >= desired_height {
        (super::UiPortalPlacementSide::Below, below)
    } else if above >= desired_height || above > below {
        (super::UiPortalPlacementSide::Above, above)
    } else if below <= 0.0 && above <= 0.0 {
        (
            super::UiPortalPlacementSide::ViewportFit,
            viewport_height - margin * 2.0,
        )
    } else {
        (super::UiPortalPlacementSide::Below, below)
    };
    let height = desired_height.min(available_height);
    if width <= 0.0 || height <= 0.0 {
        return Err(super::UiPortalPlacementDenial::InsufficientViewport);
    }
    let minimum_x = viewport_x + margin;
    let maximum_x = viewport_x + viewport_width - margin - width;
    let x = if policy.centered() {
        viewport_x + (viewport_width - width) * 0.5
    } else {
        anchor_x.clamp(minimum_x, maximum_x)
    };
    let y = match side {
        super::UiPortalPlacementSide::Below => anchor_y + anchor_height + gap,
        super::UiPortalPlacementSide::Above => anchor_y - gap - height,
        super::UiPortalPlacementSide::ViewportFit => viewport_top,
        super::UiPortalPlacementSide::Centered => viewport_y + (viewport_height - height) * 0.5,
    };
    let paint_bounds = if let Some(content) = content {
        let [paint_x, paint_y, paint_width, paint_height] = content.paint.components();
        let [layout_x, layout_y, layout_width, layout_height] = content.layout.components();
        super::UiPresentedPortalBounds::new(
            x + paint_x - layout_x,
            y + paint_y - layout_y,
            width + paint_width - layout_width,
            height + paint_height - layout_height,
        )
    } else {
        super::UiPresentedPortalBounds::new(x, y, width, height)
    };
    Ok(UiPortalArrangement {
        anchor: anchor_rect,
        viewport: viewport_rect,
        policy,
        content,
        bounds: super::UiPresentedPortalBounds::new(x, y, width, height),
        paint_bounds,
        side,
    })
}
