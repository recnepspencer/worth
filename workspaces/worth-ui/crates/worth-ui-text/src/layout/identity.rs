use sha2::{Digest, Sha256};
use worth_ui_host_contract::{
    UiPositionedTextGlyphRecord, UiQualifiedTextCaretRecord, UiQualifiedTextCostRecord,
    UiQualifiedTextCoverageRecord, UiQualifiedTextGlyphRecord, UiQualifiedTextLayoutIdentity,
    UiQualifiedTextLayoutRequestIdentity, UiQualifiedTextLineRecord, UiQualifiedTextRunRecord,
    UiQualifiedTextStyleRecord, UiQualifiedTextVisualRunRecord, UiQualifiedTextWordBoundaryRecord,
    UiTextCaretAffinity, UiTextCoverageDisposition, UiTextDirection, UiTextVisualEdge,
};

use crate::{UiQualifiedTextFaceResource, UiShapedTextParagraph};

pub(super) struct UiQualifiedTextLayoutIdentityInput<'a> {
    pub shaped: &'a UiShapedTextParagraph,
    /// The request the layout names, at the width it names.
    pub request_identity: UiQualifiedTextLayoutRequestIdentity,
    pub width_millipoints: u32,
    pub word_boundaries: &'a [UiQualifiedTextWordBoundaryRecord],
    pub logical_runs: &'a [UiQualifiedTextRunRecord],
    pub logical_glyphs: &'a [UiQualifiedTextGlyphRecord],
    pub styles: &'a [UiQualifiedTextStyleRecord],
    pub lines: &'a [UiQualifiedTextLineRecord],
    pub visual_runs: &'a [UiQualifiedTextVisualRunRecord],
    pub positioned_glyphs: &'a [UiPositionedTextGlyphRecord],
    pub carets: &'a [UiQualifiedTextCaretRecord],
    pub coverage: &'a [UiQualifiedTextCoverageRecord],
    pub faces: &'a [UiQualifiedTextFaceResource],
    pub cost: UiQualifiedTextCostRecord,
}

pub(super) fn for_layout(
    input: UiQualifiedTextLayoutIdentityInput<'_>,
) -> UiQualifiedTextLayoutIdentity {
    let shaped = input.shaped;
    let mut hasher = Sha256::new();
    hasher.update(b"worth-ui-qualified-text-layout-v3\0");
    hash_bytes(&mut hasher, shaped.source().as_bytes());
    hasher.update(input.request_identity.digest());
    hasher.update(shaped.profile_generation().get().to_le_bytes());
    hasher.update(shaped.font_collection_generation().get().to_le_bytes());
    hasher.update(shaped.text_scale_generation().get().to_le_bytes());
    hash_constraints(&mut hasher, shaped, input.width_millipoints);
    hash_graphemes(&mut hasher, shaped.graphemes());
    hash_word_boundaries(&mut hasher, input.word_boundaries);
    hash_styles(&mut hasher, input.styles);
    hash_logical_records(&mut hasher, input.logical_runs, input.logical_glyphs);
    hash_layout_records(
        &mut hasher,
        input.lines,
        input.visual_runs,
        input.positioned_glyphs,
    );
    hash_carets(&mut hasher, input.carets);
    hash_coverage(&mut hasher, input.coverage);
    hash_faces(&mut hasher, input.faces);
    hash_cost(&mut hasher, input.cost);
    UiQualifiedTextLayoutIdentity::from_text_mechanics(hasher.finalize().into())
}

fn hash_word_boundaries(hasher: &mut Sha256, boundaries: &[UiQualifiedTextWordBoundaryRecord]) {
    hash_len(hasher, boundaries.len());
    for boundary in boundaries {
        hash_range(hasher, boundary.original_boundary());
    }
}

