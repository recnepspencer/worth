use std::collections::HashMap;

use worth_ui_host_contract::{
    UiMountedLogicalDamage, UiMountedPaintCommand, UiMountedPaintCommandIdentity,
    UiMountedPaintOrderIdentity,
};

use super::damage_index::{UiNativeDamageIndex, UiNativeDamageIndexDenial};
use super::retained_order::{UiNativeRetainedOrder, UiNativeRetainedOrderDenial};

#[path = "retained_draw_list/command_store.rs"]
mod command_store;
#[path = "retained_draw_list/complete.rs"]
mod complete;
#[path = "retained_draw_list/delta_transaction.rs"]
mod delta_transaction;
#[path = "retained_draw_list/denial.rs"]
mod denial;
#[cfg(feature = "certification-support")]
mod foreground_replay_certification;
#[path = "retained_draw_list/lifecycle.rs"]
mod lifecycle;
#[path = "retained_draw_list/mutation.rs"]
mod mutation;
#[path = "retained_draw_list/physical_coverage.rs"]
mod physical_coverage;
#[path = "retained_draw_list/physical_replay.rs"]
mod physical_replay;
#[cfg(feature = "certification-support")]
#[path = "retained_draw_list/physical_replay_certification.rs"]
mod physical_replay_certification;
#[cfg(feature = "certification-support")]
mod text_transition_certification;
#[cfg(feature = "certification-support")]
pub use foreground_replay_certification::UiNativeTextReplayOperation;
#[path = "retained_draw_list/appearance_delta.rs"]
mod appearance_delta;
#[path = "retained_draw_list/appearance_raster.rs"]
mod appearance_raster;
mod appearance_replay;
#[path = "retained_draw_list/appearance_state.rs"]
mod appearance_state;
#[path = "retained_draw_list/raster_command.rs"]
mod raster_command;
#[path = "retained_draw_list/render_order.rs"]
mod render_order;
#[path = "retained_draw_list/replay.rs"]
mod replay;
#[path = "retained_draw_list/sample_transaction.rs"]
mod sample_transaction;
#[path = "retained_draw_list/text_coverage.rs"]
mod text_coverage;
#[cfg(feature = "certification-support")]
#[path = "retained_draw_list/text_coverage_certification.rs"]
mod text_coverage_certification;
mod text_raster;

pub(super) use delta_transaction::UiNativeRetainedDeltaUndo;
pub(super) use denial::UiNativeRetainedDrawListDenial;
pub(super) use lifecycle::UiNativeRetainedUnchangedUndo;
pub(super) use sample_transaction::sampled_visible_bounds;
pub(super) use sample_transaction::UiNativeRetainedSampleUndo;

pub(crate) struct UiNativeRetainedDrawList {
    physical_coverage: Option<physical_coverage::UiNativePhysicalCoverage>,
    staged_appearance: Option<(
        worth_ui_host_contract::UiMountedSurfaceBindingRequirement,
        super::appearance::UiNativeAppearanceRetained,
    )>,
    frame: worth_ui_host_contract::UiMountedFrameIdentity,
    surface: worth_ui_host_contract::UiSemanticSurfaceIdentity,
    binding: worth_ui_host_contract::UiSurfaceBindingGeneration,
    content: worth_ui_host_contract::UiMountedContentGeneration,
    baseline: worth_ui_host_contract::UiHostSurfaceBaselineIdentity,
    commands: command_store::UiNativeRetainedCommandStore,
    order: UiNativeRetainedOrder<UiMountedPaintOrderIdentity>,
    order_integrity: worth_ui_host_contract::UiMountedPaintOrderIntegrity,
    damage: UiNativeDamageIndex<UiMountedPaintCommandIdentity>,
    glyph_runs:
        HashMap<UiMountedPaintCommandIdentity, Box<[worth_ui_host_contract::UiGlyphRunView]>>,
    sample_overrides: HashMap<
        UiMountedPaintCommandIdentity,
        worth_ui_host_contract::UiMountedPresentationSampleChange,
    >,
    regions: super::retained_regions::UiNativeRetainedRegions,
    identity_overlay: super::identity_overlay::UiNativeRetainedIdentityOverlay,
    last_paint_attribution: Option<(usize, UiNativeRetainedPresentationAttribution)>,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub(super) struct UiNativeRetainedPresentationAttribution {
    pub(super) color: worth_ui_host_contract::UiMountedRgba8,
    pub(super) bounds: worth_ui_host_contract::UiMountedCanonicalBox,
    pub(super) mounted_instance: worth_ui_host_contract::UiMountedInstanceIdentity,
    pub(super) node_receipt: worth_ui_host_contract::UiMountedNodeReceiptIdentity,
}

#[derive(Debug, PartialEq)]
pub(crate) struct UiNativeRetainedReplayPlan {
    pub(super) baseline_rgba8: [u8; 4],
    pub(super) regions: Box<[UiNativeRetainedReplayRegion]>,
    pub(super) physical_text_regions: Vec<super::RasterRect>,
    pub(super) staged_appearance_regions: Box<[super::appearance::UiNativeAppearanceReplayRegion]>,
    pub(super) counters: UiNativeRetainedMutationCounters,
    pub(super) identity_overlay_effect: bool,
}

#[derive(Debug, PartialEq)]
pub(super) struct UiNativeRetainedReplayRegion {
    pub(super) damage: UiMountedLogicalDamage,
    pub(super) replay: Box<[UiMountedPaintCommandIdentity]>,
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub(super) struct UiNativeRetainedMutationCounters {
    pub(super) draw_mutations: u64,
    pub(super) order_mutations: u64,
    pub(super) order_index_lookups: u64,
    pub(super) order_index_node_touches: u64,
    pub(super) order_index_rotations: u64,
    pub(super) order_index_high_water: u64,
    pub(super) damage_rows_carried: u64,
    pub(super) damage_regions: u64,
    pub(super) damage_index_branch_aabb_probes: u64,
    pub(super) damage_index_leaf_command_bounds_probes: u64,
    pub(super) damage_index_stored_records: u64,
    pub(super) damage_index_high_water: u64,
    pub(super) damage_region_command_checks: u64,
    pub(super) replayed_commands: u64,
    pub(super) retained_command_scans: u64,
}

impl UiNativeRetainedDrawList {
    pub(crate) const fn frame(&self) -> worth_ui_host_contract::UiMountedFrameIdentity {
        self.frame
    }

