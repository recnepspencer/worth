//! Complete glyph coverage for staged foreground candidates, using the ordinary
//! text-owned demand/attribution producer. This does not replace a surface pin set.

use super::{
    inspect_demand_boundary, prepare_demands, MountedSemanticTextWork, MountedTextDemandJoin,
    UiMountedEventTimeDpiAuthority, UiNativeTextPresentationPreparation,
    UiNativeTextPresentationReadiness,
};
use worth_ui_host_contract::{UiMountedPaintCommandIdentity, UiMountedSemanticTextMechanic};

pub(crate) fn prepare_complete_semantic_text<'work>(
    candidates: &'work [UiMountedSemanticTextMechanic],
    dpi: UiMountedEventTimeDpiAuthority,
    lane: worth_ui_host_contract::UiGlyphRasterLane,
    resolve: impl Fn(
        worth_ui_host_contract::UiQualifiedTextLayoutIdentity,
    ) -> Option<&'work worth_ui_text::UiQualifiedTextLayout>,
) -> Result<UiNativeTextPresentationPreparation, UiNativeTextPresentationReadiness> {
    let work = MountedSemanticTextWork {
        mechanics: candidates
            .iter()
            .map(|candidate| {
                (
                    UiMountedPaintCommandIdentity::semantic_text(candidate),
                    candidate,
                )
            })
            .collect(),
        removals: Vec::new(),
        complete: false,
    };
    let join = MountedTextDemandJoin {
        dpi,
        lane,
        selection: worth_ui_text::UiGlyphRasterDemandSelection::CompleteLayout,
        resolve,
        _layout: std::marker::PhantomData,
    };
    let demands = prepare_demands(&work.mechanics, &join)?;
    Ok(inspect_demand_boundary(&work, demands))
}
