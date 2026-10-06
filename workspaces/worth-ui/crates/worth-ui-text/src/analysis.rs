use unicode_bidi::{BidiInfo, Level};
use unicode_segmentation::UnicodeSegmentation;
use worth_ui_host_contract::{UiQualifiedTextGraphemeRecord, UiTextOriginalRange};

use crate::{
    bidi_data::UiUnicode17BidiData, dictionary_segmentation, line_break, UiAdmittedTextParagraph,
    UiTextBaseDirection,
};

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct UiTextAnalysisCost {
    analyzed_bytes: u32,
    grapheme_records: u32,
    word_boundaries: u32,
    line_opportunities: u32,
    bidi_contexts: u32,
}

pub(crate) struct UiAnalyzedTextParagraph {
    admitted: UiAdmittedTextParagraph,
    graphemes: Box<[UiQualifiedTextGraphemeRecord]>,
    word_boundaries: Box<[u32]>,
    line_opportunities: Box<[u32]>,
    bidi_paragraphs: Box<[UiAnalyzedBidiParagraph]>,
    cost: UiTextAnalysisCost,
}

#[derive(Clone, Copy)]
pub(crate) struct UiAnalyzedBidiParagraph {
    pub(crate) start: u32,
    pub(crate) end: u32,
    pub(crate) level: u8,
}

impl UiAnalyzedTextParagraph {
    pub(crate) fn analyze(admitted: UiAdmittedTextParagraph) -> Self {
        let source = admitted.source();
        let paragraph_level = match admitted.constraints().base_direction() {
            UiTextBaseDirection::Auto => None,
            UiTextBaseDirection::LeftToRight => Some(Level::ltr()),
            UiTextBaseDirection::RightToLeft => Some(Level::rtl()),
        };
        let bidi = BidiInfo::new_with_data_source(&UiUnicode17BidiData, source, paragraph_level);
        let graphemes = source
            .grapheme_indices(true)
            .map(|(start, grapheme)| {
                let end = start + grapheme.len();
                let range = UiTextOriginalRange::from_text_mechanics(
                    u32::try_from(start).expect("admitted source fits u32"),
                    u32::try_from(end).expect("admitted source fits u32"),
                )
                .expect("grapheme indices are ordered");
                let level = bidi.levels.get(start).copied().unwrap_or_else(Level::ltr);
                UiQualifiedTextGraphemeRecord::from_text_mechanics(range, level.number())
            })
            .collect::<Vec<_>>()
            .into_boxed_slice();
        assert!(
            graphemes.len() <= admitted.capacity().graphemes() as usize,
            "admission reservation bounds analyzed graphemes"
        );
        let word_boundaries = dictionary_segmentation::word_boundaries(source);
        let line_opportunities = line_break::opportunities(source);
        let bidi_paragraphs = bidi
            .paragraphs
            .iter()
            .map(|paragraph| UiAnalyzedBidiParagraph {
                start: u32::try_from(paragraph.range.start).expect("admitted source fits u32"),
                end: u32::try_from(paragraph.range.end).expect("admitted source fits u32"),
                level: paragraph.level.number(),
            })
            .collect::<Vec<_>>()
            .into_boxed_slice();
        let cost = UiTextAnalysisCost {
            analyzed_bytes: u32::try_from(source.len()).expect("admitted source fits u32"),
            grapheme_records: u32::try_from(graphemes.len()).expect("profile cap fits u32"),
            word_boundaries: u32::try_from(word_boundaries.len()).expect("profile cap fits u32"),
            line_opportunities: u32::try_from(line_opportunities.len())
                .expect("profile cap fits u32"),
            bidi_contexts: u32::try_from(bidi.paragraphs.len()).expect("profile cap fits u32"),
        };
        Self {
            admitted,
            graphemes,
            word_boundaries,
            line_opportunities,
            bidi_paragraphs,
            cost,
        }
    }