    pub(crate) fn owns_node_receipt(
        &self,
        receipt: worth_ui_host_contract::UiMountedNodeReceiptIdentity,
    ) -> bool {
        self.regions.owns_node_receipt(receipt)
    }

    #[cfg(test)]
    pub(super) fn apply_delta(
        &mut self,
        delta: &worth_ui_host_contract::UiMountedPresentationDelta,
    ) -> Result<UiNativeRetainedReplayPlan, UiNativeRetainedDrawListDenial> {
        let (plan, _) = self.stage_delta(delta, &[])?;
        Ok(plan)
    }

    pub(super) fn command(
        &self,
        identity: UiMountedPaintCommandIdentity,
    ) -> Option<&UiMountedPaintCommand> {
        self.commands.get(&identity)
    }

    pub(super) fn glyph_runs(
        &self,
        identity: UiMountedPaintCommandIdentity,
    ) -> &[worth_ui_host_contract::UiGlyphRunView] {
        self.glyph_runs
            .get(&identity)
            .map(Box::as_ref)
            .unwrap_or_default()
    }

    pub(super) fn sample_override(
        &self,
        identity: UiMountedPaintCommandIdentity,
    ) -> Option<worth_ui_host_contract::UiMountedPresentationSampleChange> {
        self.sample_overrides.get(&identity).copied()
    }

    pub(super) fn all_glyph_runs(&self) -> Vec<worth_ui_host_contract::UiGlyphRunView> {
        self.glyph_runs
            .values()
            .flat_map(|runs| runs.iter().copied())
            .collect()
    }

    pub(crate) fn realized_regions(
        &self,
    ) -> Option<Vec<worth_ui_host_contract::UiHostRealizedRegion>> {
        self.regions.realized(self.order.ordered())
    }

    pub(super) fn identity_overlay_operations(
        &self,
        basis: super::raster::UiNativeRasterBasis,
    ) -> Result<
        Vec<super::UiNativeRasterOperation>,
        worth_ui_host_contract::UiHostSurfacePresentationDenial,
    > {
        self.identity_overlay.raster_operations(basis)
    }

    pub(crate) const fn identity_overlay_active(&self) -> bool {
        self.identity_overlay.is_active()
    }

    pub(super) fn top_paint_attribution(
        &self,
    ) -> Option<(usize, UiNativeRetainedPresentationAttribution)> {
        self.current_top_paint_attribution()
            .or(self.last_paint_attribution)
            .map(|(ordinal, mut attribution)| {
                attribution.node_receipt = self.regions.current_receipt(attribution.node_receipt);
                (ordinal, attribution)
            })
    }

    fn current_top_paint_attribution(
        &self,
    ) -> Option<(usize, UiNativeRetainedPresentationAttribution)> {
        let (ordinal, identity) = self.order.ordered().enumerate().last()?;
        let attribution = match self.commands.get(&identity.command())? {
            UiMountedPaintCommand::FilledRect { mechanic, .. } => {
                UiNativeRetainedPresentationAttribution {
                    color: mechanic.color(),
                    bounds: mechanic.bounds(),
                    mounted_instance: mechanic.mounted_instance(),
                    node_receipt: mechanic.node_receipt(),
                }
            }
            UiMountedPaintCommand::PortalOverlay { mechanic, .. } => {
                UiNativeRetainedPresentationAttribution {
                    color: mechanic.color(),
                    bounds: mechanic.bounds(),
                    mounted_instance: mechanic.owner(),
                    node_receipt: mechanic.owner_receipt(),
                }
            }
            UiMountedPaintCommand::SemanticText { mechanic, .. } => {
                UiNativeRetainedPresentationAttribution {
                    color: mechanic.foregrounds().first()?.color(),
                    bounds: mechanic.bounds(),
                    mounted_instance: mechanic.mounted_instance(),
                    node_receipt: mechanic.node_receipt(),
                }
            }
        };
        Some((ordinal, attribution))
    }

    fn retain_current_paint_attribution(&mut self) {
        if let Some(attribution) = self.current_top_paint_attribution() {
            self.last_paint_attribution = Some(attribution);
        }
    }
}

#[cfg(test)]
#[path = "retained_draw_list_tests.rs"]
pub(super) mod tests;
