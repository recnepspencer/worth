use sha2::Digest;
use worth_ui_host_contract::UiCountedSha256;
use worth_ui_host_contract::{
    UiFontCollectionGeneration, UiFontSlant, UiQualifiedFontFaceIdentity,
};

use super::{
    face::UiQualifiedFontFace, profile_data::PROFILE_FACES, UiFontCollectionAdmissionCost,
    UiFontCollectionAdmissionDenial, UiFontFaceSource, UiGlobalFontCollection,
    UiProfileFontFaceInput,
};

impl UiGlobalFontCollection {
    pub fn admit_qualified_profile(
    ) -> Result<(Self, UiFontCollectionAdmissionCost), UiFontCollectionAdmissionDenial> {
        Self::admit_profile(
            UiFontCollectionGeneration::new(1).expect("qualified profile generation"),
            super::profile_data::embedded_profile_inputs(),
        )
    }

    pub fn admit_profile(
        generation: UiFontCollectionGeneration,
        inputs: Box<[UiProfileFontFaceInput]>,
    ) -> Result<(Self, UiFontCollectionAdmissionCost), UiFontCollectionAdmissionDenial> {
        if inputs.len() != PROFILE_FACES.len() {
            return Err(UiFontCollectionAdmissionDenial::WrongFaceCount);
        }
        let mut cost = UiFontCollectionAdmissionCost::default();
        let mut sources = Vec::with_capacity(inputs.len());
        for (input, expected) in inputs.into_vec().into_iter().zip(PROFILE_FACES) {
            cost.faces_checked += 1;
            if input.id.as_ref() != expected.id
                || usize::from(expected.fallback_rank) != sources.len()
            {
                return Err(UiFontCollectionAdmissionDenial::WrongFaceIdentity);
            }
            if input.bytes.len() != expected.byte_length {
                return Err(UiFontCollectionAdmissionDenial::WrongByteLength);
            }
            cost.bytes_hashed += u64::try_from(input.bytes.len()).expect("font bytes fit u64");
            let observed: [u8; 32] = UiCountedSha256::digest(&input.bytes).into();
            if observed != expected.digest {
                return Err(UiFontCollectionAdmissionDenial::FontDigestMismatch);
            }
            let family_name = if matches!(expected.id, "noto-sans-roman" | "noto-sans-italic") {
                "noto-sans"
            } else {
                expected.id
            };
            sources.push(UiFontFaceSource {
                bytes: input.bytes,
                face_index: expected.face_index,
                identity: UiQualifiedFontFaceIdentity::from_text_mechanics(
                    expected.digest,
                    expected.face_index,
                ),
                family: crate::font_family::profile_family_identity(family_name),
                pack: None,
                weight: 400,
                width_milli_percent: 100_000,
                slant: if expected.id.ends_with("italic") {
                    UiFontSlant::Italic
                } else {
                    UiFontSlant::Upright
                },
                emoji: expected.emoji,
                intrinsic_color: expected.emoji,
                last_resort: expected.last_resort,
                max_glyphs_per_input_byte:
                    crate::UiGlobalTextProfile::MAX_GLYPH_EXPANSION_PER_INPUT_BYTE,
            });
            cost.shaper_data_built += 1;
        }
        let faces = instantiate(&sources)?;
        cost.coverage_ranges_built = coverage_range_count(&faces)?;
        Ok((
            Self {
                generation,
                identity_digest: super::collection_identity(&sources),
                capacity_bound: super::UiFontCollectionCapacityBound::from_sources(&sources),
                lineage: std::sync::Arc::new(super::UiFontCollectionLineage::new(generation)),
                sources: sources.into_boxed_slice(),
                faces: faces.into_boxed_slice(),
                packs: Box::new([]),
                application_bytes: 0,
            },
            cost,
        ))
    }
}

pub(super) fn instantiate(
    sources: &[UiFontFaceSource],
) -> Result<Vec<UiQualifiedFontFace>, UiFontCollectionAdmissionDenial> {
    sources
        .iter()
        .map(|source| UiQualifiedFontFace::admit(source.as_face_input()))
        .collect()
}

pub(super) fn coverage_range_count(
    faces: &[UiQualifiedFontFace],
) -> Result<u32, UiFontCollectionAdmissionDenial> {
    faces.iter().try_fold(0u32, |total, face| {
        total
            .checked_add(
                u32::try_from(face.coverage_range_count())
                    .map_err(|_| UiFontCollectionAdmissionDenial::MissingUnicodeCoverage)?,
            )
            .ok_or(UiFontCollectionAdmissionDenial::MissingUnicodeCoverage)
    })
}
