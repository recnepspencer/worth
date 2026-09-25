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
    let viewport_rect = viewport.bounds();
    if anchor_rect.coordinate_space() != worth_ui_host_contract::UiMountedCoordinateSpace::Viewport
        || viewport_rect.coordinate_space()
            != worth_ui_host_contract::UiMountedCoordinateSpace::Viewport
    {
        return Err(super::UiPortalPlacementDenial::IncompatibleCoordinateSpace);
    }
    if !anchor_rect.has_area() {
        return Err(super::UiPortalPlacementDenial::EmptyAnchor);
    }
    // Placement is arithmetic within published truth: every input is committed.
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
    let paint_bounds = if let Some(content) = request.content_bounds() {
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
    Ok(Some(super::UiPreparedPortalPlacement::planned(
        anchor.presentation(),
        anchor_rect,
        viewport_rect,
        super::UiPresentedPortalBounds::new(x, y, width, height),
        paint_bounds,
        side,
        super::UiPortalLayerIdentity::planned(request.portal(), request.parent(), depth),
        request.shielding(),
    )))
}
