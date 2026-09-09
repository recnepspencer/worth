use std::collections::BTreeMap;

use worth_ui_host_contract::{
    UiAppearanceBackdropExtent, UiAppearanceClip, UiMountedAppearanceColor,
    UiMountedAppearanceOpacity, UiMountedBackdropAppearanceAttribution, UiMountedBackdropIdentity,
    UiMountedBackdropScope, UiOverlayParticipantIdentity, UiOverlayPlacementReceipt,
};

use super::fact::{UiMountedAppearanceBackdropInput, UiMountedAppearanceSurfaceOverlayInput};
use super::UiMountedAppearanceLoweringDenial;
use crate::runtime::appearance::{
    UiAppearanceSupportPosture, UiBackdropAppearanceProjection, UiOverlayStackSnapshot,
};
use crate::runtime::overlay_composition::UiOverlayStackParticipant;

impl UiMountedAppearanceSurfaceOverlayInput {
    pub(crate) fn from_runtime_projections(
        snapshot: &UiOverlayStackSnapshot,
        projections: impl IntoIterator<Item = UiBackdropAppearanceProjection>,
    ) -> Result<Self, UiMountedAppearanceLoweringDenial> {
        let projections = projections
            .into_iter()
            .map(|projection| (projection.instance(), projection))
            .collect::<BTreeMap<_, _>>();
        let mut portals = Vec::new();
        let mut backdrops = Vec::new();
        let mut order = Vec::with_capacity(snapshot.participants().len());
        for (ordinal, participant) in snapshot.participants().iter().enumerate() {
            match participant {
                UiOverlayStackParticipant::Portal(row) => {
                    let instance = row.portal().owner().mounted_instance_identity();
                    portals.push(instance);
                    order.push(UiOverlayParticipantIdentity::Portal(instance));
                }
                UiOverlayStackParticipant::Backdrop(row) => {
                    let projection = projections
                        .get(&row.identity())
                        .ok_or(UiMountedAppearanceLoweringDenial::NodeProjectionUnavailable)?;
                    let placement = UiOverlayPlacementReceipt::from_runtime_overlay_order(
                        snapshot.portal_revision(),
                        u32::try_from(ordinal)
                            .map_err(|_| UiMountedAppearanceLoweringDenial::WorkConstruction)?,
                    )
                    .ok_or(UiMountedAppearanceLoweringDenial::OverlayRevisionMissing)?;
                    let identity = mounted_identity(snapshot, row)?;
                    order.push(UiOverlayParticipantIdentity::Backdrop(identity.clone()));
                    backdrops.push(backdrop_input(
                        snapshot, row, projection, identity, placement,
                    )?);
                }
            }
        }
        portals.sort();
        portals.dedup();
        Ok(Self {
            semantic_surface: snapshot.runtime_surface(),
            portal_revision: snapshot.portal_revision(),
            backdrop_revision: snapshot.backdrop_declaration_revision(),
            portal_instances: portals.into_boxed_slice(),
            backdrops: backdrops.into_boxed_slice(),
            bottom_to_top: order.into_boxed_slice(),
        })
    }
}

fn mounted_identity(
    snapshot: &UiOverlayStackSnapshot,
    row: &crate::runtime::overlay_composition::UiOverlayBackdropRow,
) -> Result<UiMountedBackdropIdentity, UiMountedAppearanceLoweringDenial> {
    let scope = match row.identity().scope() {
        crate::runtime::overlay_composition::UiOverlayBackdropInstanceScope::SurfaceSingleton => {
            UiMountedBackdropScope::SurfaceSingleton(snapshot.runtime_surface())
        }
        crate::runtime::overlay_composition::UiOverlayBackdropInstanceScope::Portal(portal) => {
            UiMountedBackdropScope::PerPortalInstance(portal.owner().mounted_instance_identity())
        }
    };
    UiMountedBackdropIdentity::from_runtime_mounting(
        format!("backdrop:{}", row.declaration().value()),
        scope,
        row.identity().semantic_digest().max(1),
    )
    .ok_or(UiMountedAppearanceLoweringDenial::WorkConstruction)
}

fn backdrop_input(
    snapshot: &UiOverlayStackSnapshot,
    row: &crate::runtime::overlay_composition::UiOverlayBackdropRow,
    projection: &UiBackdropAppearanceProjection,
    identity: UiMountedBackdropIdentity,
    placement: UiOverlayPlacementReceipt,
) -> Result<UiMountedAppearanceBackdropInput, UiMountedAppearanceLoweringDenial> {
    let bounds = super::geometry::allocation(row.extent().bounds())
        .map_err(UiMountedAppearanceLoweringDenial::Geometry)?;
    let extent =
        UiAppearanceBackdropExtent::new(bounds.x(), bounds.y(), bounds.width(), bounds.height())
            .map_err(|_| UiMountedAppearanceLoweringDenial::WorkConstruction)?;
    let clip = UiAppearanceClip::new(bounds.x(), bounds.y(), bounds.width(), bounds.height())
        .map_err(|_| UiMountedAppearanceLoweringDenial::WorkConstruction)?;
    let mut background = None;
    let mut opacity = UiMountedAppearanceOpacity::ONE;
    for aspect in projection.aspects() {
        if aspect.support() != UiAppearanceSupportPosture::Supported {
            continue;
        }
        match (aspect.aspect(), aspect.value()) {
            (
                worth_ui_dsl::UiAppearanceAspect::Background,
                worth_ui_dsl::UiThemeValue::Color(color),
            ) => {
                background = Some(UiMountedAppearanceColor::from_straight_srgba(
                    color.channels(),
                ));
            }
            (
                worth_ui_dsl::UiAppearanceAspect::Opacity,
                worth_ui_dsl::UiThemeValue::Opacity(value),
            ) => opacity = UiMountedAppearanceOpacity::from_units(value.units()),
            _ => {}
        }
    }
    let semantic_digest = projection.semantic_digest();
    let attribution = UiMountedBackdropAppearanceAttribution::from_runtime_transport(
        snapshot.runtime_surface(),
        placement,
        u64::from(row.declaration().value()).max(1),
        snapshot.backdrop_declaration_revision(),
    )
    .ok_or(UiMountedAppearanceLoweringDenial::WorkConstruction)?;
    Ok(UiMountedAppearanceBackdropInput {
        identity,
        semantic_surface: snapshot.runtime_surface(),
        placement,
        extent,
        clip,
        background: background
            .ok_or(UiMountedAppearanceLoweringDenial::NodeProjectionUnavailable)?,
        appearance_opacity: opacity,
        motion_opacity: None,
        motion_target: row
            .motion()
            .map(|motion| motion.portal().owner().mounted_instance_identity()),
        attribution,
        semantic_digest,
    })
}
