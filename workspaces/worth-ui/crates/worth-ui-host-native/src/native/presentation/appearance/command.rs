use worth_ui_host_contract::{
    UiMountedBackdropMechanic, UiMountedOutlineAppearanceMechanic, UiMountedOverlayOrderMechanic,
    UiMountedPointerAffordanceMechanic, UiMountedPortalSurfaceAppearanceMechanic,
    UiMountedSurfaceAppearanceMechanic,
};

use super::backdrop_pipeline::UiNativeBackdropPipeline;
use super::damage::UiNativeAppearanceDamageRect;
use super::geometry::{UiNativeAppearanceScale, UiNativeGeometryDenial};
use super::outline_pipeline::UiNativeOutlinePipeline;
use super::surface_pipeline::UiNativeSurfacePipeline;

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub(crate) struct UiNativeAppearanceCommandKey(u32);

impl UiNativeAppearanceCommandKey {
    pub(crate) const fn new(value: u32) -> Self {
        Self(value)
    }

    pub(crate) const fn value(self) -> u32 {
        self.0
    }
}

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub(crate) enum UiNativeAppearanceCommandFamily {
    Surface,
    PortalSurface,
    Outline,
    TextForeground,
    Backdrop,
    OverlayOrder,
    PointerAffordance,
}

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub(crate) enum UiNativeAppearanceCommandIdentity {
    Surface(worth_ui_host_contract::UiMountedInstanceIdentity),
    PortalSurface(worth_ui_host_contract::UiMountedInstanceIdentity),
    Outline(worth_ui_host_contract::UiMountedInstanceIdentity),
    TextForeground {
        target: worth_ui_host_contract::UiMountedInstanceIdentity,
        command: (u16, Option<[u8; 32]>),
        span_digest: [u8; 32],
    },
    Backdrop {
        surface: worth_ui_host_contract::UiSemanticSurfaceIdentity,
        identity: worth_ui_host_contract::UiMountedBackdropIdentity,
    },
    OverlayOrder {
        surface: worth_ui_host_contract::UiSemanticSurfaceIdentity,
    },
    PointerAffordance {
        pointer: worth_ui_host_contract::UiHostPointerIdentity,
        surface: worth_ui_host_contract::UiSemanticSurfaceIdentity,
        target: worth_ui_host_contract::UiMountedInstanceIdentity,
    },
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) enum UiNativeAppearanceCommand {
    Surface(UiMountedSurfaceAppearanceMechanic),
    PortalSurface(UiMountedPortalSurfaceAppearanceMechanic),
    Outline(UiMountedOutlineAppearanceMechanic),
    TextForeground(super::text_foreground::UiNativeFinalizedTextForeground),
    Backdrop(UiMountedBackdropMechanic),
    OverlayOrder(UiMountedOverlayOrderMechanic),
    PointerAffordance(UiMountedPointerAffordanceMechanic),
}

impl UiNativeAppearanceCommand {
    pub(crate) fn family(&self) -> UiNativeAppearanceCommandFamily {
        match self {
            Self::Surface(_) => UiNativeAppearanceCommandFamily::Surface,
            Self::PortalSurface(_) => UiNativeAppearanceCommandFamily::PortalSurface,
            Self::Outline(_) => UiNativeAppearanceCommandFamily::Outline,
            Self::TextForeground(_) => UiNativeAppearanceCommandFamily::TextForeground,
            Self::Backdrop(_) => UiNativeAppearanceCommandFamily::Backdrop,
            Self::OverlayOrder(_) => UiNativeAppearanceCommandFamily::OverlayOrder,
            Self::PointerAffordance(_) => UiNativeAppearanceCommandFamily::PointerAffordance,
        }
    }

    pub(crate) fn identity(&self) -> UiNativeAppearanceCommandIdentity {
        match self {
            Self::Surface(mechanic) => UiNativeAppearanceCommandIdentity::Surface(
                mechanic.node_receipt().mounted_instance(),
            ),
            Self::PortalSurface(mechanic) => {
                UiNativeAppearanceCommandIdentity::PortalSurface(mechanic.portal_instance())
            }
            Self::Outline(mechanic) => UiNativeAppearanceCommandIdentity::Outline(
                mechanic.node_receipt().mounted_instance(),
            ),
            Self::TextForeground(mechanic) => UiNativeAppearanceCommandIdentity::TextForeground {
                target: mechanic.mechanic().node_receipt().mounted_instance(),
                command: mechanic
                    .mechanic()
                    .command()
                    .semantic_text_identity_parts()
                    .expect("text foreground admits one semantic text command"),
                span_digest: mechanic.mechanic().paint_span().digest(),
            },
            Self::Backdrop(mechanic) => UiNativeAppearanceCommandIdentity::Backdrop {
                surface: mechanic.semantic_surface(),
                identity: mechanic.identity().clone(),
            },
            Self::OverlayOrder(mechanic) => UiNativeAppearanceCommandIdentity::OverlayOrder {
                surface: mechanic.semantic_surface(),
            },
            Self::PointerAffordance(mechanic) => {
                UiNativeAppearanceCommandIdentity::PointerAffordance {
                    pointer: mechanic.pointer(),
                    surface: mechanic.surface(),
                    target: mechanic.target(),
                }
            }
        }
    }

    pub(crate) fn damage_rect(
        &self,
        scale: UiNativeAppearanceScale,
    ) -> Result<Option<UiNativeAppearanceDamageRect>, UiNativeGeometryDenial> {
        match self {
            Self::Surface(mechanic) => Ok(Some(
                UiNativeSurfacePipeline::prepare(mechanic, scale)?.damage_rect(),
            )),
            Self::PortalSurface(mechanic) => Ok(Some(
                UiNativeSurfacePipeline::prepare(mechanic.surface(), scale)?.damage_rect(),
            )),
            Self::Outline(mechanic) => Ok(Some(
                UiNativeOutlinePipeline::prepare(mechanic, scale)?.damage_rect(),
            )),
            Self::Backdrop(mechanic) => Ok(Some(
                UiNativeBackdropPipeline::prepare(mechanic, scale)?.damage_rect(),
            )),
            Self::TextForeground(text) => text.damage_bounds(scale),
            Self::OverlayOrder(_) | Self::PointerAffordance(_) => Ok(None),
        }
    }

    pub(crate) fn text_coverage(&self) -> Option<&[UiNativeAppearanceDamageRect]> {
        match self {
            Self::TextForeground(text) => Some(text.coverage()),
            _ => None,
        }
    }

    pub(crate) fn is_drawable(&self) -> bool {
        matches!(
            self,
            Self::Surface(_) | Self::PortalSurface(_) | Self::Outline(_) | Self::Backdrop(_)
        )
    }
}
