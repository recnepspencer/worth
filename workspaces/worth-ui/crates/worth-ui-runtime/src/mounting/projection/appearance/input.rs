use super::fact::UiMountedAppearanceNodeInput;
use super::style::ResolvedNodeStyle;

struct ResolvedMountingBasis {
    bounds: worth_ui_host_contract::UiAppearanceAllocationBounds,
    surface_paint_posture: crate::mounting::UiMountedSurfacePaintPosture,
    attribution: worth_ui_host_contract::UiMountedNodeAppearanceAttribution,
}

impl UiMountedAppearanceNodeInput {
    pub(crate) fn from_resolved_projection(
        source: super::UiResolvedAppearanceNodeSource<'_>,
    ) -> Result<Option<Self>, super::UiMountedAppearanceLoweringDenial> {
        let Some(basis) = resolve_mounting_basis(&source)? else {
            return Ok(None);
        };
        let super::UiResolvedAppearanceNodeSource {
            issuer,
            semantic_surface,
            node_receipt,
            graph_node: _,
            plan_digest: _,
            allocation: _,
            clip,
            surface_paint_order,
            geometry_input,
            text_foreground_spans,
            projection,
            outline_fringe,
        } = source;
        let ResolvedMountingBasis {
            bounds,
            surface_paint_posture,
            attribution,
        } = basis;
        let ResolvedNodeStyle {
            mut radii,
            fill,
            border,
            outline,
            foreground,
            opacity: appearance_opacity,
        } = super::style::resolve(bounds, projection)?;
        let surface_paint = lower_surface_paint(fill, border);
        radii = retain_exterior_radii(bounds, radii, surface_paint_posture.exterior_corners());
        let surface_border_omissions = lower_border_omissions(&surface_paint_posture)?;
        let outline = lower_outline(outline, bounds, radii, outline_fringe)?;
        Ok(Some(Self {
            geometry_input,
            issuer,
            semantic_surface,
            node_receipt,
            projection: attribution,
            bounds,
            clip,
            surface_paint_order,
            radii,
            surface_border_edges: surface_paint_posture.border_edges(),
            surface_border_omissions,
            surface_paint,
            outline,
            text_foregrounds: lower_text_foregrounds(foreground, text_foreground_spans),
            appearance_opacity,
            motion_opacity: None,
            semantic_digest: projection.semantic_digest(),
            portal_instance: None,
        }))
    }
}

fn resolve_mounting_basis(
    source: &super::UiResolvedAppearanceNodeSource<'_>,
) -> Result<Option<ResolvedMountingBasis>, super::UiMountedAppearanceLoweringDenial> {
    source.clip.require_resolved()?;
    if source.clip == super::UiMountedAppearanceClip::Suppressed {
        return Ok(None);
    }
    let bounds = match source.allocation {
        worth_ui_host_contract::UiMountedAllocationProjection::Known { bounds, .. }
        | worth_ui_host_contract::UiMountedAllocationProjection::PortalAnchorObservation {
            bounds,
            ..
        } => bounds,
        worth_ui_host_contract::UiMountedAllocationProjection::Omitted(_) => {
            return Err(super::UiMountedAppearanceLoweringDenial::NodeAllocationUnavailable);
        }
    };
    let bounds = super::geometry::allocation(bounds)
        .map_err(super::UiMountedAppearanceLoweringDenial::Geometry)?;
    let surface_paint_posture = source
        .geometry_input
        .as_ref()
        .map(super::UiMountedAppearanceGeometryInput::surface_paint_posture)
        .unwrap_or_default();
    let attribution =
        worth_ui_host_contract::UiMountedNodeAppearanceAttribution::from_runtime_mounting(
            source.issuer,
            source.graph_node.digest(),
            source.plan_digest,
        )
        .ok_or(super::UiMountedAppearanceLoweringDenial::NodeProjectionUnavailable)?;
    Ok(Some(ResolvedMountingBasis {
        bounds,
        surface_paint_posture,
        attribution,
    }))
}

fn lower_surface_paint(
    fill: Option<worth_ui_host_contract::UiMountedAppearanceColor>,
    border: Option<(
        worth_ui_host_contract::UiMountedAppearanceColor,
        worth_ui_host_contract::UiAppearanceLogicalLength,
    )>,
) -> Option<worth_ui_host_contract::UiMountedSurfacePaint> {
    match (fill, border) {
        (Some(fill), Some((border, inward_width))) => Some(
            worth_ui_host_contract::UiMountedSurfacePaint::FillAndBorder {
                fill,
                border,
                inward_width,
            },
        ),
        (Some(fill), None) => Some(worth_ui_host_contract::UiMountedSurfacePaint::Fill(fill)),
        (None, Some((color, inward_width))) => {
            Some(worth_ui_host_contract::UiMountedSurfacePaint::Border {
                color,
                inward_width,
            })
        }
        (None, None) => None,
    }
}

fn lower_text_foregrounds(
    foreground: Option<worth_ui_host_contract::UiMountedAppearanceColor>,
    spans: &[super::UiMountedAppearanceTextSpanInput],
) -> Box<[super::fact::UiMountedAppearanceTextForegroundInput]> {
    foreground
        .into_iter()
        .flat_map(|foreground| {
            spans.iter().flat_map(move |span| {
                let span_identity = span.identity();
                span.geometry()
                    .iter()
                    .cloned()
                    .map(
                        move |geometry| super::fact::UiMountedAppearanceTextForegroundInput {
                            span: span_identity,
                            command: geometry.command(),
                            geometry: std::sync::Arc::from([geometry]),
                            foreground,
                            motion_opacity: None,
                        },
                    )
                    .collect::<Vec<_>>()
            })
        })
        .collect()
}