    pub fn source(&self) -> &str {
        self.admitted.source()
    }
    pub(crate) fn into_artifact_source(
        self,
    ) -> (
        std::sync::Arc<str>,
        Box<[worth_ui_host_contract::UiQualifiedTextGraphemeRecord]>,
    ) {
        (self.admitted.into_source(), self.graphemes)
    }
    pub fn graphemes(&self) -> &[UiQualifiedTextGraphemeRecord] {
        &self.graphemes
    }
    pub fn word_boundaries(&self) -> &[u32] {
        &self.word_boundaries
    }
    pub fn line_opportunities(&self) -> &[u32] {
        &self.line_opportunities
    }
    pub(crate) fn bidi_paragraphs(&self) -> &[UiAnalyzedBidiParagraph] {
        &self.bidi_paragraphs
    }
    pub const fn constraints(&self) -> &crate::UiTextParagraphConstraints {
        self.admitted.constraints()
    }
    pub fn styles(&self) -> &[crate::UiTextStyleSpan] {
        self.admitted.styles()
    }
    pub fn style_index_for(&self, range: UiTextOriginalRange) -> usize {
        self.styles()
            .partition_point(|span| span.original_range().end() <= range.start())
    }
    pub const fn font_collection_generation(
        &self,
    ) -> worth_ui_host_contract::UiFontCollectionGeneration {
        self.admitted.font_collection_generation()
    }
    pub const fn profile_generation(&self) -> worth_ui_host_contract::UiTextProfileGeneration {
        self.admitted.profile_generation()
    }
    pub const fn text_scale_generation(&self) -> worth_ui_host_contract::UiTextScaleGeneration {
        self.admitted.text_scale_generation()
    }
    pub(crate) const fn capacity(&self) -> crate::admission::UiTextCapacityReservation {
        self.admitted.capacity()
    }
    pub const fn cost(&self) -> UiTextAnalysisCost {
        self.cost
    }
}

impl UiTextAnalysisCost {
    pub const fn analyzed_bytes(self) -> u32 {
        self.analyzed_bytes
    }
    pub const fn grapheme_records(self) -> u32 {
        self.grapheme_records
    }
    pub const fn word_boundaries(self) -> u32 {
        self.word_boundaries
    }
    pub const fn line_opportunities(self) -> u32 {
        self.line_opportunities
    }
    pub const fn bidi_contexts(self) -> u32 {
        self.bidi_contexts
    }
}

#[cfg(test)]
pub(crate) mod tests {
    use super::*;
    use crate::{
        UiTextAlignment, UiTextOverflow, UiTextParagraphAdmissionInput, UiTextParagraphConstraints,
        UiTextParagraphConstraintsInput, UiTextWrap,
    };
    use std::sync::Arc;
    use worth_ui_host_contract::{
        UiFontCollectionGeneration, UiTextProfileGeneration, UiTextScaleGeneration,
    };

    fn analyze_source(source: &str) -> UiAnalyzedTextParagraph {
        let constraints = UiTextParagraphConstraints::new(UiTextParagraphConstraintsInput {
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
        .unwrap();
        let styles = Box::new([
            crate::UiTextStyleSpan::whole_paragraph(source, &constraints).expect("nonempty source"),
        ]);
        let (admitted, _) = UiAdmittedTextParagraph::admit(UiTextParagraphAdmissionInput {
            source: Arc::from(source),
            constraints,
            profile_generation: UiTextProfileGeneration::new(1).unwrap(),
            font_collection_generation: UiFontCollectionGeneration::new(1).unwrap(),
            text_scale_generation: UiTextScaleGeneration::new(1).unwrap(),
            styles,
        })
        .unwrap();
        UiAnalyzedTextParagraph::analyze(admitted)
    }

    #[test]
    pub(crate) fn representative_rgi_emoji_sequences_remain_atomic_original_ranges() {
        for sequence in [
            "\u{1F468}\u{200D}\u{1F469}\u{200D}\u{1F467}\u{200D}\u{1F466}",
            "\u{1F469}\u{1F3FD}\u{200D}\u{1F4BB}",
            "\u{1F1FA}\u{1F1F3}",
            "1\u{FE0F}\u{20E3}",
            "\u{1F3F4}\u{E0067}\u{E0062}\u{E0073}\u{E0063}\u{E0074}\u{E007F}",
            "\u{263A}\u{FE0F}",
        ] {
            let analyzed = analyze_source(sequence);
            assert_eq!(analyzed.graphemes().len(), 1, "split {sequence:?}");
            let range = analyzed.graphemes()[0].original_range();
            assert_eq!((range.start(), range.end()), (0, sequence.len() as u32));
        }
    }

    #[test]
    fn bidi_levels_and_complex_word_boundaries_are_derived_from_original_text() {
        let source = "hello \u{05E9}\u{05DC}\u{05D5}\u{05DD} \u{0E20}\u{0E32}\u{0E29}\u{0E32}\u{0E44}\u{0E17}\u{0E22}\u{0E20}\u{0E32}\u{0E29}\u{0E32}\u{0E44}\u{0E17}\u{0E22}";
        let analyzed = analyze_source(source);
        assert!(analyzed
            .graphemes()
            .iter()
            .any(|record| !record.bidi_level().is_multiple_of(2)));
        assert_eq!(analyzed.word_boundaries().first(), Some(&0));
        assert_eq!(
            analyzed.word_boundaries().last(),
            Some(&(source.len() as u32))
        );
        assert!(analyzed.cost().bidi_contexts() >= 1);
    }
}

#[cfg(test)]
#[path = "bidi_conformance_tests.rs"]
pub(crate) mod bidi_conformance_tests;
#[cfg(test)]
#[path = "analysis_conformance_tests.rs"]
pub(crate) mod conformance_tests;
