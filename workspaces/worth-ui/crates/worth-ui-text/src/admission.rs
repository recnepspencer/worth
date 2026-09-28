use std::sync::Arc;

use worth_ui_host_contract::{
    UiFontCollectionGeneration, UiTextProfileGeneration, UiTextScaleGeneration,
};

use crate::{UiGlobalTextProfile, UiTextParagraphConstraints, UiTextStyleSpan};
use unicode_segmentation::UnicodeSegmentation;

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct UiTextAdmissionCost {
    input_bytes_inspected: u32,
    analysis_steps: u32,
    shaping_steps: u32,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum UiTextParagraphAdmissionDenial {
    InputCapacityExceeded,
    DerivedCapacityExceeded,
    FontCollectionGenerationMismatch,
    StaleFontCollectionGeneration,
    InvalidStyleSpans,
}

#[derive(Clone)]
pub struct UiTextParagraphAdmissionInput {
    pub source: Arc<str>,
    pub constraints: UiTextParagraphConstraints,
    pub profile_generation: UiTextProfileGeneration,
    pub font_collection_generation: UiFontCollectionGeneration,
    pub text_scale_generation: UiTextScaleGeneration,
    pub styles: Box<[UiTextStyleSpan]>,
}

pub(crate) struct UiAdmittedTextParagraph {
    source: Arc<str>,
    constraints: UiTextParagraphConstraints,
    profile_generation: UiTextProfileGeneration,
    font_collection_generation: UiFontCollectionGeneration,
    text_scale_generation: UiTextScaleGeneration,
    styles: Box<[UiTextStyleSpan]>,
    capacity: UiTextCapacityReservation,
}

#[derive(Clone, Copy)]
pub(crate) struct UiTextCapacityReservation {
    graphemes: u32,
    glyphs: u32,
    lines: u32,
    runs: u32,
}

impl UiAdmittedTextParagraph {
    #[cfg(test)]
    pub(crate) fn admit(
        input: UiTextParagraphAdmissionInput,
    ) -> Result<(Self, UiTextAdmissionCost), UiTextParagraphAdmissionDenial> {
        Self::admit_with_bound(
            input,
            crate::font_collection::UiFontCollectionCapacityBound::qualified_profile(),
        )
    }

    pub(crate) fn admit_from(
        input: UiTextParagraphAdmissionInput,
        fonts: &crate::UiGlobalFontCollection,
        posture: crate::qualification::QualificationPosture,
    ) -> Result<(Self, UiTextAdmissionCost), UiTextParagraphAdmissionDenial> {
        if input.font_collection_generation != fonts.generation() {
            return Err(UiTextParagraphAdmissionDenial::FontCollectionGenerationMismatch);
        }
        if posture.requires_current_collection() && !fonts.is_current_for_admission() {
            return Err(UiTextParagraphAdmissionDenial::StaleFontCollectionGeneration);
        }
        Self::admit_with_bound(input, fonts.capacity_bound())
    }

    fn admit_with_bound(
        input: UiTextParagraphAdmissionInput,
        capacity_bound: crate::font_collection::UiFontCollectionCapacityBound,
    ) -> Result<(Self, UiTextAdmissionCost), UiTextParagraphAdmissionDenial> {
        let bytes = input.source.len();
        if bytes > UiGlobalTextProfile::MAX_PARAGRAPH_UTF8_BYTES {
            return Err(UiTextParagraphAdmissionDenial::InputCapacityExceeded);
        }
        let ellipsis_glyph_headroom =
            if input.constraints.overflow() == crate::UiTextOverflow::Ellipsis {
                capacity_bound.max_glyphs_per_input_byte()
            } else {
                0
            };
        let glyphs = bytes
            .checked_mul(capacity_bound.max_glyphs_per_input_byte())
            .and_then(|glyphs| glyphs.checked_add(ellipsis_glyph_headroom))
            .filter(|glyphs| *glyphs <= UiGlobalTextProfile::MAX_GLYPHS)
            .ok_or(UiTextParagraphAdmissionDenial::DerivedCapacityExceeded)?;
        let ellipsis_run_headroom =
            usize::from(input.constraints.overflow() == crate::UiTextOverflow::Ellipsis);
        if input.styles.len() + ellipsis_run_headroom > UiGlobalTextProfile::MAX_RUNS_PER_PARAGRAPH
        {
            return Err(UiTextParagraphAdmissionDenial::DerivedCapacityExceeded);
        }
        let graphemes = validate_style_spans(&input.source, &input.styles)?;
        let lines = graphemes
            .saturating_add(usize::from(source_ends_with_hard_break(&input.source)))
            .max(1);
        if lines > UiGlobalTextProfile::MAX_LINE_RECORDS {
            return Err(UiTextParagraphAdmissionDenial::DerivedCapacityExceeded);
        }
        let admitted = Self {
            source: input.source,
            constraints: input.constraints,
            profile_generation: input.profile_generation,
            font_collection_generation: input.font_collection_generation,
            text_scale_generation: input.text_scale_generation,
            styles: input.styles,
            capacity: UiTextCapacityReservation {
                graphemes: u32::try_from(graphemes).expect("profile grapheme cap fits u32"),
                glyphs: u32::try_from(glyphs).expect("profile glyph cap fits u32"),
                lines: u32::try_from(lines).expect("profile line cap fits u32"),
                runs: u32::try_from(UiGlobalTextProfile::MAX_RUNS_PER_PARAGRAPH)
                    .expect("profile run cap fits u32"),
            },
        };
        Ok((
            admitted,
            UiTextAdmissionCost {
                input_bytes_inspected: u32::try_from(bytes).expect("profile byte cap fits u32"),
                ..UiTextAdmissionCost::default()
            },
        ))
    }

    pub fn source(&self) -> &str {
        &self.source
    }
    pub(crate) fn into_source(self) -> Arc<str> {
        self.source
    }
    pub const fn constraints(&self) -> &UiTextParagraphConstraints {
        &self.constraints
    }
    pub const fn profile_generation(&self) -> UiTextProfileGeneration {
        self.profile_generation
    }
    pub const fn font_collection_generation(&self) -> UiFontCollectionGeneration {
        self.font_collection_generation
    }
    pub const fn text_scale_generation(&self) -> UiTextScaleGeneration {
        self.text_scale_generation
    }
    pub fn styles(&self) -> &[UiTextStyleSpan] {
        &self.styles
    }
    pub const fn capacity(&self) -> UiTextCapacityReservation {
        self.capacity
    }
}

fn validate_style_spans(
    source: &str,
    styles: &[UiTextStyleSpan],
) -> Result<usize, UiTextParagraphAdmissionDenial> {
    if source.is_empty() {
        return styles
            .is_empty()
            .then_some(0)
            .ok_or(UiTextParagraphAdmissionDenial::InvalidStyleSpans);
    }
    if styles.is_empty() {
        return Err(UiTextParagraphAdmissionDenial::InvalidStyleSpans);
    }
    let mut boundaries = source
        .grapheme_indices(true)
        .map(|(offset, _)| offset)
        .collect::<Vec<_>>();
    boundaries.push(source.len());
    let mut expected_start = 0usize;
    for span in styles {
        let range = span.original_range();
        let start = usize::try_from(range.start())
            .map_err(|_| UiTextParagraphAdmissionDenial::InvalidStyleSpans)?;
        let end = usize::try_from(range.end())
            .map_err(|_| UiTextParagraphAdmissionDenial::InvalidStyleSpans)?;
        if start != expected_start
            || end <= start
            || end > source.len()
            || boundaries.binary_search(&start).is_err()
            || boundaries.binary_search(&end).is_err()
        {
            return Err(UiTextParagraphAdmissionDenial::InvalidStyleSpans);
        }
        expected_start = end;
    }
    if expected_start != source.len() {
        return Err(UiTextParagraphAdmissionDenial::InvalidStyleSpans);
    }
    Ok(boundaries.len() - 1)
}

fn source_ends_with_hard_break(source: &str) -> bool {
    source.chars().next_back().is_some_and(|character| {
        matches!(
            character,
            '\n' | '\r' | '\u{000B}' | '\u{000C}' | '\u{0085}' | '\u{2028}' | '\u{2029}'
        )
    })
}

impl UiTextCapacityReservation {
    pub const fn graphemes(self) -> u32 {
        self.graphemes
    }
    pub const fn glyphs(self) -> u32 {
        self.glyphs
    }
    pub const fn lines(self) -> u32 {
        self.lines
    }
    pub const fn runs(self) -> u32 {
        self.runs
    }
}

impl UiTextAdmissionCost {
    pub const fn input_bytes_inspected(self) -> u32 {
        self.input_bytes_inspected
    }
    pub const fn analysis_steps(self) -> u32 {
        self.analysis_steps
    }
    pub const fn shaping_steps(self) -> u32 {
        self.shaping_steps
    }
}

#[cfg(test)]
pub(crate) mod tests {
    use super::*;
    use crate::{
        UiTextAlignment, UiTextBaseDirection, UiTextOverflow, UiTextParagraphConstraintsInput,
        UiTextWrap,
    };

    fn constraints() -> UiTextParagraphConstraints {
        UiTextParagraphConstraints::new(UiTextParagraphConstraintsInput {
            language: Arc::from("und"),
            base_direction: UiTextBaseDirection::Auto,
            wrap: UiTextWrap::UnicodeWord,
            alignment: UiTextAlignment::Start,
            overflow: UiTextOverflow::Clip,
            font_size_millipoints: 14_000,
            width_millipoints: 320_000,
            line_height_millipoints: 18_000,
            letter_spacing_millipoints: 0,
            word_spacing_millipoints: 0,
            tab_interval_millipoints: 56_000,
            maximum_lines: 16,
        })
        .expect("qualified constraints")
    }

    #[test]
    pub(crate) fn capacity_denial_precedes_analysis_and_shaping() {
        let source = "x".repeat(UiGlobalTextProfile::MAX_PARAGRAPH_UTF8_BYTES + 1);
        let result = UiAdmittedTextParagraph::admit(UiTextParagraphAdmissionInput {
            source: Arc::from(source.clone()),
            constraints: constraints(),
            profile_generation: UiTextProfileGeneration::new(1).unwrap(),
            font_collection_generation: UiFontCollectionGeneration::new(1).unwrap(),
            text_scale_generation: UiTextScaleGeneration::new(1).unwrap(),
            styles: Box::new([]),
        });
        assert!(matches!(
            result,
            Err(UiTextParagraphAdmissionDenial::InputCapacityExceeded)
        ));
    }

    #[test]
    pub(crate) fn collection_issued_derived_bound_denies_before_grapheme_or_style_analysis() {
        let (fonts, _) = crate::UiGlobalFontCollection::admit_qualified_profile().unwrap();
        let source = "x".repeat(UiGlobalTextProfile::MAX_PARAGRAPH_UTF8_BYTES);
        let ellipsis = UiTextParagraphConstraints::new(UiTextParagraphConstraintsInput {
            language: Arc::from("und"),
            base_direction: UiTextBaseDirection::Auto,
            wrap: UiTextWrap::UnicodeWord,
            alignment: UiTextAlignment::Start,
            overflow: UiTextOverflow::Ellipsis,
            font_size_millipoints: 14_000,
            width_millipoints: 320_000,
            line_height_millipoints: 18_000,
            letter_spacing_millipoints: 0,
            word_spacing_millipoints: 0,
            tab_interval_millipoints: 56_000,
            maximum_lines: 16,
        })
        .unwrap();
        let input = UiTextParagraphAdmissionInput {
            source: Arc::from(source),
            constraints: ellipsis,
            profile_generation: UiTextProfileGeneration::new(1).unwrap(),
            font_collection_generation: fonts.generation(),
            text_scale_generation: UiTextScaleGeneration::new(1).unwrap(),
            styles: Box::new([]),
        };
        let result = UiAdmittedTextParagraph::admit_from(
            input,
            &fonts,
            crate::qualification::QualificationPosture::Fresh,
        );
        assert!(matches!(
            result,
            Err(UiTextParagraphAdmissionDenial::DerivedCapacityExceeded)
        ));
    }
}
