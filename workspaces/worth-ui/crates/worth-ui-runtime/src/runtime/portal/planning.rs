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
    let bounds = anchor.bounds();
    let viewport = request
        .presented_viewport()
        .ok_or(super::UiPortalPlacementDenial::MissingPresentedViewport)?;
    if viewport.presentation() != anchor.presentation() {
        return Err(super::UiPortalPlacementDenial::IncompatibleCoordinateSpace);
    }
    let viewport = viewport.bounds();
    if bounds.coordinate_space() != worth_ui_host_contract::UiMountedCoordinateSpace::Viewport
        || viewport.coordinate_space() != worth_ui_host_contract::UiMountedCoordinateSpace::Viewport
    {
        return Err(super::UiPortalPlacementDenial::IncompatibleCoordinateSpace);
    }
    if bounds.posture() != worth_ui_host_contract::UiMountedGeometryPosture::Area {
        return Err(super::UiPortalPlacementDenial::EmptyAnchor);
    }
    let margin = f32::from(policy.viewport_margin());
    let gap = f32::from(policy.anchor_gap());
    let width = f32::from(policy.preferred_width()).min(viewport.width() - margin * 2.0);
    let viewport_top = viewport.y() + margin;
    let viewport_bottom = viewport.y() + viewport.height() - margin;
    let below = viewport_bottom - (bounds.y() + bounds.height() + gap);
    let above = bounds.y() - gap - viewport_top;
    let desired_height = f32::from(policy.maximum_height());
    let (side, available_height) = if policy.centered() {
        (
            super::UiPortalPlacementSide::Centered,
            viewport.height() - margin * 2.0,
        )
    } else if below >= desired_height {
        (super::UiPortalPlacementSide::Below, below)
    } else if above >= desired_height || above > below {
        (super::UiPortalPlacementSide::Above, above)
    } else if below <= 0.0 && above <= 0.0 {
        (
            super::UiPortalPlacementSide::ViewportFit,
            viewport.height() - margin * 2.0,
        )
    } else {
        (super::UiPortalPlacementSide::Below, below)
    };
    let height = desired_height.min(available_height);
    if width <= 0.0 || height <= 0.0 {
        return Err(super::UiPortalPlacementDenial::InsufficientViewport);
    }
    let minimum_x = viewport.x() + margin;
    let maximum_x = viewport.x() + viewport.width() - margin - width;
    let x = if policy.centered() {
        viewport.x() + (viewport.width() - width) * 0.5
    } else {
        bounds.x().clamp(minimum_x, maximum_x)
    };
    let y = match side {
        super::UiPortalPlacementSide::Below => bounds.y() + bounds.height() + gap,
        super::UiPortalPlacementSide::Above => bounds.y() - gap - height,
        super::UiPortalPlacementSide::ViewportFit => viewport_top,
        super::UiPortalPlacementSide::Centered => viewport.y() + (viewport.height() - height) * 0.5,
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
        super::UiPresentedPortalBounds::new(
            x + content.paint.x() - content.layout.x(),
            y + content.paint.y() - content.layout.y(),
            width + content.paint.width() - content.layout.width(),
            height + content.paint.height() - content.layout.height(),
        )
    } else {
        super::UiPresentedPortalBounds::new(x, y, width, height)
    };
    Ok(Some(super::UiPreparedPortalPlacement::planned(
        anchor.presentation(),
        bounds,
        viewport,
        super::UiPresentedPortalBounds::new(x, y, width, height),
        paint_bounds,
        side,
        super::UiPortalLayerIdentity::planned(request.portal(), request.parent(), depth),
        request.shielding(),
    )))
}
