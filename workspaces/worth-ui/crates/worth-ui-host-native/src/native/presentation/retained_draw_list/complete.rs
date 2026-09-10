use std::collections::HashMap;

use worth_ui_host_contract::{
    UiMountedPaintCommand, UiMountedPaintOrderIdentity, UiMountedPaintOrderIntegrity,
    UiMountedPresentationInitial, UiMountedPresentationReconstruction,
};

use super::mutation::visible_bounds;
use super::{
    command_store::UiNativeRetainedCommandStore, UiNativeRetainedDrawList,
    UiNativeRetainedDrawListDenial,
};
use crate::native::presentation::damage_index::UiNativeDamageIndex;
use crate::native::presentation::retained_order::UiNativeRetainedOrder;

impl UiNativeRetainedDrawList {
    pub(in crate::native::presentation) fn initial(
        initial: &UiMountedPresentationInitial,
        glyph_runs: &[worth_ui_host_contract::UiGlyphRunView],
    ) -> Result<Self, UiNativeRetainedDrawListDenial> {
        if initial.affinity().baseline().transparent_rgba8() != [0, 0, 0, 0]
            || !initial.order_integrity().admits(initial.order())
        {
            return Err(UiNativeRetainedDrawListDenial::BaselineUnavailable);
        }
        Self::from_complete_with_projection(
            initial.affinity().successor(),
            initial.affinity().surface(),
            initial.affinity().binding(),
            initial.affinity().content(),
            initial.affinity().baseline(),
            initial.commands(),
            initial.order(),
            initial.order_integrity(),
            glyph_runs,
            &[],
            initial.projection(),
        )
    }

    pub(in crate::native::presentation) fn reconstruction(
        work: &UiMountedPresentationReconstruction,
        glyph_runs: &[worth_ui_host_contract::UiGlyphRunView],
    ) -> Result<Self, UiNativeRetainedDrawListDenial> {
        if work.affinity().predecessor().is_none()
            || work.affinity().baseline().transparent_rgba8() != [0, 0, 0, 0]
            || !work.order_integrity().admits(work.order())
        {
            return Err(UiNativeRetainedDrawListDenial::BaselineUnavailable);
        }
        Self::from_complete_with_projection(
            work.affinity().successor(),
            work.affinity().surface(),
            work.affinity().binding(),
            work.affinity().content(),
            work.affinity().baseline(),
            work.commands(),
            work.order(),
            work.order_integrity(),
            glyph_runs,
            work.sample_overrides(),
            work.projection(),
        )
    }

    #[cfg(any(test, feature = "certification-support"))]
    pub(in crate::native::presentation) fn from_complete(
        frame: worth_ui_host_contract::UiMountedFrameIdentity,
        surface: worth_ui_host_contract::UiSemanticSurfaceIdentity,
        binding: worth_ui_host_contract::UiSurfaceBindingGeneration,
        content: worth_ui_host_contract::UiMountedContentGeneration,
        baseline: worth_ui_host_contract::UiHostSurfaceBaselineIdentity,
        source_commands: &[UiMountedPaintCommand],
        source_order: &[UiMountedPaintOrderIdentity],
        order_integrity: UiMountedPaintOrderIntegrity,
        source_glyph_runs: &[worth_ui_host_contract::UiGlyphRunView],
    ) -> Result<Self, UiNativeRetainedDrawListDenial> {
        Self::from_complete_parts(
            frame,
            surface,
            binding,
            content,
            baseline,
            source_commands,
            source_order,
            order_integrity,
            source_glyph_runs,
            &[],
            super::super::retained_regions::UiNativeRetainedRegions::paint_only(source_commands),
            super::super::identity_overlay::UiNativeRetainedIdentityOverlay::default(),
        )
    }

