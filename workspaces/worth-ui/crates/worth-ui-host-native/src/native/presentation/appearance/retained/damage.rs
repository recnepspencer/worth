//! Record exact historical image damage for a retained command mutation.
use super::super::damage::{
    UiNativeAppearanceDamage, UiNativeAppearanceDamageRect, UiNativeAppearanceDamageSetDenial,
};
use super::{UiNativeAppearanceCommand, UiNativeAppearanceRetainedDenial, UiNativeGeometryDenial};

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