fn hash_cost(hasher: &mut Sha256, cost: UiQualifiedTextCostRecord) {
    for value in [
        cost.analyzed_bytes(),
        cost.graphemes(),
        cost.word_boundaries(),
        cost.line_opportunities(),
        cost.bidi_contexts(),
        cost.fallback_clusters(),
        cost.coverage_index_queries(),
        cost.face_shape_attempts(),
        cost.probed_glyphs(),
        cost.shaped_runs(),
        cost.shaped_scalars(),
        cost.emitted_glyphs(),
        cost.fitted_units(),
        cost.emitted_lines(),
        cost.emitted_visual_runs(),
        cost.positioned_glyphs(),
        cost.emitted_carets(),
    ] {
        hasher.update(value.to_le_bytes());
    }
}

fn hash_constraints(hasher: &mut Sha256, shaped: &UiShapedTextParagraph, width_millipoints: u32) {
    let constraints = shaped.constraints();
    hasher.update([
        direction_tag(constraints.base_direction()),
        wrap_tag(constraints.wrap()),
        alignment_tag(constraints.alignment()),
        overflow_tag(constraints.overflow()),
    ]);
    hash_bytes(hasher, constraints.language().as_bytes());
    for value in [
        constraints.font_size_millipoints(),
        width_millipoints,
        constraints.line_height_millipoints(),
        constraints.tab_interval_millipoints(),
        constraints.maximum_lines(),
    ] {
        hasher.update(value.to_le_bytes());
    }
    hasher.update(constraints.letter_spacing_millipoints().to_le_bytes());
    hasher.update(constraints.word_spacing_millipoints().to_le_bytes());
}

fn hash_styles(hasher: &mut Sha256, styles: &[UiQualifiedTextStyleRecord]) {
    hash_len(hasher, styles.len());
    for style in styles {
        hasher.update(style.original_range().start().to_le_bytes());
        hasher.update(style.original_range().end().to_le_bytes());
        hash_bytes(hasher, style.language().as_bytes());
        hasher.update(style.font_size_millipoints().to_le_bytes());
        hasher.update(style.letter_spacing_millipoints().to_le_bytes());
        hasher.update(style.word_spacing_millipoints().to_le_bytes());
        hash_len(hasher, style.family_stack().len());
        for family in style.family_stack() {
            hasher.update(family.digest());
        }
        hasher.update(style.weight().to_le_bytes());
        hasher.update(style.width_milli_percent().to_le_bytes());
        hasher.update([match style.slant() {
            worth_ui_host_contract::UiFontSlant::Upright => 0,
            worth_ui_host_contract::UiFontSlant::Italic => 1,
            worth_ui_host_contract::UiFontSlant::Oblique => 2,
        }]);
        hash_len(hasher, style.features().len());
        for feature in style.features() {
            hasher.update(feature.tag());
            hasher.update(feature.value().to_le_bytes());
        }
        hash_len(hasher, style.variations().len());
        for variation in style.variations() {
            hasher.update(variation.axis());
            hasher.update(variation.value_milli().to_le_bytes());
        }
    }
}

fn hash_logical_records(
    hasher: &mut Sha256,
    runs: &[UiQualifiedTextRunRecord],
    glyphs: &[UiQualifiedTextGlyphRecord],
) {
    hash_len(hasher, runs.len());
    for run in runs {
        hash_range(hasher, run.original_range());
        let glyph_range = run.glyph_range();
        hasher.update(glyph_range.start.to_le_bytes());
        hasher.update(glyph_range.end.to_le_bytes());
        hasher.update(run.face().selection_digest());
        hasher.update(run.face().font_bytes_digest());
        hasher.update(run.face().face_index().to_le_bytes());
        hasher.update(run.script_tag());
        hasher.update([run.bidi_level()]);
        hasher.update(run.units_per_em().to_le_bytes());
        hasher.update(run.style_index().to_le_bytes());
        hasher.update(run.ascender_font_units().to_le_bytes());
        hasher.update(run.descender_font_units().to_le_bytes());
        hasher.update(run.line_gap_font_units().to_le_bytes());
    }
    hash_len(hasher, glyphs.len());
    for glyph in glyphs {
        hasher.update(glyph.glyph_id().to_le_bytes());
        hasher.update(glyph.original_range().start().to_le_bytes());
        hasher.update(glyph.original_range().end().to_le_bytes());
        hasher.update(glyph.x_advance_font_units().to_le_bytes());
        hasher.update(glyph.y_advance_font_units().to_le_bytes());
        hasher.update(glyph.x_offset_font_units().to_le_bytes());
        hasher.update(glyph.y_offset_font_units().to_le_bytes());
        hash_font_unit_rect(hasher, glyph.ink_bounds_font_units());
    }
}

