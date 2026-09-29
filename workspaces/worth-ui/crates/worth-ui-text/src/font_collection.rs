mod admission;
mod application_pack;
pub(crate) mod color_glyph;
mod coverage;
mod face;
mod ink_bounds;
mod lifecycle;
pub(super) mod profile_data;
mod selection;

pub use application_pack::{
    UiApplicationFontFaceDefinition, UiApplicationFontLicenseRecord,
    UiApplicationFontPackDefinition, UiQualifiedFontAxisReceipt, UiQualifiedFontFaceReceipt,
    UiQualifiedFontFamilyReceipt, UiQualifiedFontNameRecordReceipt, UiQualifiedFontPackReceipt,
};

use std::sync::{
    atomic::{AtomicU64, Ordering},
    Arc,
};

use face::UiQualifiedFontFace;
use harfrust::{Direction, Language, Script};
use worth_ui_host_contract::{
    UiFontCollectionGeneration, UiFontSlant, UiGlyphVariationCoordinates,
    UiQualifiedFontFaceIdentity, UiQualifiedFontFamilyIdentity, UiQualifiedFontPackIdentity,
    UiQualifiedTextStyleRecord,
};

use profile_data::PROFILE_FACES;

pub struct UiProfileFontFaceInput {
    pub id: Arc<str>,
    pub bytes: Arc<[u8]>,
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct UiFontCollectionAdmissionCost {
    faces_checked: u16,
    bytes_hashed: u64,
    shaper_data_built: u16,
    coverage_ranges_built: u32,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum UiFontCollectionAdmissionDenial {
    WrongFaceCount,
    WrongFaceIdentity,
    WrongByteLength,
    FontDigestMismatch,
    MalformedFont,
    MalformedPackDefinition,
    ApplicationFaceCapacityExceeded,
    ApplicationFontByteCapacityExceeded,
    DuplicateFontPack,
    UnknownFontPack,
    StaleCollectionGeneration,
    CollectionGenerationExhausted,
    AmbiguousFaceSelection,
    UnsupportedColorFontTable,
    MalformedColorFontTables,
    GlyphExpansionCapacityExceeded,
    UnboundedGlyphExpansion,
    FaceMetadataMismatch,
    MissingUnicodeCoverage,
    UnsupportedFontContainer,
    UnsupportedShapingTable,
}

pub struct UiGlobalFontCollection {
    generation: UiFontCollectionGeneration,
    identity_digest: [u8; 32],
    capacity_bound: UiFontCollectionCapacityBound,
    lineage: Arc<UiFontCollectionLineage>,
    sources: Box<[UiFontFaceSource]>,
    faces: Box<[UiQualifiedFontFace]>,
    packs: Box<[application_pack::UiQualifiedFontPackReceipt]>,
    application_bytes: usize,
}

#[derive(Clone, Copy)]
pub(crate) struct UiFontCollectionCapacityBound {
    max_glyphs_per_input_byte: usize,
}

struct UiFontCollectionLineage {
    current_generation: AtomicU64,
}

#[derive(Clone)]
pub(super) struct UiFontFaceSource {
    pub(super) bytes: Arc<[u8]>,
    pub(super) face_index: u32,
    pub(super) identity: UiQualifiedFontFaceIdentity,
    pub(super) family: UiQualifiedFontFamilyIdentity,
    pub(super) pack: Option<UiQualifiedFontPackIdentity>,
    pub(super) weight: u16,
    pub(super) width_milli_percent: u32,
    pub(super) slant: UiFontSlant,
    pub(super) emoji: bool,
    pub(super) intrinsic_color: bool,
    pub(super) last_resort: bool,
    pub(super) max_glyphs_per_input_byte: usize,
}

pub(super) struct UiFontShapeProbe {
    pub(super) script: Script,
    pub(super) glyph_count: usize,
    pub(super) has_notdef: bool,
    pub(super) variation_qualified: bool,
    pub(super) features_qualified: bool,
    pub(super) color_qualified: bool,
}

pub(super) struct UiFontShapedGlyph {
    pub(super) glyph_id: u32,
    pub(super) cluster: u32,
    pub(super) x_advance: i32,
    pub(super) y_advance: i32,
    pub(super) x_offset: i32,
    pub(super) y_offset: i32,
    pub(super) ink_bounds: UiFontGlyphInkBounds,
    pub(super) unsafe_to_break: bool,
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub(super) struct UiFontGlyphInkBounds {
    pub(super) x_min: i32,
    pub(super) y_min: i32,
    pub(super) x_max: i32,
    pub(super) y_max: i32,
}

pub(super) struct UiFontShapedRun {
    pub(super) units_per_em: u16,
    pub(super) ascender_font_units: i16,
    pub(super) descender_font_units: i16,
    pub(super) line_gap_font_units: i16,
    pub(super) glyphs: Box<[UiFontShapedGlyph]>,
}

impl UiGlobalFontCollection {
    pub const fn generation(&self) -> UiFontCollectionGeneration {
        self.generation
    }

    pub(crate) const fn identity_digest(&self) -> [u8; 32] {
        self.identity_digest
    }

    pub(crate) const fn capacity_bound(&self) -> UiFontCollectionCapacityBound {
        self.capacity_bound
    }

    pub(crate) fn is_current_for_admission(&self) -> bool {
        self.lineage.current_generation.load(Ordering::Acquire) == self.generation.get()
    }

    pub(super) fn advance_lineage(
        &self,
        successor: UiFontCollectionGeneration,
    ) -> Result<(), UiFontCollectionAdmissionDenial> {
        self.lineage
            .current_generation
            .compare_exchange(
                self.generation.get(),
                successor.get(),
                Ordering::AcqRel,
                Ordering::Acquire,
            )
            .map(|_| ())
            .map_err(|_| UiFontCollectionAdmissionDenial::StaleCollectionGeneration)
    }

    pub(super) fn validate_successor_generation(
        &self,
        successor: UiFontCollectionGeneration,
    ) -> Result<(), UiFontCollectionAdmissionDenial> {
        if !self.is_current_for_admission() {
            return Err(UiFontCollectionAdmissionDenial::StaleCollectionGeneration);
        }
        let expected = self
            .generation
            .get()
            .checked_add(1)
            .ok_or(UiFontCollectionAdmissionDenial::CollectionGenerationExhausted)?;
        if successor.get() == expected {
            Ok(())
        } else {
            Err(UiFontCollectionAdmissionDenial::StaleCollectionGeneration)
        }
    }

    pub fn face_count(&self) -> usize {
        self.faces.len()
    }

    pub fn application_packs(&self) -> &[application_pack::UiQualifiedFontPackReceipt] {
        &self.packs
    }

    pub const fn application_font_bytes(&self) -> usize {
        self.application_bytes
    }

    pub const fn maximum_glyph_expansion_per_input_byte(&self) -> usize {
        self.capacity_bound.max_glyphs_per_input_byte()
    }

    pub fn contains_face(&self, identity: UiQualifiedFontFaceIdentity) -> bool {
        self.faces.iter().any(|face| face.identity() == identity)
    }

    pub fn profile_face_requirements() -> impl Iterator<Item = (&'static str, &'static str)> {
        PROFILE_FACES.iter().map(|face| (face.id, face.path))
    }

    pub(super) fn probe(
        &self,
        slot: usize,
        text: &str,
        direction: Direction,
        language: &Language,
        style: &crate::UiTextStyle,
        require_color: bool,
    ) -> UiFontShapeProbe {
        self.faces[slot].probe(text, direction, language, style, require_color)
    }

    pub(super) fn shape_run(
        &self,
        slot: usize,
        text: &str,
        original_start: u32,
        direction: Direction,
        language: &Language,
        script_tag: [u8; 4],
        style: &crate::UiTextStyle,
    ) -> UiFontShapedRun {
        self.faces[slot].shape_run(text, original_start, direction, language, script_tag, style)
    }

    pub(crate) fn raster_variations(
        &self,
        face: UiQualifiedFontFaceIdentity,
        style: &UiQualifiedTextStyleRecord,
    ) -> Option<UiGlyphVariationCoordinates> {
        self.faces
            .iter()
            .find(|candidate| candidate.identity() == face)?
            .raster_variations(style)
    }

    pub(super) fn selected_face_resources(
        &self,
        runs: &[worth_ui_host_contract::UiQualifiedTextRunRecord],
    ) -> std::sync::Arc<[crate::UiQualifiedTextFaceResource]> {
        let mut identities = runs.iter().map(|run| run.face()).collect::<Vec<_>>();
        identities.sort_by_key(|identity| identity.selection_digest());
        identities.dedup();
        identities
            .into_iter()
            .map(|identity| {
                self.faces
                    .iter()
                    .find(|face| face.identity() == identity)
                    .expect("selected face belongs to admitted collection")
                    .resource()
            })
            .collect::<Vec<_>>()
            .into()
    }
}

impl UiFontCollectionCapacityBound {
    #[cfg(test)]
    pub(super) const fn qualified_profile() -> Self {
        Self {
            max_glyphs_per_input_byte:
                crate::UiGlobalTextProfile::MAX_GLYPH_EXPANSION_PER_INPUT_BYTE,
        }
    }

    pub(crate) const fn max_glyphs_per_input_byte(self) -> usize {
        self.max_glyphs_per_input_byte
    }

    pub(super) fn from_sources(sources: &[UiFontFaceSource]) -> Self {
        Self {
            max_glyphs_per_input_byte: sources
                .iter()
                .map(|source| source.max_glyphs_per_input_byte)
                .max()
                .unwrap_or(crate::UiGlobalTextProfile::MAX_GLYPH_EXPANSION_PER_INPUT_BYTE)
                .max(crate::UiGlobalTextProfile::MAX_GLYPH_EXPANSION_PER_INPUT_BYTE),
        }
    }
}

pub(super) fn collection_identity(sources: &[UiFontFaceSource]) -> [u8; 32] {
    use sha2::Digest;
    use worth_ui_host_contract::UiCountedSha256;

    let mut faces = sources
        .iter()
        .map(|source| source.identity.selection_digest())
        .collect::<Vec<_>>();
    faces.sort_unstable();
    let mut hash = UiCountedSha256::new();
    hash.update(b"worth-ui-font-collection-v2\0");
    for face in faces {
        hash.update(face);
    }
    hash.finalize().into()
}

impl UiFontCollectionLineage {
    fn new(generation: UiFontCollectionGeneration) -> Self {
        Self {
            current_generation: AtomicU64::new(generation.get()),
        }
    }
}

impl UiFontCollectionAdmissionCost {
    pub const fn faces_checked(self) -> u16 {
        self.faces_checked
    }

    pub const fn bytes_hashed(self) -> u64 {
        self.bytes_hashed
    }

    pub const fn shaper_data_built(self) -> u16 {
        self.shaper_data_built
    }

    pub const fn coverage_ranges_built(self) -> u32 {
        self.coverage_ranges_built
    }
}

#[cfg(test)]
pub(super) fn profile_inputs_from_repository() -> Box<[UiProfileFontFaceInput]> {
    profile_data::embedded_profile_inputs()
}

#[cfg(test)]
mod application_alpha_raster_controls;
#[cfg(test)]
mod application_byte_capacity_tests;
#[cfg(test)]
pub(crate) mod application_capacity_tests;
#[cfg(test)]
mod application_color_fixtures;
#[cfg(test)]
mod application_color_graph_controls;
#[cfg(test)]
mod application_color_graph_fixtures;
#[cfg(test)]
mod application_color_raster_controls;
#[cfg(test)]
mod application_color_tests;
#[cfg(test)]
pub(crate) mod application_fallback_tests;
#[cfg(test)]
mod application_family_tests;
#[cfg(test)]
mod application_feature_tests;
#[cfg(test)]
mod application_format_tests;
#[cfg(test)]
mod application_lifecycle_tests;
#[cfg(test)]
mod application_metadata_tests;
#[cfg(test)]
mod application_pack_tests;
#[cfg(test)]
pub(crate) mod application_reconstruction_tests;
#[cfg(test)]
mod application_selection_tests;
#[cfg(test)]
mod application_test_world;
#[cfg(test)]
mod application_variation_tests;
#[cfg(test)]
mod coverage_tests;
#[cfg(test)]
pub(crate) mod ink_bounds_tests;
