mod contextual_line_shaping;
#[cfg(test)]
pub(crate) mod contextual_shaping_tests;
mod ellipsis;
mod identity;
#[cfg(test)]
mod ink_metrics_tests;
mod interaction;
mod line_fitting;
#[cfg(test)]
mod original_range_tests;
#[cfg(test)]
pub(crate) mod paragraph_alignment_tests;
mod recording;
#[cfg(test)]
mod selection_tests;
mod style_projection;
mod units;
mod visual_order;

use std::sync::Arc;

use worth_ui_host_contract::{
    UiPositionedTextGlyphRecord, UiQualifiedTextCaretRecord, UiQualifiedTextCoverageRecord,
    UiQualifiedTextLayoutIdentity, UiQualifiedTextLineRecord, UiQualifiedTextSelectionRect,
    UiQualifiedTextStyleRecord, UiQualifiedTextVisualRunRecord, UiQualifiedTextWordBoundaryRecord,
    UiTextHitResult, UiTextOriginalRange, UiTextPoint,
};

use crate::{
    UiQualifiedTextLayoutArtifact, UiQualifiedTextLayoutArtifactInput, UiShapedTextParagraph,
};

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct UiTextLayoutCost {
    units_fitted: u32,
    lines_emitted: u32,
    visual_runs_emitted: u32,
    glyphs_positioned: u32,
    caret_records_emitted: u32,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum UiTextLayoutDenial {
    LineCapacityExceeded,
    RunCapacityExceeded,
    GlyphCapacityExceeded,
    EllipsisRequiresQualifiedGlyph,
    StaleFontCollectionGeneration,
    ContextualReshapingFailed(crate::UiTextShapingDenial),
    ContextualLineFittingDidNotConverge,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum UiTextSelectionDenial {
    RangeOutOfBounds,
    NotUtf8Boundary,
    NotClusterBoundary,
}

pub struct UiQualifiedTextLayout {
    fonts: Arc<crate::UiGlobalFontCollection>,
    artifact: Arc<UiQualifiedTextLayoutArtifact>,
    logical_glyphs: Arc<[worth_ui_host_contract::UiQualifiedTextGlyphRecord]>,
    identity: UiQualifiedTextLayoutIdentity,
    lines: Arc<[UiQualifiedTextLineRecord]>,
    visual_runs: Arc<[UiQualifiedTextVisualRunRecord]>,
    positioned_glyphs: Arc<[UiPositionedTextGlyphRecord]>,
    carets: Arc<[UiQualifiedTextCaretRecord]>,
    styles: Arc<[UiQualifiedTextStyleRecord]>,
    positioned_units: Box<[interaction::PositionedCluster]>,
    line_anchors: Box<[interaction::PositionedLineAnchor]>,
    reconstruction: Option<Arc<crate::UiQualifiedTextReconstructionSource>>,
    cost: UiTextLayoutCost,
    reflow: line_fitting::ReflowWidths,
    reflow_key: crate::UiQualifiedTextReflowKey,
}

impl UiQualifiedTextLayout {
    #[cfg(test)]
    pub(crate) fn layout(shaped: UiShapedTextParagraph) -> Result<Self, UiTextLayoutDenial> {
        Self::layout_with_posture(shaped, crate::qualification::QualificationPosture::Fresh)
    }

    pub(crate) fn layout_with_posture(
        shaped: UiShapedTextParagraph,
        posture: crate::qualification::QualificationPosture,
    ) -> Result<Self, UiTextLayoutDenial> {
        let fonts = Arc::clone(shaped.fonts());
        if posture.requires_current_collection() && !fonts.is_current_for_admission() {
            return Err(UiTextLayoutDenial::StaleFontCollectionGeneration);
        }
        let mut shaped = shaped;
        let contextual_line_shaping::FittedLines {
            mut units,
            mut plans,
            widths: fit_widths,
        } = contextual_line_shaping::fit(&mut shaped)?;
        let mut decided_by_fit_alone = plans.iter().all(|plan| !plan.overflowed);
        let mut logical_runs = shaped.runs().to_vec();
        let mut logical_glyphs = shaped.glyphs().to_vec();
        if plans.len() > shaped.capacity().lines() as usize {
            return Err(UiTextLayoutDenial::LineCapacityExceeded);
        }
        if shaped.constraints().overflow() == crate::UiTextOverflow::Ellipsis {
            ellipsis::apply(
                &shaped,
                &fonts,
                &mut units,
                &mut plans,
                &mut logical_runs,
                &mut logical_glyphs,
            )?;
        }
        let mut lines = Vec::with_capacity(plans.len());
        let mut visual_runs = Vec::new();
        let mut positioned_glyphs = Vec::new();
        let mut positioned_units = Vec::new();
        let mut line_anchors = Vec::new();
        for (line_index, plan) in plans.iter().enumerate() {
            let visual = visual_order::order(&shaped, &units, plan);
            decided_by_fit_alone &= visual_order::places_independently_of_width(&shaped, &visual);
            let mut output = recording::Output {
                lines: &mut lines,
                visual_runs: &mut visual_runs,
                glyphs: &mut positioned_glyphs,
                positioned_units: &mut positioned_units,
                line_anchors: &mut line_anchors,
            };
            recording::line(
                &shaped,
                &units,
                plan,
                &visual,
                &logical_runs,
                &logical_glyphs,
                line_index,
                &mut output,
            );
        }
        let carets: Arc<[_]> = interaction::carets(&positioned_units, &line_anchors).into();
        let styles: Arc<[_]> = style_projection::records(&shaped).into();
        let cost = UiTextLayoutCost {
            units_fitted: u32::try_from(units.len()).expect("profile capacity fits u32"),
            lines_emitted: u32::try_from(lines.len()).expect("profile capacity fits u32"),
            visual_runs_emitted: u32::try_from(visual_runs.len())
                .expect("profile capacity fits u32"),
            glyphs_positioned: u32::try_from(positioned_glyphs.len())
                .expect("profile capacity fits u32"),
            caret_records_emitted: u32::try_from(carets.len()).expect("profile capacity fits u32"),
        };
        let host_cost = shaped.cost_record(cost);
        let logical_runs: Arc<[_]> = logical_runs.into();
        let logical_glyphs: Arc<[_]> = logical_glyphs.into();
        let lines: Arc<[_]> = lines.into();
        let visual_runs: Arc<[_]> = visual_runs.into();
        let positioned_glyphs: Arc<[_]> = positioned_glyphs.into();
        let logical_bounds =
            recording::aggregate_line_bounds(&lines, UiQualifiedTextLineRecord::logical_bounds);
        let ink_bounds =
            recording::aggregate_line_bounds(&lines, UiQualifiedTextLineRecord::ink_bounds);
        let coverage: Arc<[_]> = shaped
            .selected_clusters()
            .iter()
            .copied()
            .map(|cluster| {
                UiQualifiedTextCoverageRecord::from_text_mechanics(
                    cluster.original_range(),
                    cluster.face(),
                    cluster.coverage(),
                    cluster.attempted_collection_generation(),
                )
            })
            .collect::<Vec<_>>()
            .into();
        let faces = fonts.selected_face_resources(&logical_runs);
        let word_boundaries: Arc<[_]> = shaped
            .word_boundaries()
            .iter()
            .copied()
            .map(|boundary| {
                let boundary = UiTextOriginalRange::from_text_mechanics(boundary, boundary)
                    .expect("admitted dictionary boundary is ordered");
                UiQualifiedTextWordBoundaryRecord::from_text_mechanics(boundary)
            })
            .collect::<Vec<_>>()
            .into();
        let requested_width = shaped.constraints().width_millipoints();
        let reflow = fit_widths.reflow(decided_by_fit_alone, requested_width);
        let parts = crate::request::UiQualifiedTextRequestParts::of_shaped(&shaped, &fonts);
        let reflow_key = parts.reflow_key();
        // Name the layout by the least width it stands for, whatever width it was fitted at.
        let request_identity = parts.identity_at(reflow.fitted_width());
        let identity = identity::for_layout(identity::UiQualifiedTextLayoutIdentityInput {
            shaped: &shaped,
            request_identity,
            width_millipoints: reflow.fitted_width(),
            word_boundaries: &word_boundaries,
            logical_runs: &logical_runs,
            logical_glyphs: &logical_glyphs,
            styles: &styles,
            lines: &lines,
            visual_runs: &visual_runs,
            positioned_glyphs: &positioned_glyphs,
            carets: &carets,
            coverage: &coverage,
            faces: &faces,
            cost: host_cost,
        });
        let profile = shaped.profile_generation();
        let font_collection = shaped.font_collection_generation();
        let text_scale = shaped.text_scale_generation();
        let width_basis =
            worth_ui_host_contract::UiQualifiedTextLayoutWidthBasis::new(reflow.fitted_width())
                .expect("a fitted width is never zero");
        let (source, graphemes) = shaped.into_artifact_source();
        let artifact = Arc::new(UiQualifiedTextLayoutArtifact::new(
            UiQualifiedTextLayoutArtifactInput {
                request_identity,
                identity,
                source,
                graphemes: graphemes.into(),
                word_boundaries,
                styles: Arc::clone(&styles),
                logical_runs: Arc::clone(&logical_runs),
                glyphs: Arc::clone(&logical_glyphs),
                lines: Arc::clone(&lines),
                visual_runs: Arc::clone(&visual_runs),
                positioned_glyphs: Arc::clone(&positioned_glyphs),
                logical_bounds,
                ink_bounds,
                carets: Arc::clone(&carets),
                coverage,
                faces,
                cost: host_cost,
                profile,
                font_collection,
                text_scale,
                width_basis,
            },
        ));
        Ok(Self {
            fonts,
            artifact,
            logical_glyphs,
            identity,
            lines,
            visual_runs,
            positioned_glyphs,
            carets,
            styles,
            positioned_units: positioned_units.into_boxed_slice(),
            line_anchors: line_anchors.into_boxed_slice(),
            reconstruction: None,
            cost,
            reflow,
            reflow_key,
        })
    }

    /// The key this layout shares with its request at every width.
    pub const fn reflow_key(&self) -> crate::UiQualifiedTextReflowKey {
        self.reflow_key
    }

    /// Whether qualifying this layout's request at `width_millipoints` lays
    /// the paragraph out as this layout, identity and all.
    pub fn admits_width(&self, width_millipoints: u32) -> bool {
        self.reflow.admits(width_millipoints)
    }

    /// The least width this layout stands for. The layouts of one request
    /// admit disjoint widths, so the one admitting a width is the one with
    /// the greatest least width not above it.
    pub const fn least_width_millipoints(&self) -> u32 {
        self.reflow.fitted_width()
    }

    pub const fn identity(&self) -> UiQualifiedTextLayoutIdentity {
        self.identity
    }
    pub fn source(&self) -> &str {
        self.artifact.view().source()
    }
    pub fn lines(&self) -> &[UiQualifiedTextLineRecord] {
        &self.lines
    }
    pub fn visual_runs(&self) -> &[UiQualifiedTextVisualRunRecord] {
        &self.visual_runs
    }
    pub fn glyphs(&self) -> &[worth_ui_host_contract::UiQualifiedTextGlyphRecord] {
        &self.logical_glyphs
    }
    pub fn positioned_glyphs(&self) -> &[UiPositionedTextGlyphRecord] {
        &self.positioned_glyphs
    }
    pub fn carets(&self) -> &[UiQualifiedTextCaretRecord] {
        &self.carets
    }
    pub fn styles(&self) -> &[UiQualifiedTextStyleRecord] {
        &self.styles
    }
    pub const fn cost(&self) -> UiTextLayoutCost {
        self.cost
    }

    pub fn pinned_font_collection(&self) -> &Arc<crate::UiGlobalFontCollection> {
        &self.fonts
    }

    pub(crate) fn artifact(&self) -> &Arc<UiQualifiedTextLayoutArtifact> {
        &self.artifact
    }

    pub fn reconstruction_source(
        &self,
    ) -> Option<&Arc<crate::UiQualifiedTextReconstructionSource>> {
        self.reconstruction.as_ref()
    }

    pub(crate) fn attach_reconstruction_source(
        &mut self,
        source: Arc<crate::UiQualifiedTextReconstructionSource>,
    ) {
        debug_assert_eq!(source.layout_identity(), self.identity);
        self.reconstruction = Some(source);
    }

    pub fn view(&self) -> worth_ui_host_contract::UiQualifiedTextLayoutView<'_> {
        self.artifact.view()
    }

    pub fn hit_test(&self, point: UiTextPoint) -> Option<UiTextHitResult> {
        interaction::hit_test(
            &self.positioned_units,
            &self.line_anchors,
            &self.carets,
            point,
        )
    }

    pub fn selection_rects(
        &self,
        range: UiTextOriginalRange,
    ) -> Result<Box<[UiQualifiedTextSelectionRect]>, UiTextSelectionDenial> {
        interaction::selection_rects(
            self.view().source(),
            self.view().graphemes(),
            &self.logical_glyphs,
            &self.positioned_units,
            range,
        )
    }
}

impl UiTextLayoutCost {
    pub const fn units_fitted(self) -> u32 {
        self.units_fitted
    }
    pub const fn lines_emitted(self) -> u32 {
        self.lines_emitted
    }
    pub const fn visual_runs_emitted(self) -> u32 {
        self.visual_runs_emitted
    }
    pub const fn glyphs_positioned(self) -> u32 {
        self.glyphs_positioned
    }
    pub const fn caret_records_emitted(self) -> u32 {
        self.caret_records_emitted
    }
}

#[cfg(test)]
pub(crate) mod line_anchor_tests;
#[cfg(test)]
pub(crate) mod rgi_tests;
#[cfg(test)]
pub(crate) mod tests;
#[cfg(test)]
pub(crate) mod word_boundary_tests;
