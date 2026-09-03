use super::fact::UiMountedAppearanceNodeInput;
use worth_ui_host_contract::{
    UiAppearanceAllocationBounds, UiMountedNodeReceiptIdentity, UiMountedNodeReceiptIssuer,
    UiSemanticSurfaceIdentity,
};

impl UiMountedAppearanceNodeInput {
    pub(crate) fn from_resolved_projection(
        issuer: UiMountedNodeReceiptIssuer,
        semantic_surface: UiSemanticSurfaceIdentity,
        node_receipt: UiMountedNodeReceiptIdentity,
        graph_node: crate::graph::UiGraphNodeIdentity,
        plan_digest: u64,
        allocation: worth_ui_host_contract::UiMountedAllocationProjection,
        projection: &crate::runtime::appearance::UiAppearanceProjection,
    ) -> Result<Self, super::UiMountedAppearanceLoweringDenial> {
        let bounds = match allocation {
            worth_ui_host_contract::UiMountedAllocationProjection::Known { bounds, .. }
            | worth_ui_host_contract::UiMountedAllocationProjection::PortalAnchorObservation {
                bounds,
                ..
            } => bounds,
            worth_ui_host_contract::UiMountedAllocationProjection::Omitted(_) => {
                return Err(super::UiMountedAppearanceLoweringDenial::NodeAllocationUnavailable)
            }
        };
        let bounds = runtime_bounds(bounds)?;
        let attribution =
            worth_ui_host_contract::UiMountedNodeAppearanceAttribution::from_runtime_mounting(
                issuer,
                graph_node.digest(),
                plan_digest,
            )
            .ok_or(super::UiMountedAppearanceLoweringDenial::NodeProjectionUnavailable)?;
        let mut radii = worth_ui_host_contract::UiAppearanceNormalizedLogicalRadii::normalize(
            bounds,
            [worth_ui_host_contract::UiAppearanceLogicalLength::ZERO; 4],
        );
        let mut fill = None;
        let mut border = None;
        let mut outline = None;
        let mut appearance_opacity = worth_ui_host_contract::UiMountedAppearanceOpacity::ONE;
        for aspect in projection.aspects() {
            if aspect.support() != crate::runtime::appearance::UiAppearanceSupportPosture::Supported
            {
                continue;
            }
            match (aspect.aspect(), aspect.value()) {
                (
                    worth_ui_dsl::UiAppearanceAspect::Background,
                    worth_ui_dsl::UiThemeValue::Color(color),
                ) => {
                    fill = Some(
                        worth_ui_host_contract::UiMountedAppearanceColor::from_straight_srgba(
                            color.channels(),
                        ),
                    )
                }
                (
                    worth_ui_dsl::UiAppearanceAspect::Border,
                    worth_ui_dsl::UiThemeValue::SolidStroke(stroke),
                ) => {
                    let width = worth_ui_host_contract::UiAppearanceLogicalLength::new(
                        stroke.width().subpixels(),
                    )
                    .map_err(|_| {
                        super::UiMountedAppearanceLoweringDenial::NodeAllocationUnavailable
                    })?;
                    border = Some((
                        worth_ui_host_contract::UiMountedAppearanceColor::from_straight_srgba(
                            stroke.color().channels(),
                        ),
                        width,
                    ));
                }
                (
                    worth_ui_dsl::UiAppearanceAspect::Radius,
                    worth_ui_dsl::UiThemeValue::CornerRadii(authored),
                ) => {
                    let authored = authored
                        .corners()
                        .into_iter()
                        .map(|length| {
                            worth_ui_host_contract::UiAppearanceLogicalLength::new(
                                length.subpixels(),
                            )
                        })
                        .collect::<Result<Vec<_>, _>>()
                        .map_err(|_| {
                            super::UiMountedAppearanceLoweringDenial::NodeAllocationUnavailable
                        })?;
                    radii = worth_ui_host_contract::UiAppearanceNormalizedLogicalRadii::normalize(
                        bounds,
                        [authored[0], authored[1], authored[2], authored[3]],
                    );
                }
                (
                    worth_ui_dsl::UiAppearanceAspect::Opacity,
                    worth_ui_dsl::UiThemeValue::Opacity(opacity),
                ) => {
                    appearance_opacity =
                        worth_ui_host_contract::UiMountedAppearanceOpacity::from_units(
                            opacity.units(),
                        );
                }
                (
                    worth_ui_dsl::UiAppearanceAspect::Outline,
                    worth_ui_dsl::UiThemeValue::SolidOutline(authored),
                ) => outline = Some(authored),
                _ => {}
            }
        }
        let surface_paint = match (fill, border) {
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
        };
        let outline = outline
            .map(|authored| {
                let width = worth_ui_host_contract::UiAppearanceLogicalLength::new(
                    authored.stroke().width().subpixels(),
                )
                .map_err(|_| super::UiMountedAppearanceLoweringDenial::NodeAllocationUnavailable)?;
                let offset = worth_ui_host_contract::UiAppearanceLogicalLength::new(
                    authored.offset().subpixels(),
                )
                .map_err(|_| super::UiMountedAppearanceLoweringDenial::NodeAllocationUnavailable)?;
                let geometry = worth_ui_host_contract::UiAppearanceOutlineGeometry::admit(
                    bounds,
                    radii,
                    width,
                    offset,
                    worth_ui_host_contract::UiAppearanceLogicalLength::ZERO,
                )
                .map_err(|_| super::UiMountedAppearanceLoweringDenial::NodeAllocationUnavailable)?;
                Ok(super::fact::UiMountedAppearanceOutlineInput {
                    geometry,
                    color: worth_ui_host_contract::UiMountedAppearanceColor::from_straight_srgba(
                        authored.stroke().color().channels(),
                    ),
                })
            })
            .transpose()?;
        Ok(Self {
            issuer,
            semantic_surface,
            node_receipt,
            projection: attribution,
            bounds,
            clip: worth_ui_host_contract::UiAppearanceClip::new(
                bounds.x(),
                bounds.y(),
                bounds.width(),
                bounds.height(),
            )
            .map_err(|_| super::UiMountedAppearanceLoweringDenial::NodeAllocationUnavailable)?,
            layer: worth_ui_host_contract::UiMountedLayerProjection::Omitted(
                worth_ui_host_contract::UiMountedOmissionReason::NotDefinedByCurrentRuntime,
            ),
            radii,
            surface_paint,
            outline,
            text_foregrounds: Box::new([]),
            pointer: None,
            appearance_opacity,
            motion_opacity: None,
            semantic_digest: projection.semantic_digest(),
            portal_instance: None,
        })
    }
}

