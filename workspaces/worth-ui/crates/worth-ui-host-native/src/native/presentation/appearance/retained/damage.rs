//! Record exact historical image damage for a retained command mutation.
use super::super::damage::{
    UiNativeAppearanceDamage, UiNativeAppearanceDamageRect, UiNativeAppearanceDamageSetDenial,
};
use super::{UiNativeAppearanceCommand, UiNativeAppearanceRetainedDenial, UiNativeGeometryDenial};
use crate::native::presentation::damage_index::UiNativeDamageIndexDenial;
use std::collections::{BTreeMap, BTreeSet};
use worth_ui_host_contract::{UiMountedOverlayOrderMechanic, UiOverlayParticipantIdentity};

pub(super) fn record_damage(
    command: &UiNativeAppearanceCommand,
    bounds: Option<UiNativeAppearanceDamageRect>,
    pending: &mut UiNativeAppearanceDamage,
) -> Result<(), UiNativeAppearanceRetainedDenial> {
    if let Some(regions) = command.text_coverage() {
        for &region in regions {
            pending.add(region).map_err(map_damage_denial)?;
        }
    } else if let Some(bounds) = bounds {
        pending.add(bounds).map_err(map_damage_denial)?;
    }
    Ok(())
}

fn map_damage_denial(
    denial: UiNativeAppearanceDamageSetDenial,
) -> UiNativeAppearanceRetainedDenial {
    match denial {
        UiNativeAppearanceDamageSetDenial::CapacityExceeded => {
            UiNativeAppearanceRetainedDenial::DamageCapacityExceeded
        }
        UiNativeAppearanceDamageSetDenial::Empty => {
            UiNativeAppearanceRetainedDenial::Geometry(UiNativeGeometryDenial::CoordinateOverflow)
        }
    }
}

pub(super) fn map_index_denial(
    denial: UiNativeDamageIndexDenial,
) -> UiNativeAppearanceRetainedDenial {
    match denial {
        UiNativeDamageIndexDenial::CapacityExceeded => {
            UiNativeAppearanceRetainedDenial::DamageIndexCapacityExceeded
        }
        UiNativeDamageIndexDenial::DuplicateIdentity => {
            UiNativeAppearanceRetainedDenial::DuplicateIdentity
        }
        UiNativeDamageIndexDenial::MissingIdentity => {
            UiNativeAppearanceRetainedDenial::MissingIdentity
        }
    }
}

pub(super) fn record_overlay_order_damage(
    previous: &UiMountedOverlayOrderMechanic,
    successor: &UiMountedOverlayOrderMechanic,
    identities: &BTreeMap<
        super::UiNativeAppearanceCommandIdentity,
        super::UiNativeAppearanceCommandKey,
    >,
    bounds: &BTreeMap<super::UiNativeAppearanceCommandKey, UiNativeAppearanceDamageRect>,
    pending: &mut UiNativeAppearanceDamage,
) -> Result<(), UiNativeAppearanceRetainedDenial> {
    if previous.semantic_surface() != successor.semantic_surface() {
        return Err(UiNativeAppearanceRetainedDenial::OverlayOrderMismatch);
    }
    if previous.bottom_to_top() == successor.bottom_to_top() {
        return Ok(());
    }
    let participants = previous
        .bottom_to_top()
        .iter()
        .chain(successor.bottom_to_top())
        .cloned()
        .collect::<BTreeSet<_>>();
    for participant in participants {
        let identity = match participant {
            UiOverlayParticipantIdentity::Portal(instance) => {
                super::UiNativeAppearanceCommandIdentity::PortalSurface(instance)
            }
            UiOverlayParticipantIdentity::Backdrop(identity) => {
                super::UiNativeAppearanceCommandIdentity::Backdrop {
                    surface: successor.semantic_surface(),
                    identity,
                }
            }
        };
        if let Some(damage) = identities.get(&identity).and_then(|key| bounds.get(key)) {
            pending.add(*damage).map_err(map_damage_denial)?;
        }
    }
    Ok(())
}
