#[must_use = "a staged focus reveal must commit with or be discarded by its proposal"]
pub(crate) struct UiStagedFocusReveal {
    target: worth_ui_host_contract::UiMountedInstanceIdentity,
    registrations: Vec<crate::runtime::scroll::UiScrollOwnerRegistration>,
    anchor: crate::runtime::scroll::UiScrollAnchor,
    request: crate::runtime::scroll::UiScrollProgrammaticRevealRequest,
    receipt: crate::runtime::scroll::UiScrollRouteReceipt,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum UiFocusRevealStagingDenial {
    Ownership(crate::runtime::scroll::UiScrollOwnershipResolutionDenial),
    Bounds(crate::runtime::scroll::UiScrollBoundsResolutionDenial),
    GeometryOutOfRange,
    UnpublishedScrollGeometry,
    Route(crate::runtime::scroll::UiScrollRouteDenial),
}

impl super::super::WorthUiApplicationSessionState {
    pub(in crate::runtime) fn stage_focus_reveal(
        &self,
        requirement: crate::runtime::session::service_proposal::UiFocusRevealRequirement,
        frame: &crate::mounting::UiPreparedMountedFrame,
        mounted: &crate::mounting::WorthUiMountedSessionState,
        scroll: &crate::runtime::scroll::UiScrollRuntimeState,
        surface_incarnation: crate::runtime::scroll::UiScrollOwnerIncarnation,
    ) -> Result<Option<UiStagedFocusReveal>, UiFocusRevealStagingDenial> {
        let Some(target) = mounted.current_mounted_identity_basis(requirement.target()) else {
            return Ok(None);
        };
        let chain = scroll
            .ownership_chain(requirement.target())
            .map_err(UiFocusRevealStagingDenial::Ownership)?;
        if chain.owners().is_empty() {
            return Ok(None);
        }
        let surface = target.semantic_surface_identity();
        if scroll.has_pending_direct(surface) {
            return Err(UiFocusRevealStagingDenial::UnpublishedScrollGeometry);
        }
        // A completed layout commits with the first frame that presents its
        // surface, before the proposal bound to that frame settles. The reveal
        // is then measured from that frame and from Scroll with the layout
        // committed; a frame that leaves the layout unpresented cannot carry it.
        let layout = scroll.has_unpresented_layout(surface);
        let row = if layout {
            if !frame
                .surfaces()
                .iter()
                .any(|receipt| receipt.requirement().semantic_surface() == surface)
            {
                return Err(UiFocusRevealStagingDenial::UnpublishedScrollGeometry);
            }
            frame.prepared_hit_row(surface, requirement.target())
        } else {
            presented_hit_row(mounted, surface, requirement.target())
        };
        let Some(row) = row else {
            return Ok(None);
        };
        let [target_x, target_y, target_width, target_height] = row.bounds().adopted().components();
        let [viewport_x, viewport_y, viewport_width, viewport_height] =
            row.clip_bounds().adopted().components();
        let mounted_incarnation =
            crate::runtime::scroll::UiScrollOwnerIncarnation::from_mount_incarnation(
                target.mount_incarnation(),
            );
        let anchor = crate::runtime::scroll::UiScrollAnchor::new(
            requirement.application_item_anchor().map_or_else(
                || crate::runtime::scroll::UiScrollAnchorIdentity::mounted(requirement.target()),
                crate::runtime::scroll::UiScrollAnchorIdentity::application_item,
            ),
            row.mounted().binding(),
            signed_subpixels(target_x)?.max(0),
            signed_subpixels(target_y)?.max(0),
        )
        .ok_or(UiFocusRevealStagingDenial::GeometryOutOfRange)?;
        let mut successor = scroll.clone();
        if layout {
            successor.commit_presented_layout(surface);
        }
        let mut entries = Vec::with_capacity(chain.owners().len());
        let mut registrations = Vec::with_capacity(chain.owners().len());
        for owner in chain.owners().iter().copied() {
            let incarnation = match owner {
                crate::runtime::scroll::UiScrollOwnerIdentity::Region { .. } => mounted_incarnation,
                crate::runtime::scroll::UiScrollOwnerIdentity::Surface(_)
                | crate::runtime::scroll::UiScrollOwnerIdentity::Viewport(_) => surface_incarnation,
            };
            let bounds = self
                .scroll_bounds_for(owner, target.graph_node_identity())
                .map_err(UiFocusRevealStagingDenial::Bounds)?;
            let registration = crate::runtime::scroll::UiScrollOwnerRegistration::new(
                owner,
                incarnation,
                axes_for(bounds),
                bounds,
                crate::runtime::scroll::UiScrollOffset::origin(),
            );
            successor
                .reconcile_rebind(crate::runtime::scroll::UiScrollRebindRequest::new(
                    registration,
                    Some(anchor),
                    crate::runtime::scroll::UiScrollAnchorPolicy::Rebase,
                ))
                .map_err(UiFocusRevealStagingDenial::Route)?;
            registrations.push(registration);
            entries.push(crate::runtime::scroll::UiScrollChainEntry::new(
                owner,
                incarnation,
            ));
        }
        let target_inline = interval(target_x, target_width, viewport_x)?;
        let target_block = interval(target_y, target_height, viewport_y)?;
        let viewport = crate::runtime::scroll::UiScrollViewportExtent::new(
            positive_subpixels(viewport_width)?,
            positive_subpixels(viewport_height)?,
        )
        .ok_or(UiFocusRevealStagingDenial::GeometryOutOfRange)?;
        let request = crate::runtime::scroll::UiScrollProgrammaticRevealRequest::new(
            entries,
            crate::runtime::scroll::UiScrollRevealTarget::new(target_inline, target_block),
            viewport,
            scroll.reveal_alignment(),
        )
        .map_err(UiFocusRevealStagingDenial::Route)?;
        let receipt = successor
            .reveal(request.clone())
            .map_err(UiFocusRevealStagingDenial::Route)?;
        Ok(Some(UiStagedFocusReveal {
            target: requirement.target(),
            registrations,
            anchor,
            request,
            receipt,
        }))
    }
}

impl UiStagedFocusReveal {
    /// Route the reveal on `successor`, a copy of current Scroll truth, and
    /// return the occurrence it reveals with the exact prevalidated receipt.
    ///
    /// A reveal is direct authority over the offset, the same as a thumb
    /// placed on a track, so a caller whose reveal moves an offset stages the
    /// routed successor as a direct placement that lands with the frame
    /// carrying it. Committing it at once would move Scroll off the pose the
    /// host displays with nothing awaiting publication to show for it.
    pub(crate) fn route(
        self,
        successor: &mut crate::runtime::scroll::UiScrollRuntimeState,
    ) -> (
        worth_ui_host_contract::UiMountedInstanceIdentity,
        crate::runtime::scroll::UiScrollRouteReceipt,
    ) {
        for registration in self.registrations {
            successor
                .reconcile_rebind(crate::runtime::scroll::UiScrollRebindRequest::new(
                    registration,
                    Some(self.anchor),
                    crate::runtime::scroll::UiScrollAnchorPolicy::Rebase,
                ))
                .expect("staged focus reveal retains its exact current Scroll owner");
        }
        let routed = successor
            .reveal(self.request)
            .expect("staged focus reveal rebases against current Scroll truth");
        assert_eq!(
            routed, self.receipt,
            "staged focus reveal must route the exact prevalidated Scroll transition"
        );
        (self.target, routed)
    }
}

/// The row the host displays for `target` now, Motion samples included.
fn presented_hit_row(
    mounted: &crate::mounting::WorthUiMountedSessionState,
    surface: worth_ui_host_contract::UiSemanticSurfaceIdentity,
    target: worth_ui_host_contract::UiMountedInstanceIdentity,
) -> Option<crate::mounting::UiPresentedHitTestRow> {
    let presentation = mounted
        .current_publication()?
        .presentation_for_surface(surface)
        .map(|displayed| displayed.basis())?;
    mounted
        .interaction_hit_test_basis(presentation)
        .ok()?
        .rows()
        .iter()
        .copied()
        .find(|row| row.mounted_instance() == target)
}

fn axes_for(
    bounds: crate::runtime::scroll::UiScrollBounds,
) -> crate::runtime::scroll::UiScrollAxes {
    match (
        bounds.max_inline_subpixels() > 0,
        bounds.max_block_subpixels() > 0,
    ) {
        (true, false) => crate::runtime::scroll::UiScrollAxes::Inline,
        (false, true) => crate::runtime::scroll::UiScrollAxes::Block,
        (true, true) | (false, false) => crate::runtime::scroll::UiScrollAxes::Both,
    }
}

fn interval(
    start: f32,
    extent: f32,
    viewport_start: f32,
) -> Result<crate::runtime::scroll::UiScrollRevealInterval, UiFocusRevealStagingDenial> {
    let start = signed_subpixels(start)?
        .checked_sub(signed_subpixels(viewport_start)?)
        .ok_or(UiFocusRevealStagingDenial::GeometryOutOfRange)?
        .max(0);
    let end = start.saturating_add(positive_subpixels(extent)?);
    crate::runtime::scroll::UiScrollRevealInterval::new(start, end)
        .ok_or(UiFocusRevealStagingDenial::GeometryOutOfRange)
}

fn signed_subpixels(value: f32) -> Result<i64, UiFocusRevealStagingDenial> {
    let scaled = f64::from(value)
        * worth_ui_host_contract::UI_HOST_SURFACE_POSITION_SUBPIXELS_PER_UNIT as f64;
    if !scaled.is_finite() || scaled < i64::MIN as f64 || scaled > i64::MAX as f64 {
        return Err(UiFocusRevealStagingDenial::GeometryOutOfRange);
    }
    Ok(scaled.round() as i64)
}

fn positive_subpixels(value: f32) -> Result<i64, UiFocusRevealStagingDenial> {
    let value = signed_subpixels(value)?;
    if value <= 0 {
        Err(UiFocusRevealStagingDenial::GeometryOutOfRange)
    } else {
        Ok(value)
    }
}