    #[allow(clippy::too_many_arguments)]
    fn from_complete_with_projection(
        frame: worth_ui_host_contract::UiMountedFrameIdentity,
        surface: worth_ui_host_contract::UiSemanticSurfaceIdentity,
        binding: worth_ui_host_contract::UiSurfaceBindingGeneration,
        content: worth_ui_host_contract::UiMountedContentGeneration,
        baseline: worth_ui_host_contract::UiHostSurfaceBaselineIdentity,
        source_commands: &[UiMountedPaintCommand],
        source_order: &[UiMountedPaintOrderIdentity],
        order_integrity: UiMountedPaintOrderIntegrity,
        source_glyph_runs: &[worth_ui_host_contract::UiGlyphRunView],
        source_sample_overrides: &[worth_ui_host_contract::UiMountedPresentationSampleChange],
        projection: &worth_ui_host_contract::UiMountedProjectionView,
    ) -> Result<Self, UiNativeRetainedDrawListDenial> {
        let regions = super::super::retained_regions::UiNativeRetainedRegions::prepare(
            projection,
            source_commands,
        )
        .map_err(|_| UiNativeRetainedDrawListDenial::CommandMismatch)?;
        let identity_overlay =
            super::super::identity_overlay::UiNativeRetainedIdentityOverlay::prepare(projection)
                .map_err(|_| UiNativeRetainedDrawListDenial::CommandMismatch)?;
        Self::from_complete_parts(
            frame,
            surface,
            binding,
            content,
            baseline,
            source_commands,
            source_order,
            order_integrity,
            source_glyph_runs,
            source_sample_overrides,
            regions,
            identity_overlay,
        )
    }

    #[allow(clippy::too_many_arguments)]
    fn from_complete_parts(
        frame: worth_ui_host_contract::UiMountedFrameIdentity,
        surface: worth_ui_host_contract::UiSemanticSurfaceIdentity,
        binding: worth_ui_host_contract::UiSurfaceBindingGeneration,
        content: worth_ui_host_contract::UiMountedContentGeneration,
        baseline: worth_ui_host_contract::UiHostSurfaceBaselineIdentity,
        source_commands: &[UiMountedPaintCommand],
        source_order: &[UiMountedPaintOrderIdentity],
        order_integrity: UiMountedPaintOrderIntegrity,
        source_glyph_runs: &[worth_ui_host_contract::UiGlyphRunView],
        source_sample_overrides: &[worth_ui_host_contract::UiMountedPresentationSampleChange],
        regions: super::super::retained_regions::UiNativeRetainedRegions,
        identity_overlay: super::super::identity_overlay::UiNativeRetainedIdentityOverlay,
    ) -> Result<Self, UiNativeRetainedDrawListDenial> {
        let mut commands = UiNativeRetainedCommandStore::with_capacity(source_commands.len());
        let mut damage = UiNativeDamageIndex::new();
        for command in source_commands {
            if commands
                .insert(command.identity(), command.clone())
                .is_some()
            {
                return Err(UiNativeRetainedDrawListDenial::CommandMismatch);
            }
            if let Some(bounds) = visible_bounds(command) {
                damage.insert(command.identity(), bounds)?;
            }
        }
        if source_order.len() != commands.len()
            || source_order
                .iter()
                .any(|identity| !commands.contains(&identity.command()))
        {
            return Err(UiNativeRetainedDrawListDenial::OrderMismatch);
        }
        let semantic_identities = source_commands
            .iter()
            .filter_map(|command| match command {
                UiMountedPaintCommand::SemanticText { identity, .. } => Some(*identity),
                UiMountedPaintCommand::FilledRect { .. }
                | UiMountedPaintCommand::PortalOverlay { .. } => None,
            })
            .collect::<std::collections::HashSet<_>>();
        if source_glyph_runs
            .iter()
            .any(|run| !semantic_identities.contains(&run.mechanic()))
        {
            return Err(UiNativeRetainedDrawListDenial::CommandMismatch);
        }
        let glyph_runs = semantic_identities
            .into_iter()
            .map(|identity| {
                (
                    identity,
                    source_glyph_runs
                        .iter()
                        .copied()
                        .filter(|run| run.mechanic() == identity)
                        .collect::<Vec<_>>()
                        .into_boxed_slice(),
                )
            })
            .collect();
        let sample_overrides = source_sample_overrides
            .iter()
            .copied()
            .map(|sample| (sample.command(), sample))
            .collect::<HashMap<_, _>>();
        if sample_overrides.len() != source_sample_overrides.len()
            || sample_overrides
                .keys()
                .any(|identity| !commands.contains(identity))
        {
            return Err(UiNativeRetainedDrawListDenial::CommandMismatch);
        }
        let order = UiNativeRetainedOrder::initial_weighted(source_order.iter().map(|identity| {
            (
                *identity,
                commands
                    .get(&identity.command())
                    .expect("complete order membership was validated")
                    .layer_semantic_order(),
            )
        }))?;
        let mut retained = Self {
            physical_coverage: None,
            staged_appearance: None,
            frame,
            surface,
            binding,
            content,
            baseline,
            commands,
            order,
            order_integrity,
            damage,
            glyph_runs,
            sample_overrides,
            regions,
            identity_overlay,
            last_paint_attribution: None,
        };
        retained.retain_current_paint_attribution();
        Ok(retained)
    }
}