fn hash_layout_records(
    hasher: &mut Sha256,
    lines: &[UiQualifiedTextLineRecord],
    runs: &[UiQualifiedTextVisualRunRecord],
    glyphs: &[UiPositionedTextGlyphRecord],
) {
    hash_len(hasher, lines.len());
    for line in lines {
        hash_range(hasher, line.original_range());
        let visual = line.visual_run_range();
        hasher.update(visual.start.to_le_bytes());
        hasher.update(visual.end.to_le_bytes());
        hash_rect(hasher, line.logical_bounds());
        hash_rect(hasher, line.ink_bounds());
        hasher.update(line.baseline_millipoints().to_le_bytes());
        hasher.update([u8::from(line.hard_break()), u8::from(line.overflowed())]);
    }
    hash_len(hasher, runs.len());
    for run in runs {
        hash_range(hasher, run.original_range());
        hasher.update(run.line_index().to_le_bytes());
        let logical = run.logical_run_range();
        hasher.update(logical.start.to_le_bytes());
        hasher.update(logical.end.to_le_bytes());
        hasher.update([run.bidi_level()]);
        hash_rect(hasher, run.bounds());
    }
    hash_len(hasher, glyphs.len());
    for glyph in glyphs {
        hasher.update(glyph.source_glyph_index().to_le_bytes());
        hasher.update(glyph.line_index().to_le_bytes());
        hasher.update(glyph.visual_run_index().to_le_bytes());
        hasher.update(glyph.origin_x_millipoints().to_le_bytes());
        hasher.update(glyph.origin_y_millipoints().to_le_bytes());
        hasher.update(glyph.advance_x_millipoints().to_le_bytes());
        hash_rect(hasher, glyph.ink_bounds());
    }
}

fn hash_graphemes(
    hasher: &mut Sha256,
    graphemes: &[worth_ui_host_contract::UiQualifiedTextGraphemeRecord],
) {
    hash_len(hasher, graphemes.len());
    for grapheme in graphemes {
        hash_range(hasher, grapheme.original_range());
        hasher.update([grapheme.bidi_level(), direction(grapheme.direction())]);
    }
}

fn hash_carets(hasher: &mut Sha256, carets: &[UiQualifiedTextCaretRecord]) {
    hash_len(hasher, carets.len());
    for caret in carets {
        let position = caret.position();
        hash_range(hasher, position.original_boundary());
        hasher.update([
            visual_edge(position.visual_edge()),
            affinity(position.affinity()),
        ]);
        hasher.update(caret.line_index().to_le_bytes());
        hasher.update(caret.visual_run_index().to_le_bytes());
        hasher.update(caret.x_millipoints().to_le_bytes());
        hasher.update(caret.top_millipoints().to_le_bytes());
        hasher.update(caret.bottom_millipoints().to_le_bytes());
    }
}

fn hash_coverage(hasher: &mut Sha256, coverage: &[UiQualifiedTextCoverageRecord]) {
    hash_len(hasher, coverage.len());
    for record in coverage {
        hash_range(hasher, record.original_range());
        match record.face() {
            Some(face) => {
                hasher.update([1]);
                hasher.update(face.selection_digest());
            }
            None => hasher.update([0]),
        }
        hasher.update([coverage_disposition(record.disposition())]);
        hasher.update(record.attempted_collection().get().to_le_bytes());
    }
}

