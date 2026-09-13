use harfrust::{Direction, Language};
mod script_context;
use std::collections::HashMap;
use std::sync::Arc;
use worth_ui_host_contract::{UiQualifiedFontFaceIdentity, UiTextOriginalRange};

pub use worth_ui_host_contract::UiTextCoverageDisposition;

use crate::{
    font_collection::profile_data::is_rgi_emoji, UiAnalyzedTextParagraph, UiGlobalFontCollection,
};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct UiSelectedTextCluster {
    original_range: UiTextOriginalRange,
    face: Option<UiQualifiedFontFaceIdentity>,
    script_tag: [u8; 4],
    bidi_level: u8,
    rgi_emoji: bool,
    coverage: UiTextCoverageDisposition,
    attempted_collection_generation: worth_ui_host_contract::UiFontCollectionGeneration,
    face_slot: Option<u16>,
    style_index: u16,
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct UiTextFallbackCost {
    clusters_considered: u32,
    coverage_index_queries: u32,
    face_shape_attempts: u32,
    glyphs_probed: u32,
    last_resort_clusters: u32,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum UiTextFallbackDenial {
    FontCollectionGenerationMismatch,
    StaleFontCollectionGeneration,
    ForeignFontFamily,
    NoCompleteClusterFace,
    UnsupportedVariationCoordinate,
    UnsupportedOpenTypeFeature,
}

pub(crate) struct UiFallbackTextParagraph {
    analyzed: UiAnalyzedTextParagraph,
    fonts: Arc<UiGlobalFontCollection>,
    clusters: Box<[UiSelectedTextCluster]>,
    cost: UiTextFallbackCost,
}

struct UiFallbackRoutes {
    ordinary: Box<[usize]>,
    emoji: Box<[usize]>,
}

#[derive(Clone, Copy)]
enum UiCachedFontProbe {
    CoverageMiss,
    Shaped {
        script_tag: [u8; 4],
        has_notdef: bool,
        variation_qualified: bool,
        features_qualified: bool,
        color_qualified: bool,
    },
}

impl UiFallbackTextParagraph {
    #[cfg(test)]
    pub(crate) fn select(
        analyzed: UiAnalyzedTextParagraph,
        fonts: Arc<UiGlobalFontCollection>,
    ) -> Result<Self, UiTextFallbackDenial> {
        Self::select_with_posture(
            analyzed,
            fonts,
            crate::qualification::QualificationPosture::Fresh,
        )
    }

    pub(crate) fn select_with_posture(
        analyzed: UiAnalyzedTextParagraph,
        fonts: Arc<UiGlobalFontCollection>,
        posture: crate::qualification::QualificationPosture,
    ) -> Result<Self, UiTextFallbackDenial> {
        if analyzed.font_collection_generation() != fonts.generation() {
            return Err(UiTextFallbackDenial::FontCollectionGenerationMismatch);
        }
        if posture.requires_current_collection() && !fonts.is_current_for_admission() {
            return Err(UiTextFallbackDenial::StaleFontCollectionGeneration);
        }
        if analyzed.styles().iter().any(|span| {
            span.style()
                .family_stack()
                .families()
                .iter()
                .any(|family| !fonts.contains_family(*family))
        }) {
            return Err(UiTextFallbackDenial::ForeignFontFamily);
        }
        let mut cost = UiTextFallbackCost::default();
        let mut clusters = Vec::with_capacity(analyzed.graphemes().len());
        let routes = analyzed
            .styles()
            .iter()
            .map(|span| UiFallbackRoutes {
                ordinary: fonts.fallback_slots(false, span.style()).collect(),
                emoji: fonts.fallback_slots(true, span.style()).collect(),
            })
            .collect::<Vec<_>>();
        let mut probe_cache = HashMap::new();
        let mut last_emoji_classification = None;
        let mut last_probe = None;
        for grapheme in analyzed.graphemes() {
            let range = grapheme.original_range();
            let style_index = analyzed.style_index_for(range);
            let style = analyzed.styles()[style_index].style();
            let source = &analyzed.source()[range.start() as usize..range.end() as usize];
            let rgi_emoji = match last_emoji_classification {
                Some((previous, classified)) if previous == source => classified,
                _ => {
                    let classified = is_rgi_emoji(source);
                    last_emoji_classification = Some((source, classified));
                    classified
                }
            };
            let right_to_left = !grapheme.bidi_level().is_multiple_of(2);
            cost.clusters_considered += 1;
            if is_layout_control(source) {
                clusters.push(UiSelectedTextCluster {
                    original_range: range,
                    face: None,
                    script_tag: *b"Zyyy",
                    bidi_level: grapheme.bidi_level(),
                    rgi_emoji: false,
                    coverage: UiTextCoverageDisposition::LayoutControl,
                    attempted_collection_generation: fonts.generation(),
                    face_slot: None,
                    style_index: u16::try_from(style_index).expect("style cap fits u16"),
                });
                continue;
            }
            let mut selected = None;
            let mut variation_rejected = false;
            let mut feature_rejected = false;
            let slots = if rgi_emoji {
                &routes[style_index].emoji
            } else {
                &routes[style_index].ordinary
            };
            for &slot in slots {
                let key = (slot, style_index, right_to_left, rgi_emoji, source);
                let probe = match last_probe {
                    Some((previous, probe)) if previous == key => probe,
                    _ => {
                        let probe = *probe_cache.entry(key).or_insert_with(|| {
                            cost.coverage_index_queries += 1;
                            if !fonts.contains_cluster(slot, source, rgi_emoji) {
                                return UiCachedFontProbe::CoverageMiss;
                            }
                            let direction = if right_to_left {
                                Direction::RightToLeft
                            } else {
                                Direction::LeftToRight
                            };
                            let language =
                                Language::new(style.language()).expect("admitted language");
                            let probe =
                                fonts.probe(slot, source, direction, &language, style, rgi_emoji);
                            cost.face_shape_attempts += 1;
                            cost.glyphs_probed += u32::try_from(probe.glyph_count)
                                .expect("admitted glyph capacity fits u32");
                            UiCachedFontProbe::Shaped {
                                script_tag: probe.script.tag().to_be_bytes(),
                                has_notdef: probe.has_notdef,
                                variation_qualified: probe.variation_qualified,
                                features_qualified: probe.features_qualified,
                                color_qualified: probe.color_qualified,
                            }
                        });
                        last_probe = Some((key, probe));
                        probe
                    }
                };
                let UiCachedFontProbe::Shaped {
                    script_tag,
                    has_notdef,
                    variation_qualified,
                    features_qualified,
                    color_qualified,
                } = probe
                else {
                    continue;
                };
                if !variation_qualified {
                    variation_rejected = true;
                    continue;
                }
                if !features_qualified {
                    feature_rejected = true;
                    continue;
                }
                if !color_qualified {
                    continue;
                }
                if !has_notdef {
                    let coverage = if fonts.is_last_resort(slot) {
                        cost.last_resort_clusters += 1;
                        UiTextCoverageDisposition::MissingCluster
                    } else {
                        UiTextCoverageDisposition::QualifiedFace
                    };
                    selected = Some(UiSelectedTextCluster {
                        original_range: range,
                        face: Some(fonts.face_identity(slot)),
                        script_tag,
                        bidi_level: grapheme.bidi_level(),
                        rgi_emoji,
                        coverage,
                        attempted_collection_generation: fonts.generation(),
                        face_slot: Some(u16::try_from(slot).expect("profile face count fits u16")),
                        style_index: u16::try_from(style_index).expect("style cap fits u16"),
                    });
                    break;
                }
            }
            let selected = selected.ok_or(if feature_rejected {
                UiTextFallbackDenial::UnsupportedOpenTypeFeature
            } else if variation_rejected {
                UiTextFallbackDenial::UnsupportedVariationCoordinate
            } else {
                UiTextFallbackDenial::NoCompleteClusterFace
            })?;
            clusters.push(selected);
        }
        script_context::resolve(&mut clusters);
        Ok(Self {
            analyzed,
            fonts,
            clusters: clusters.into_boxed_slice(),
            cost,
        })
    }

    pub fn source(&self) -> &str {
        self.analyzed.source()
    }
    pub(crate) fn fonts(&self) -> &Arc<UiGlobalFontCollection> {
        &self.fonts
    }
    pub(crate) fn into_artifact_source(
        self,
    ) -> (
        std::sync::Arc<str>,
        Box<[worth_ui_host_contract::UiQualifiedTextGraphemeRecord]>,
    ) {
        self.analyzed.into_artifact_source()
    }

    pub fn styles(&self) -> &[crate::UiTextStyleSpan] {
        self.analyzed.styles()
    }

    pub fn clusters(&self) -> &[UiSelectedTextCluster] {
        &self.clusters
    }

    pub fn graphemes(&self) -> &[worth_ui_host_contract::UiQualifiedTextGraphemeRecord] {
        self.analyzed.graphemes()
    }

    pub fn word_boundaries(&self) -> &[u32] {
        self.analyzed.word_boundaries()
    }

    pub fn line_opportunities(&self) -> &[u32] {
        self.analyzed.line_opportunities()
    }

    pub(crate) fn bidi_paragraphs(&self) -> &[crate::analysis::UiAnalyzedBidiParagraph] {
        self.analyzed.bidi_paragraphs()
    }

    pub const fn constraints(&self) -> &crate::UiTextParagraphConstraints {
        self.analyzed.constraints()
    }

    pub const fn profile_generation(&self) -> worth_ui_host_contract::UiTextProfileGeneration {
        self.analyzed.profile_generation()
    }

    pub const fn font_collection_generation(
        &self,
    ) -> worth_ui_host_contract::UiFontCollectionGeneration {
        self.analyzed.font_collection_generation()
    }

    pub const fn text_scale_generation(&self) -> worth_ui_host_contract::UiTextScaleGeneration {
        self.analyzed.text_scale_generation()
    }
    pub const fn request_identity(
        &self,
    ) -> worth_ui_host_contract::UiQualifiedTextLayoutRequestIdentity {
        self.analyzed.request_identity()
    }

    pub const fn cost(&self) -> UiTextFallbackCost {
        self.cost
    }

    pub const fn analysis_cost(&self) -> crate::UiTextAnalysisCost {
        self.analyzed.cost()
    }

    pub(crate) const fn capacity(&self) -> crate::admission::UiTextCapacityReservation {
        self.analyzed.capacity()
    }
}

impl UiSelectedTextCluster {
    pub const fn original_range(self) -> UiTextOriginalRange {
        self.original_range
    }

    pub const fn face(self) -> Option<UiQualifiedFontFaceIdentity> {
        self.face
    }

    pub const fn script_tag(self) -> [u8; 4] {
        self.script_tag
    }

    pub const fn bidi_level(self) -> u8 {
        self.bidi_level
    }

    #[cfg(test)]
    pub const fn is_rgi_emoji(self) -> bool {
        self.rgi_emoji
    }

    pub const fn coverage(self) -> UiTextCoverageDisposition {
        self.coverage
    }

    pub const fn attempted_collection_generation(
        self,
    ) -> worth_ui_host_contract::UiFontCollectionGeneration {
        self.attempted_collection_generation
    }

    pub(crate) const fn face_slot(self) -> Option<usize> {
        match self.face_slot {
            Some(slot) => Some(slot as usize),
            None => None,
        }
    }

    pub const fn style_index(self) -> usize {
        self.style_index as usize
    }
}

fn is_layout_control(source: &str) -> bool {
    matches!(
        source,
        "\t" | "\n" | "\r" | "\r\n" | "\u{B}" | "\u{C}" | "\u{85}" | "\u{2028}" | "\u{2029}"
    )
}

impl UiTextFallbackCost {
    pub const fn clusters_considered(self) -> u32 {
        self.clusters_considered
    }

    pub const fn face_shape_attempts(self) -> u32 {
        self.face_shape_attempts
    }

    pub const fn coverage_index_queries(self) -> u32 {
        self.coverage_index_queries
    }

    pub const fn glyphs_probed(self) -> u32 {
        self.glyphs_probed
    }

    pub const fn last_resort_clusters(self) -> u32 {
        self.last_resort_clusters
    }
}

#[cfg(test)]
pub(crate) mod tests;
