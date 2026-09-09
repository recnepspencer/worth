//! Certification of real raster/atlas metadata/coverage with a simulated upload
//! outcome. This model neither submits GPU work nor accepts surface presentation.

use crate::native::presentation::appearance::text_foreground::{
    UiNativeFinalizedTextForeground, UiNativeTextForegroundJoin,
};
pub use crate::native::presentation::appearance::text_foreground::{
    UiNativeTextForegroundFinalizationDenial, UiNativeTextForegroundJoinCost,
};
use crate::native::text_atlas::{
    UiNativeTextAtlas, UiNativeTextAtlasCommitOutcome, UiNativeTextAtlasExternalOutcome,
};
use worth_ui_host_contract::*;

pub struct UiNativeTextForegroundAtlasModel {
    atlas: UiNativeTextAtlas,
    presentation: Option<crate::native::presentation::UiNativeRetainedDrawList>,
}

pub struct UiNativeTextForegroundCoverageCertification {
    finalized: UiNativeFinalizedTextForeground,
    cost: UiNativeTextForegroundJoinCost,
}

impl UiNativeTextForegroundCoverageCertification {
    /// Actual native vertex paint for the prepared images; no GPU submission.
    pub fn glyph_vertex_colors(&self, extent: [u32; 2]) -> Box<[[f32; 4]]> {
        self.finalized.glyph_vertex_colors(extent)
    }
    pub fn regions(&self) -> Box<[[i64; 4]]> {
        self.finalized
            .coverage()
            .iter()
            .map(|r| [r.left, r.top, r.right, r.bottom])
            .collect()
    }
    pub fn cost(&self) -> UiNativeTextForegroundJoinCost {
        self.cost
    }
}

pub use crate::native::presentation::UiNativeTextReplayOperation;

impl UiNativeTextForegroundAtlasModel {
    /// Initialize ordinary retained commands from real complete text/atlas admission.
    pub fn initialize_ordinary_presentation(
        &mut self,
        commands: &[UiMountedPaintCommand],
        order: &[UiMountedPaintOrderIdentity],
        view: &UiMountedFrameConsumptionView<'_>,
        extent: [u32; 2],
    ) -> Result<(), UiNativeTextForegroundFinalizationDenial> {
        if self.presentation.is_some() {
            return Err(UiNativeTextForegroundFinalizationDenial::PresentationCoverage);
        }
        let retained = crate::native::presentation::UiNativeRetainedDrawList::from_text_view_for_certification(
            commands, order, view, extent, &self.atlas,
        ).map_err(|_| UiNativeTextForegroundFinalizationDenial::PresentationCoverage)?;
        self.presentation = Some(retained);
        Ok(())
    }

    /// Simulate port refusal, verify rollback, retry, and retain the prepared successor.
    pub fn certify_ordinary_delta_retry(
        &mut self,
        delta: &UiMountedPresentationDelta,
        runs: &[UiGlyphRunView],
    ) -> Result<Box<[UiNativeTextReplayOperation]>, UiNativeTextForegroundFinalizationDenial> {
        self.presentation
            .as_mut()
            .ok_or(UiNativeTextForegroundFinalizationDenial::PresentationCoverage)?
            .certify_text_delta_retry(delta, runs, &self.atlas)
            .map_err(|_| UiNativeTextForegroundFinalizationDenial::PresentationCoverage)
    }

    /// Plan a sample using retained images, roll back, and verify identical retry work.
    pub fn certify_ordinary_sample_retry(
        &mut self,
        sample: &UiMountedPresentationSample,
    ) -> Result<
        (
            Box<[UiNativeTextReplayOperation]>,
            UiHostPresentationCostReport,
        ),
        UiNativeTextForegroundFinalizationDenial,
    > {
        self.presentation
            .as_mut()
            .ok_or(UiNativeTextForegroundFinalizationDenial::PresentationCoverage)?
            .certify_text_sample_retry(sample, &self.atlas)
            .map_err(|_| UiNativeTextForegroundFinalizationDenial::PresentationCoverage)
    }