fn runtime_bounds(
    bounds: worth_ui_host_contract::UiMountedCanonicalBox,
) -> Result<UiAppearanceAllocationBounds, super::UiMountedAppearanceLoweringDenial> {
    if bounds.posture() != worth_ui_host_contract::UiMountedGeometryPosture::Area {
        return Err(super::UiMountedAppearanceLoweringDenial::NodeAllocationUnavailable);
    }
    let x = exact_i32(bounds.x())?;
    let y = exact_i32(bounds.y())?;
    let width = exact_u32(bounds.width())?;
    let height = exact_u32(bounds.height())?;
    UiAppearanceAllocationBounds::new(x, y, width, height)
        .map_err(|_| super::UiMountedAppearanceLoweringDenial::NodeAllocationUnavailable)
}

fn exact_i32(value: f32) -> Result<i32, super::UiMountedAppearanceLoweringDenial> {
    let candidate = value as i32;
    (candidate as f32 == value)
        .then_some(candidate)
        .ok_or(super::UiMountedAppearanceLoweringDenial::NodeAllocationUnavailable)
}

fn exact_u32(value: f32) -> Result<u32, super::UiMountedAppearanceLoweringDenial> {
    let candidate = value as u32;
    (candidate as f32 == value)
        .then_some(candidate)
        .ok_or(super::UiMountedAppearanceLoweringDenial::NodeAllocationUnavailable)
}