fn lower_border_omissions(
    posture: &crate::mounting::UiMountedSurfacePaintPosture,
) -> Result<
    Box<[worth_ui_host_contract::UiMountedSurfaceBorderOmission]>,
    super::UiMountedAppearanceLoweringDenial,
> {
    let mut output = Vec::new();
    for omission in posture.border_omissions() {
        let start = super::geometry::extent(omission.start())
            .map_err(super::UiMountedAppearanceLoweringDenial::Geometry)?;
        let end = super::geometry::extent(omission.end())
            .map_err(super::UiMountedAppearanceLoweringDenial::Geometry)?;
        if let Some(omission) =
            worth_ui_host_contract::UiMountedSurfaceBorderOmission::from_runtime_mosaic(
                omission.side(),
                start,
                end,
            )
        {
            output.push(omission);
        }
    }
    Ok(output.into_boxed_slice())
}

fn retain_exterior_radii(
    bounds: worth_ui_host_contract::UiAppearanceAllocationBounds,
    radii: worth_ui_host_contract::UiAppearanceNormalizedLogicalRadii,
    exterior: [bool; 4],
) -> worth_ui_host_contract::UiAppearanceNormalizedLogicalRadii {
    let corners = radii.corners();
    let corners = std::array::from_fn(|index| {
        if exterior[index] {
            worth_ui_host_contract::UiAppearanceLogicalLength::new(
                i32::try_from(corners[index]).expect("normalized radius retains authored range"),
            )
            .unwrap()
        } else {
            worth_ui_host_contract::UiAppearanceLogicalLength::ZERO
        }
    });
    worth_ui_host_contract::UiAppearanceNormalizedLogicalRadii::normalize(bounds, corners)
}

fn lower_outline(
    authored: Option<worth_ui_dsl::UiThemeOutline>,
    bounds: worth_ui_host_contract::UiAppearanceAllocationBounds,
    radii: worth_ui_host_contract::UiAppearanceNormalizedLogicalRadii,
    fringe: Result<
        worth_ui_host_contract::UiAppearanceLogicalLength,
        super::UiMountedAppearanceLoweringDenial,
    >,
) -> Result<
    Option<super::fact::UiMountedAppearanceOutlineInput>,
    super::UiMountedAppearanceLoweringDenial,
> {
    authored
        .map(|authored| {
            let width = worth_ui_host_contract::UiAppearanceLogicalLength::new(
                authored.stroke().width().subpixels(),
            )
            .map_err(|_| super::UiMountedAppearanceLoweringDenial::OutlineWidthInvalid)?;
            let offset = worth_ui_host_contract::UiAppearanceLogicalLength::new(
                authored.offset().subpixels(),
            )
            .map_err(|_| super::UiMountedAppearanceLoweringDenial::OutlineOffsetInvalid)?;
            let geometry = worth_ui_host_contract::UiAppearanceOutlineGeometry::admit(
                bounds, radii, width, offset, fringe?,
            )
            .map_err(super::UiMountedAppearanceLoweringDenial::OutlineGeometry)?;
            Ok(super::fact::UiMountedAppearanceOutlineInput {
                geometry,
                color: super::style::mounted_color(authored.stroke().color()),
            })
        })
        .transpose()
}

impl super::UiMountedAppearanceLoweringInput {
    pub(crate) fn retain_node_owned_families(
        &mut self,
        portal_instances: &std::collections::BTreeSet<
            worth_ui_host_contract::UiMountedInstanceIdentity,
        >,
    ) {
        for node in &mut self.nodes {
            if portal_instances.contains(&node.node_receipt.mounted_instance()) {
                node.surface_paint = None;
                node.portal_instance = None;
            }
        }
    }

    pub(crate) fn retain_portal_surface(
        &mut self,
        portal: worth_ui_host_contract::UiMountedInstanceIdentity,
    ) -> Result<(), super::UiMountedAppearanceLoweringDenial> {
        let Some(node) = self.nodes.first_mut() else {
            return Ok(());
        };
        if node.node_receipt.mounted_instance() != portal {
            return Err(super::UiMountedAppearanceLoweringDenial::PortalSurfaceMissing);
        }
        if node.surface_paint.is_none() {
            self.nodes.clear();
            return Ok(());
        }
        node.portal_instance = Some(portal);
        node.outline = None;
        node.text_foregrounds = Box::new([]);
        Ok(())
    }

    pub(crate) fn compose_accepted_motion(
        &mut self,
        scope: &super::UiMountedAppearanceGeometryScope,
    ) -> Result<(), super::UiMountedAppearanceLoweringDenial> {
        for node in &mut self.nodes {
            node.motion_opacity = if node.surface_paint.is_some()
                || node.outline.is_some()
                || node.portal_instance.is_some()
            {
                scope
                    .instance_motion_opacity(
                        node.node_receipt.mounted_instance(),
                        node.portal_instance.is_some(),
                    )?
                    .flatten()
            } else {
                None
            };
            for foreground in &mut node.text_foregrounds {
                foreground.motion_opacity = scope.motion_opacity(foreground.command).flatten();
            }
        }
        for backdrop in &mut self.backdrops {
            backdrop.motion_opacity = backdrop
                .motion_target
                .map(|target| scope.instance_motion_opacity(target, true))
                .transpose()?
                .flatten()
                .flatten();
        }
        Ok(())
    }
}