    /// Reconstruct from current complete commands and freshly validated adoption.
    /// This observes the production planner, not GPU or recovery acceptance.
    pub fn foreground_reconstruction_commands(
        &self,
        commands: &[UiMountedPaintCommand],
        order: &[UiMountedPaintOrderIdentity],
        view: &UiMountedFrameConsumptionView<'_>,
        extent: [u32; 2],
        candidate: &UiNativeTextForegroundCoverageCertification,
    ) -> Result<Box<[UiNativeTextReplayOperation]>, UiNativeTextForegroundFinalizationDenial> {
        crate::native::presentation::UiNativeRetainedDrawList::foreground_reconstruction_for_certification(
            commands, order, view, extent, &candidate.finalized, &self.atlas,
        ).map_err(|_| UiNativeTextForegroundFinalizationDenial::PresentationCoverage)
    }

    /// Actual ordered raster operations, retaining immutable glyph attribution.
    pub fn foreground_replay_commands(
        &self,
        commands: &[UiMountedPaintCommand],
        order: &[UiMountedPaintOrderIdentity],
        view: &UiMountedFrameConsumptionView<'_>,
        extent: [u32; 2],
        candidate: &UiNativeTextForegroundCoverageCertification,
    ) -> Result<Box<[UiNativeTextReplayOperation]>, UiNativeTextForegroundFinalizationDenial> {
        crate::native::presentation::UiNativeRetainedDrawList::foreground_replay_for_certification(
            commands,
            order,
            view,
            extent,
            &candidate.finalized,
            &self.atlas,
        )
        .map_err(|_| UiNativeTextForegroundFinalizationDenial::PresentationCoverage)
    }

    /// Selected ordinary identities and actual draw order; `None` is a rectangle draw.
    pub fn physical_replay_commands(
        &self,
        commands: &[UiMountedPaintCommand],
        order: &[UiMountedPaintOrderIdentity],
        view: &UiMountedFrameConsumptionView<'_>,
        extent: [u32; 2],
        damage: UiMountedLogicalDamage,
    ) -> Result<
        (
            Box<[UiMountedPaintCommandIdentity]>,
            Box<[Option<UiMountedPaintCommandIdentity>]>,
        ),
        UiNativeTextForegroundFinalizationDenial,
    > {
        crate::native::presentation::UiNativeRetainedDrawList::physical_replay_for_certification(
            commands,
            order,
            view,
            extent,
            damage,
            &self.atlas,
        )
        .map_err(|_| UiNativeTextForegroundFinalizationDenial::PresentationCoverage)
    }

    pub fn new() -> Self {
        Self {
            atlas: UiNativeTextAtlas::new(),
            presentation: None,
        }
    }

    pub fn certify_finalized_text_retention(
        &self,
        first: UiNativeTextForegroundCoverageCertification,
        second: UiNativeTextForegroundCoverageCertification,
        successor: UiNativeTextForegroundCoverageCertification,
        gap: [i64; 4],
    ) -> Result<Box<[[i64; 4]]>, UiNativeTextRetentionCertificationDenial> {
        crate::native::presentation::appearance::text_retention_certification::certify_retention(
            &self.atlas,
            first.finalized,
            second.finalized,
            successor.finalized,
            gap,
        )
    }

    pub fn initialize_presentation_coverage(
        &mut self,
        candidate: UiNativeTextForegroundCoverageCertification,
        view: &UiMountedFrameConsumptionView<'_>,
        extent: [u32; 2],
    ) -> Result<(), UiNativeTextForegroundFinalizationDenial> {
        if self.presentation.is_some() {
            return Err(UiNativeTextForegroundFinalizationDenial::PresentationCoverage);
        }
        self.presentation = Some(crate::native::presentation::UiNativeRetainedDrawList::from_text_coverage_for_certification(
            candidate.finalized, view, extent, &self.atlas).map_err(|_| UiNativeTextForegroundFinalizationDenial::PresentationCoverage)?);
        Ok(())
    }

    pub fn initialize_presentation_coverages(
        &mut self,
        candidates: Vec<UiNativeTextForegroundCoverageCertification>,
        view: &UiMountedFrameConsumptionView<'_>,
        extent: [u32; 2],
    ) -> Result<(), UiNativeTextForegroundFinalizationDenial> {
        if self.presentation.is_some() {
            return Err(UiNativeTextForegroundFinalizationDenial::PresentationCoverage);
        }
        self.presentation = Some(
            crate::native::presentation::UiNativeRetainedDrawList::from_text_coverages_for_certification(
                candidates.into_iter().map(|candidate| candidate.finalized).collect(),
                view,
                extent,
                &self.atlas,
            )
            .map_err(|_| UiNativeTextForegroundFinalizationDenial::PresentationCoverage)?,
        );
        Ok(())
    }

