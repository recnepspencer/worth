//! Lowering one painted scroll-chrome rectangle into a host mechanic.
//!
//! The rectangle arrives already derived, already snapped to the device grid
//! and already clipped to its region's viewport; nothing here moves it. What
//! this file does is resolve the declared role's aspects into the paint the
//! host contract transports and complete that transport, so the same rectangle
//! the pointer was resolved against is the rectangle a host paints.
//!
//! Chrome rides in the fragment of the Scroll region occurrence that reserved
//! its gutter: inside that occurrence's own node fragment when it paints an
//! appearance, or as a fragment of its own when it does not. Either way the
//! occurrence is a scope anchor and nothing more: the chrome issues no
//! projection and answers no hit test.

use worth_ui_host_contract::{
    UiAppearanceAllocationBounds, UiAppearanceClip, UiMountedAppearanceColor,
    UiMountedAppearanceMechanic, UiMountedAppearanceOpacity, UiMountedInstanceIdentity,
    UiMountedScrollChromeAppearanceAttribution, UiMountedScrollChromeAxis,
    UiMountedScrollChromeCompletionInput, UiMountedScrollChromeIdentity,
    UiMountedScrollChromeMechanic, UiMountedScrollChromePart, UiSemanticSurfaceIdentity,
};

use super::UiMountedAppearanceLoweringDenial;
use crate::mounting::presentation::compose_opacity;
use crate::mounting::UiMountedScrollChromeNode;
use crate::runtime::appearance::{UiAppearanceSupportPosture, UiScrollChromeAppearanceProjection};
use crate::runtime::scroll::chrome::{UiScrollChromeAxis, UiScrollChromePart};

/// One painted chrome rectangle, resolved and ready to complete.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct UiMountedAppearanceScrollChromeInput {
    pub(super) identity: UiMountedScrollChromeIdentity,
    pub(super) semantic_surface: UiSemanticSurfaceIdentity,
    pub(super) rect: UiAppearanceAllocationBounds,
    pub(super) clip: UiAppearanceClip,
    pub(super) background: UiMountedAppearanceColor,
    pub(super) radii: worth_ui_host_contract::UiAppearanceNormalizedLogicalRadii,
    pub(super) appearance_opacity: UiMountedAppearanceOpacity,
    pub(super) motion_opacity: Option<u16>,
    pub(super) attribution: UiMountedScrollChromeAppearanceAttribution,
    pub(super) semantic_digest: u64,
}

impl UiMountedAppearanceScrollChromeInput {
    /// Already resolved geometry and opacity for retained physical sampling.
    /// This exposes no theme evaluator or authority to change Scroll state.
    pub(in crate::mounting) fn sample_target(
        &self,
    ) -> Result<
        (
            UiMountedScrollChromeIdentity,
            worth_ui_host_contract::UiMountedCanonicalBox,
            worth_ui_host_contract::UiMountedCanonicalBox,
            UiMountedAppearanceOpacity,
        ),
        (),
    > {
        use worth_ui_host_contract::{
            UiMountedCanonicalBox, UiMountedCanonicalBoxInput, UiMountedCoordinateSpace,
        };
        let unit = worth_ui_host_contract::UI_APPEARANCE_LOGICAL_SUBPIXELS_PER_POINT as f32;
        let bounds = |x: i32, y: i32, width: u32, height: u32| {
            UiMountedCanonicalBox::canonicalize(UiMountedCanonicalBoxInput {
                x: x as f32 / unit,
                y: y as f32 / unit,
                width: width as f32 / unit,
                height: height as f32 / unit,
                coordinate_space: UiMountedCoordinateSpace::Viewport,
            })
            .map_err(|_| ())
        };
        Ok((
            self.identity,
            bounds(
                self.rect.x(),
                self.rect.y(),
                self.rect.width(),
                self.rect.height(),
            )?,
            bounds(
                self.clip.x(),
                self.clip.y(),
                self.clip.width(),
                self.clip.height(),
            )?,
            self.appearance_opacity,
        ))
    }
    /// Marry one derived rectangle to the appearance its role resolved to.
    ///
    /// A part whose role declares no Background is refused rather than painted
    /// from a default colour: chrome the theme did not describe is chrome the
    /// theme did not ask for. Radius and Opacity are optional and default to
    /// square corners and full opacity, exactly as a node's do.
    pub(crate) fn from_runtime_projection(
        node: &UiMountedScrollChromeNode,
        surface: UiSemanticSurfaceIdentity,
        projection: &UiScrollChromeAppearanceProjection,
    ) -> Result<Self, UiMountedAppearanceLoweringDenial> {
        let rect = super::geometry::allocation(node.rect())
            .map_err(UiMountedAppearanceLoweringDenial::Geometry)?;
        let clip_bounds = super::geometry::allocation(node.clip())
            .map_err(UiMountedAppearanceLoweringDenial::Geometry)?;
        let clip = UiAppearanceClip::new(
            clip_bounds.x(),
            clip_bounds.y(),
            clip_bounds.width(),
            clip_bounds.height(),
        )
        .map_err(|_| UiMountedAppearanceLoweringDenial::WorkConstruction)?;
        let mut background = None;
        let mut radii = worth_ui_host_contract::UiAppearanceNormalizedLogicalRadii::normalize(
            rect,
            [worth_ui_host_contract::UiAppearanceLogicalLength::ZERO; 4],
        );
        let mut appearance_opacity = UiMountedAppearanceOpacity::ONE;
        for aspect in projection.aspects() {
            if aspect.support() != UiAppearanceSupportPosture::Supported {
                continue;
            }
            match (aspect.aspect(), aspect.value()) {
                (
                    worth_ui_dsl::UiAppearanceAspect::Background,
                    worth_ui_dsl::UiThemeValue::Color(color),
                ) => background = Some(super::style::mounted_color(color)),
                (
                    worth_ui_dsl::UiAppearanceAspect::Radius,
                    worth_ui_dsl::UiThemeValue::CornerRadii(authored),
                ) => radii = super::style::resolved_radii(rect, authored)?,
                (
                    worth_ui_dsl::UiAppearanceAspect::Opacity,
                    worth_ui_dsl::UiThemeValue::Opacity(opacity),
                ) => appearance_opacity = UiMountedAppearanceOpacity::from_units(opacity.units()),
                _ => {}
            }
        }
        let background =
            background.ok_or(UiMountedAppearanceLoweringDenial::ScrollChromeBackgroundMissing)?;
        let owner = node.owner_instance();
        Ok(Self {
            identity: UiMountedScrollChromeIdentity::from_runtime_mounting(
                owner,
                host_axis(node.axis()),
                host_part(node.part()),
            ),
            semantic_surface: surface,
            rect,
            clip,
            background,
            radii,
            appearance_opacity,
            motion_opacity: None,
            attribution: UiMountedScrollChromeAppearanceAttribution::from_runtime_transport(
                surface,
                owner,
                projection.role_digest(),
                projection.semantic_digest(),
            )
            .ok_or(UiMountedAppearanceLoweringDenial::ScrollChromeAttributionUnavailable)?,
            semantic_digest: projection.semantic_digest(),
        })
    }