fn hash_faces(hasher: &mut Sha256, faces: &[UiQualifiedTextFaceResource]) {
    hash_len(hasher, faces.len());
    for face in faces {
        hasher.update(face.identity().selection_digest());
        hasher.update(face.identity().font_bytes_digest());
        hasher.update(face.identity().face_index().to_le_bytes());
        hasher.update(face.family().digest());
        match face.pack() {
            Some(pack) => {
                hasher.update([1]);
                hasher.update(pack.digest());
            }
            None => hasher.update([0]),
        }
        hasher.update([u8::from(face.intrinsic_color())]);
        for glyph in face.color_glyphs() {
            hasher.update(glyph.glyph_id().to_le_bytes());
            hasher.update([match glyph.source() {
                crate::layout_artifact::UiQualifiedTextColorSource::Outline => 0,
                crate::layout_artifact::UiQualifiedTextColorSource::Bitmap => 1,
            }]);
        }
    }
}

fn hash_len(hasher: &mut Sha256, len: usize) {
    hasher.update(
        u64::try_from(len)
            .expect("qualified profile length fits u64")
            .to_le_bytes(),
    );
}

fn hash_bytes(hasher: &mut Sha256, bytes: &[u8]) {
    hash_len(hasher, bytes.len());
    hasher.update(bytes);
}

fn hash_range(hasher: &mut Sha256, range: worth_ui_host_contract::UiTextOriginalRange) {
    hasher.update(range.start().to_le_bytes());
    hasher.update(range.end().to_le_bytes());
}

fn hash_rect(hasher: &mut Sha256, rect: worth_ui_host_contract::UiTextRect) {
    for value in [
        rect.left_millipoints(),
        rect.top_millipoints(),
        rect.right_millipoints(),
        rect.bottom_millipoints(),
    ] {
        hasher.update(value.to_le_bytes());
    }
}

fn hash_font_unit_rect(hasher: &mut Sha256, rect: worth_ui_host_contract::UiTextFontUnitRect) {
    for value in [rect.x_min(), rect.y_min(), rect.x_max(), rect.y_max()] {
        hasher.update(value.to_le_bytes());
    }
}

const fn direction(value: UiTextDirection) -> u8 {
    match value {
        UiTextDirection::LeftToRight => 0,
        UiTextDirection::RightToLeft => 1,
    }
}

const fn visual_edge(value: UiTextVisualEdge) -> u8 {
    match value {
        UiTextVisualEdge::Leading => 0,
        UiTextVisualEdge::Trailing => 1,
    }
}

const fn affinity(value: UiTextCaretAffinity) -> u8 {
    match value {
        UiTextCaretAffinity::Upstream => 0,
        UiTextCaretAffinity::Downstream => 1,
    }
}

const fn coverage_disposition(value: UiTextCoverageDisposition) -> u8 {
    match value {
        UiTextCoverageDisposition::QualifiedFace => 0,
        UiTextCoverageDisposition::MissingCluster => 1,
        UiTextCoverageDisposition::LayoutControl => 2,
    }
}

const fn direction_tag(value: crate::UiTextBaseDirection) -> u8 {
    match value {
        crate::UiTextBaseDirection::Auto => 0,
        crate::UiTextBaseDirection::LeftToRight => 1,
        crate::UiTextBaseDirection::RightToLeft => 2,
    }
}
const fn wrap_tag(value: crate::UiTextWrap) -> u8 {
    match value {
        crate::UiTextWrap::None => 0,
        crate::UiTextWrap::UnicodeWord => 1,
        crate::UiTextWrap::Grapheme => 2,
    }
}
const fn alignment_tag(value: crate::UiTextAlignment) -> u8 {
    match value {
        crate::UiTextAlignment::Start => 0,
        crate::UiTextAlignment::Center => 1,
        crate::UiTextAlignment::End => 2,
    }
}
const fn overflow_tag(value: crate::UiTextOverflow) -> u8 {
    match value {
        crate::UiTextOverflow::Clip => 0,
        crate::UiTextOverflow::Ellipsis => 1,
    }
}

#[cfg(test)]
#[path = "identity_color_tests.rs"]
mod color_tests;
