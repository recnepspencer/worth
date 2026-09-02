// Translation is intentionally dormant until the unpublished surface is adopted.

mod backdrop;
mod outline;
mod overlay_order;
mod pointer_affordance;
mod surface;
mod text_foreground;
mod work;

use worth_ui_host_contract::{UiMountedAppearanceMechanic, UiMountedAppearanceWork};

use super::super::headless_transcript::appearance::reference_raster;
use super::super::headless_transcript::appearance::UiHeadlessAppearanceWorkTranscript;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum UiHeadlessAppearanceTranslationDenial {
    InvalidMechanic,
    StructuralMismatch,
}

pub(crate) fn translate(
    source: &UiMountedAppearanceWork,
) -> Result<UiHeadlessAppearanceWorkTranscript, UiHeadlessAppearanceTranslationDenial> {
    if !overlay_order::validate(
        source.successor().semantic_surface(),
        source.successor().overlay_order(),
    ) || !work::validate_changes(source)
        || !reference_raster::has_issued_participants(source.successor())
    {
        return Err(UiHeadlessAppearanceTranslationDenial::InvalidMechanic);
    }
    if source
        .successor()
        .mechanics()
        .iter()
        .any(|mechanic| !validate_mechanic(mechanic, source.successor().semantic_surface()))
    {
        return Err(UiHeadlessAppearanceTranslationDenial::InvalidMechanic);
    }
    let reference_source_over = reference_raster::compose(source.successor())
        .ok_or(UiHeadlessAppearanceTranslationDenial::StructuralMismatch)?;
    let transcript = UiHeadlessAppearanceWorkTranscript::from_mounted(source)
        .ok_or(UiHeadlessAppearanceTranslationDenial::StructuralMismatch)?;
    let mechanics_match = transcript
        .successor()
        .mechanics()
        .iter()
        .zip(source.successor().mechanics())
        .all(|(transcript, source)| transcript.matches_mounted(source));
    let changes_match =
        transcript
            .changes()
            .iter()
            .zip(source.changes())
            .all(|(transcript, source)| {
                super::super::headless_transcript::appearance::work::matches_mounted(
                    transcript, source,
                )
            });
    if transcript.successor().mechanics().len() != source.successor().mechanics().len()
        || transcript.changes().len() != source.changes().len()
        || !mechanics_match
        || !changes_match
        || transcript.damage() != source.damage()
        || transcript.posture() != source.posture()
        || transcript.predecessor() != source.predecessor()
        || transcript.predecessor_manifest() != source.predecessor_manifest()
        || transcript.successor().frame() != source.successor().frame()
        || transcript.successor().semantic_surface() != source.successor().semantic_surface()
        || transcript.successor().overlay_order() != source.successor().overlay_order()
        || transcript.order_changed() != source.order_changed()
        || transcript.successor().reference_source_over() != reference_source_over
    {
        return Err(UiHeadlessAppearanceTranslationDenial::StructuralMismatch);
    }
    Ok(transcript)
}

fn validate_mechanic(
    mechanic: &UiMountedAppearanceMechanic,
    semantic_surface: worth_ui_host_contract::UiSemanticSurfaceIdentity,
) -> bool {
    match mechanic {
        UiMountedAppearanceMechanic::Surface(mechanic) => surface::validate(mechanic),
        UiMountedAppearanceMechanic::PortalSurface(mechanic) => surface::validate_portal(mechanic),
        UiMountedAppearanceMechanic::Outline(mechanic) => outline::validate(mechanic),
        UiMountedAppearanceMechanic::TextForeground(mechanic) => {
            text_foreground::validate(mechanic)
        }
        UiMountedAppearanceMechanic::Pointer(mechanic) => {
            mechanic.surface() == semantic_surface && pointer_affordance::validate(mechanic)
        }
        UiMountedAppearanceMechanic::Backdrop(mechanic) => {
            mechanic.semantic_surface() == semantic_surface && backdrop::validate(mechanic)
        }
    }
}

#[cfg(test)]
mod tests;