    pub(in crate::mounting::projection) const fn owner_instance(
        &self,
    ) -> UiMountedInstanceIdentity {
        self.identity.owner_instance()
    }
    pub(in crate::mounting::projection) const fn semantic_surface(
        &self,
    ) -> UiSemanticSurfaceIdentity {
        self.semantic_surface
    }
    pub(super) const fn rect(&self) -> UiAppearanceAllocationBounds {
        self.rect
    }
    pub(super) const fn clip(&self) -> UiAppearanceClip {
        self.clip
    }
    pub(super) const fn attribution(&self) -> UiMountedScrollChromeAppearanceAttribution {
        self.attribution
    }
    pub(super) const fn semantic_digest(&self) -> u64 {
        self.semantic_digest
    }
}

const fn host_axis(axis: UiScrollChromeAxis) -> UiMountedScrollChromeAxis {
    match axis {
        UiScrollChromeAxis::Inline => UiMountedScrollChromeAxis::Inline,
        UiScrollChromeAxis::Block => UiMountedScrollChromeAxis::Block,
    }
}

const fn host_part(part: UiScrollChromePart) -> UiMountedScrollChromePart {
    match part {
        UiScrollChromePart::Track => UiMountedScrollChromePart::Track,
        UiScrollChromePart::Thumb => UiMountedScrollChromePart::Thumb,
    }
}

pub(super) fn lower(
    input: &UiMountedAppearanceScrollChromeInput,
) -> Result<UiMountedAppearanceMechanic, UiMountedAppearanceLoweringDenial> {
    UiMountedScrollChromeMechanic::complete_from_runtime_mounting(
        UiMountedScrollChromeCompletionInput {
            identity: input.identity,
            semantic_surface: input.semantic_surface,
            rect: input.rect,
            clip: input.clip,
            background: input.background,
            radii: input.radii,
            opacity: compose_opacity(
                input.appearance_opacity,
                input.motion_opacity.unwrap_or(u16::MAX),
            ),
            attribution: input.attribution,
        },
    )
    .map(UiMountedAppearanceMechanic::ScrollChrome)
    .map_err(UiMountedAppearanceLoweringDenial::ScrollChrome)
}

impl super::UiMountedAppearanceLoweringInput {
    /// The whole fragment of a Scroll region occurrence that paints nothing of
    /// its own: no node, only the chrome it owns. The occurrence still lends
    /// the fragment its node receipt, so the host attributes the bars to it.
    pub(crate) fn for_scroll_chrome_owner(
        frame: worth_ui_host_contract::UiMountedFrameIdentity,
        semantic_surface: UiSemanticSurfaceIdentity,
        presentation: worth_ui_host_contract::UiMountedPresentationAttemptIdentity,
        owner: UiMountedInstanceIdentity,
        chrome: Vec<UiMountedAppearanceScrollChromeInput>,
    ) -> Self {
        let mut input = Self::empty(frame, semantic_surface, presentation);
        input.chrome_owner = Some(owner);
        input.scroll_chrome = chrome;
        input
    }
}
