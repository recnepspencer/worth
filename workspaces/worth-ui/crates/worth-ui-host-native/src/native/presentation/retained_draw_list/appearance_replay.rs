//! Finalized appearance damage joins the ordinary physical replay owner.
use super::{UiNativeRetainedDrawList, UiNativeRetainedDrawListDenial as Denial};
use crate::native::presentation::{
    appearance::{UiNativeAppearanceCommand, UiNativeAppearanceReplayRegion},
    raster::{raster_physical_bounds, UiNativeRasterBasis},
    RasterRect,
};
use worth_ui_host_contract::UiMountedPaintCommand;

impl UiNativeRetainedDrawList {
    pub(in crate::native::presentation) fn appearance_only_replay_plan(
        &mut self,
    ) -> Result<super::UiNativeRetainedReplayPlan, Denial> {
        let mut replay = self.replay_plan(&[], 0, 0)?;
        replay.staged_appearance_regions = self.prepare_appearance_replay()?;
        Ok(replay)
    }

    pub(in crate::native::presentation) fn staged_appearance_clears(
        &self,
        regions: &[UiNativeAppearanceReplayRegion],
        basis: UiNativeRasterBasis,
    ) -> Result<Vec<RasterRect>, Denial> {
        if regions.is_empty() {
            return Ok(Vec::new());
        }
        let (binding, appearance) = self
            .staged_appearance
            .as_ref()
            .ok_or(Denial::CommandMismatch)?;
        if binding.device_scale_milli() as f32 / 1_000.0 != basis.scale_factor() {
            return Err(Denial::AffinityMismatch);
        }
        let mut clears = Vec::with_capacity(regions.len());
        for region in regions {
            // Every surviving appearance participant must have its ordinary
            // command. Otherwise clearing could silently erase unrepresented text.
            for key in &region.replay {
                let Some(command) = appearance.command(*key) else {
                    return Err(Denial::CommandMismatch);
                };
                let UiNativeAppearanceCommand::TextForeground(foreground) = command else {
                    continue;
                };
                for identity in foreground.candidate_commands() {
                    let Some(UiMountedPaintCommand::SemanticText { mechanic, .. }) =
                        self.command(identity)
                    else {
                        return Err(Denial::CommandMismatch);
                    };
                    foreground
                        .validate_command(mechanic)
                        .map_err(|_| Denial::CommandMismatch)?;
                }
            }
            let damage = region.damage;
            if damage.is_empty() {
                return Err(Denial::CommandMismatch);
            }
            let [width, height] = basis.extent();
            let edges = [
                damage.left.clamp(0, i64::from(width)) as u32,
                damage.top.clamp(0, i64::from(height)) as u32,
                damage.right.clamp(0, i64::from(width)) as u32,
                damage.bottom.clamp(0, i64::from(height)) as u32,
            ];
            if let Some(clear) = raster_physical_bounds(edges, basis.extent()) {
                clears.push(clear);
            }
        }
        Ok(clears)
    }
}
