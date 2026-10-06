use worth_ui_host_contract::{
    UiQualifiedTextCostInput, UiQualifiedTextCostRecord, UiQualifiedTextGlyphInput,
    UiQualifiedTextGlyphRecord, UiQualifiedTextRunRecord, UiTextFontUnitRect, UiTextOriginalRange,
};

use crate::{UiFallbackTextParagraph, UiSelectedTextCluster};

mod records;

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct UiTextShapingCost {
    runs_shaped: u32,
    input_scalars_shaped: u32,
    glyphs_emitted: u32,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum UiTextShapingDenial {
    RunCapacityExceeded,
    GlyphCapacityExceeded,
    NotdefAfterFaceSelection,
    MissingClusterGlyphUnavailable,
    GlyphMetricOverflow,
    ImpossibleOriginalRange,
    InconsistentRunMetrics,
}

pub(crate) struct UiShapedTextParagraph {
    fallback: UiFallbackTextParagraph,
    runs: Box<[UiQualifiedTextRunRecord]>,
    glyphs: Box<[UiQualifiedTextGlyphRecord]>,
    contextual_break_candidates: Box<[u32]>,
    cost: UiTextShapingCost,
}

impl UiShapedTextParagraph {
    pub(crate) fn shape(fallback: UiFallbackTextParagraph) -> Result<Self, UiTextShapingDenial> {
        let records = records::shape(&fallback, &[])?;
        Ok(Self {
            fallback,
            runs: records.runs,
            glyphs: records.glyphs,
            contextual_break_candidates: records.unsafe_break_boundaries,
            cost: records.cost,
        })
    }

    pub(crate) fn reshape_at_line_boundaries(
        &mut self,
        boundaries: &[u32],
    ) -> Result<(), UiTextShapingDenial> {
        let records = records::shape(&self.fallback, boundaries)?;
        self.runs = records.runs;
        self.glyphs = records.glyphs;
        self.contextual_break_candidates = merge_break_candidates(
            &self.contextual_break_candidates,
            &records.unsafe_break_boundaries,
        );
        self.cost.add(records.cost);
        Ok(())
    }

    pub(crate) fn break_is_contextual(&self, boundary: u32) -> bool {
        self.contextual_break_candidates
            .binary_search(&boundary)
            .is_ok()
    }

    pub fn source(&self) -> &str {
        self.fallback.source()
    }
    pub(crate) fn into_artifact_source(
        self,
    ) -> (
        std::sync::Arc<str>,
        Box<[worth_ui_host_contract::UiQualifiedTextGraphemeRecord]>,
    ) {
        self.fallback.into_artifact_source()
    }
    pub fn runs(&self) -> &[UiQualifiedTextRunRecord] {
        &self.runs
    }
    pub fn glyphs(&self) -> &[UiQualifiedTextGlyphRecord] {
        &self.glyphs
    }
    pub(crate) fn selected_clusters(&self) -> &[UiSelectedTextCluster] {
        self.fallback.clusters()
    }
    pub fn graphemes(&self) -> &[worth_ui_host_contract::UiQualifiedTextGraphemeRecord] {
        self.fallback.graphemes()
    }
    pub fn word_boundaries(&self) -> &[u32] {
        self.fallback.word_boundaries()
    }
    pub fn line_opportunities(&self) -> &[u32] {
        self.fallback.line_opportunities()
    }
    pub(crate) fn bidi_paragraphs(&self) -> &[crate::analysis::UiAnalyzedBidiParagraph] {
        self.fallback.bidi_paragraphs()
    }
    pub fn styles(&self) -> &[crate::UiTextStyleSpan] {
        self.fallback.styles()
    }
    pub const fn constraints(&self) -> &crate::UiTextParagraphConstraints {
        self.fallback.constraints()
    }
    pub const fn profile_generation(&self) -> worth_ui_host_contract::UiTextProfileGeneration {
        self.fallback.profile_generation()
    }
    pub const fn font_collection_generation(
        &self,
    ) -> worth_ui_host_contract::UiFontCollectionGeneration {
        self.fallback.font_collection_generation()
    }
    pub const fn text_scale_generation(&self) -> worth_ui_host_contract::UiTextScaleGeneration {
        self.fallback.text_scale_generation()
    }
    /// What qualifying this paragraph cost at every stage, given what laying
    /// it out cost.
    pub(crate) fn cost_record(&self, layout: crate::UiTextLayoutCost) -> UiQualifiedTextCostRecord {
        let analysis = self.analysis_cost();
        let fallback = self.fallback_cost();
        UiQualifiedTextCostRecord::from_text_mechanics(UiQualifiedTextCostInput {
            analyzed_bytes: analysis.analyzed_bytes(),
            graphemes: analysis.grapheme_records(),
            word_boundaries: analysis.word_boundaries(),
            line_opportunities: analysis.line_opportunities(),
            bidi_contexts: analysis.bidi_contexts(),
            fallback_clusters: fallback.clusters_considered(),
            coverage_index_queries: fallback.coverage_index_queries(),
            face_shape_attempts: fallback.face_shape_attempts(),
            probed_glyphs: fallback.glyphs_probed(),
            shaped_runs: self.cost.runs_shaped(),
            shaped_scalars: self.cost.input_scalars_shaped(),
            emitted_glyphs: self.cost.glyphs_emitted(),
            fitted_units: layout.units_fitted(),
            emitted_lines: layout.lines_emitted(),
            emitted_visual_runs: layout.visual_runs_emitted(),
            positioned_glyphs: layout.glyphs_positioned(),
            emitted_carets: layout.caret_records_emitted(),
        })
    }
    pub const fn analysis_cost(&self) -> crate::UiTextAnalysisCost {
        self.fallback.analysis_cost()
    }
    pub const fn fallback_cost(&self) -> crate::UiTextFallbackCost {
        self.fallback.cost()
    }
    pub(crate) fn fonts(&self) -> &std::sync::Arc<crate::UiGlobalFontCollection> {
        self.fallback.fonts()
    }
    pub(crate) const fn capacity(&self) -> crate::admission::UiTextCapacityReservation {
        self.fallback.capacity()
    }
}

fn merge_break_candidates(current: &[u32], discovered: &[u32]) -> Box<[u32]> {
    let mut candidates = Vec::with_capacity(current.len() + discovered.len());
    candidates.extend_from_slice(current);
    candidates.extend_from_slice(discovered);
    candidates.sort_unstable();
    candidates.dedup();
    candidates.into_boxed_slice()
}

pub(super) fn missing_cluster_glyph(
    glyphs: &[crate::font_collection::UiFontShapedGlyph],
    original_start: u32,
    original_end: u32,
) -> Result<UiQualifiedTextGlyphRecord, UiTextShapingDenial> {
    let first = glyphs
        .first()
        .ok_or(UiTextShapingDenial::MissingClusterGlyphUnavailable)?;
    let x_advance = checked_metric_sum(glyphs.iter().map(|glyph| glyph.x_advance))?;
    let y_advance = checked_metric_sum(glyphs.iter().map(|glyph| glyph.y_advance))?;
    let original_range = UiTextOriginalRange::from_text_mechanics(original_start, original_end)
        .ok_or(UiTextShapingDenial::ImpossibleOriginalRange)?;
    Ok(UiQualifiedTextGlyphRecord::from_text_mechanics(
        UiQualifiedTextGlyphInput {
            glyph_id: first.glyph_id,
            original_range,
            x_advance_font_units: x_advance,
            y_advance_font_units: y_advance,
            x_offset_font_units: first.x_offset,
            y_offset_font_units: first.y_offset,
            ink_bounds_font_units: font_unit_bounds(first),
        },
    ))
}

pub(super) fn qualified_glyph(
    glyph: crate::font_collection::UiFontShapedGlyph,
    original_range: UiTextOriginalRange,
) -> UiQualifiedTextGlyphRecord {
    UiQualifiedTextGlyphRecord::from_text_mechanics(UiQualifiedTextGlyphInput {
        glyph_id: glyph.glyph_id,
        original_range,
        x_advance_font_units: glyph.x_advance,
        y_advance_font_units: glyph.y_advance,
        x_offset_font_units: glyph.x_offset,
        y_offset_font_units: glyph.y_offset,
        ink_bounds_font_units: font_unit_bounds(&glyph),
    })
}

fn font_unit_bounds(glyph: &crate::font_collection::UiFontShapedGlyph) -> UiTextFontUnitRect {
    UiTextFontUnitRect::from_text_mechanics(
        glyph.ink_bounds.x_min,
        glyph.ink_bounds.y_min,
        glyph.ink_bounds.x_max,
        glyph.ink_bounds.y_max,
    )
    .expect("font-derived glyph bounds are ordered")
}

pub(super) fn checked_metric_sum(
    mut metrics: impl Iterator<Item = i32>,
) -> Result<i32, UiTextShapingDenial> {
    metrics.try_fold(0_i32, |sum, value| {
        sum.checked_add(value)
            .ok_or(UiTextShapingDenial::GlyphMetricOverflow)
    })
}

pub(super) fn cluster_boundaries(
    glyphs: &[crate::font_collection::UiFontShapedGlyph],
    start: u32,
    end: u32,
) -> Result<Vec<u32>, UiTextShapingDenial> {
    let mut boundaries = glyphs.iter().map(|glyph| glyph.cluster).collect::<Vec<_>>();
    if boundaries
        .iter()
        .any(|boundary| *boundary < start || *boundary >= end)
    {
        return Err(UiTextShapingDenial::ImpossibleOriginalRange);
    }
    boundaries.push(start);
    boundaries.push(end);
    boundaries.sort_unstable();
    boundaries.dedup();
    Ok(boundaries)
}

pub(super) fn range_for_cluster(
    boundaries: &[u32],
    cluster: u32,
) -> Result<UiTextOriginalRange, UiTextShapingDenial> {
    let index = boundaries
        .binary_search(&cluster)
        .map_err(|_| UiTextShapingDenial::ImpossibleOriginalRange)?;
    let end = *boundaries
        .get(index + 1)
        .ok_or(UiTextShapingDenial::ImpossibleOriginalRange)?;
    UiTextOriginalRange::from_text_mechanics(cluster, end)
        .ok_or(UiTextShapingDenial::ImpossibleOriginalRange)
}

impl UiTextShapingCost {
    fn add(&mut self, other: Self) {
        self.runs_shaped = self.runs_shaped.saturating_add(other.runs_shaped);
        self.input_scalars_shaped = self
            .input_scalars_shaped
            .saturating_add(other.input_scalars_shaped);
        self.glyphs_emitted = self.glyphs_emitted.saturating_add(other.glyphs_emitted);
    }
    pub const fn runs_shaped(self) -> u32 {
        self.runs_shaped
    }
    pub const fn input_scalars_shaped(self) -> u32 {
        self.input_scalars_shaped
    }
    pub const fn glyphs_emitted(self) -> u32 {
        self.glyphs_emitted
    }
}

#[cfg(test)]
pub(crate) mod reference_fixture_tests;
#[cfg(test)]
pub(crate) mod tests;