    pub fn prepare_presentation_coverage_changes(
        &mut self,
        candidates: Vec<UiNativeTextForegroundCoverageCertification>,
        fragment: &UiUnpublishedAppearanceFragment,
        delta: &UiMountedPresentationDelta,
        view: &UiMountedFrameConsumptionView<'_>,
    ) -> Result<Box<[[i64; 4]]>, UiNativeTextForegroundFinalizationDenial> {
        self.presentation
            .as_mut()
            .ok_or(UiNativeTextForegroundFinalizationDenial::PresentationCoverage)?
            .prepare_text_coverage_changes_for_certification(
                candidates
                    .into_iter()
                    .map(|candidate| candidate.finalized)
                    .collect(),
                &self.atlas,
                fragment,
                delta,
                view,
            )
            .map_err(|_| UiNativeTextForegroundFinalizationDenial::PresentationCoverage)
    }

    pub fn certify_presentation_coverage(
        &mut self,
        candidate: UiNativeTextForegroundCoverageCertification,
        fragment: &UiUnpublishedAppearanceFragment,
        view: &UiMountedFrameConsumptionView<'_>,
    ) -> Result<Box<[[i64; 4]]>, UiNativeTextForegroundFinalizationDenial> {
        self.presentation
            .as_mut()
            .ok_or(UiNativeTextForegroundFinalizationDenial::PresentationCoverage)?
            .certify_text_coverage_settlement(candidate.finalized, &self.atlas, fragment, view)
            .map_err(|_| UiNativeTextForegroundFinalizationDenial::PresentationCoverage)
    }

    pub fn validate_images(
        &self,
        candidate: &UiNativeTextForegroundCoverageCertification,
    ) -> Result<(), UiNativeTextForegroundFinalizationDenial> {
        candidate.finalized.validate_images(&self.atlas)
    }

    pub fn finalize_existing(
        &self,
        fragment: &UiUnpublishedAppearanceFragment,
        view: &UiMountedFrameConsumptionView<'_>,
        mechanic: &UiMountedTextForegroundAppearanceMechanic,
        extent: [u32; 2],
    ) -> Result<UiNativeTextForegroundCoverageCertification, UiNativeTextForegroundFinalizationDenial>
    {
        let join = UiNativeTextForegroundJoin::admit(fragment, view, mechanic)?;
        let (finalized, cost) = join.finalize(&self.atlas, extent)?;
        Ok(UiNativeTextForegroundCoverageCertification { finalized, cost })
    }

    pub fn rasterize_with_simulated_submission(
        &mut self,
        fragment: &UiUnpublishedAppearanceFragment,
        view: &UiMountedFrameConsumptionView<'_>,
        mechanic: &UiMountedTextForegroundAppearanceMechanic,
        extent: [u32; 2],
    ) -> Result<UiNativeTextForegroundCoverageCertification, UiNativeTextForegroundFinalizationDenial>
    {
        use UiNativeTextForegroundFinalizationDenial as Denial;
        let join = UiNativeTextForegroundJoin::admit(fragment, view, mechanic)?;
        let work = view.text_raster_work().ok_or(Denial::MissingRasterWork)?;
        let plan = self
            .atlas
            .plan_many(work.demands(), &Default::default())
            .map_err(|_| Denial::AtlasAdmission)?;
        struct Callback<'a>(&'a UiMountedTextRasterWork<'a>);
        impl UiGlyphRasterMissRasterizer for Callback<'_> {
            fn rasterize(
                &mut self,
                misses: UiGlyphRasterMissSelectionView<'_>,
                sink: &mut dyn UiGlyphRasterBatchSink,
            ) -> Result<(), UiGlyphRasterCallbackDenial> {
                self.0.rasterize(misses, sink)
            }
        }
        let uploads = super::text_atlas_rasterization::rasterize_misses(&plan, &mut Callback(work))
            .map_err(|_| Denial::RasterAdmission)?;
        if !matches!(
            self.atlas
                .settle(plan, &uploads, UiNativeTextAtlasExternalOutcome::Submitted),
            UiNativeTextAtlasCommitOutcome::Committed(_)
        ) {
            return Err(Denial::AtlasAdmission);
        }
        let (finalized, cost) = join.finalize(&self.atlas, extent)?;
        Ok(UiNativeTextForegroundCoverageCertification { finalized, cost })
    }
}

pub use crate::native::presentation::appearance::text_retention_certification::UiNativeTextRetentionCertificationDenial;
