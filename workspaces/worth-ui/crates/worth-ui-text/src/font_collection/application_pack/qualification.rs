use std::collections::BTreeMap;

use sha2::Digest;

use worth_ui_host_contract::UiCountedSha256;
use worth_ui_host_contract::{
    UiFontCollectionGeneration, UiFontSlant, UiQualifiedFontFaceIdentity,
    UiQualifiedFontFamilyIdentity, UiQualifiedFontPackIdentity,
};

use super::{
    metadata::validate_face_definition, UiApplicationFontFaceDefinition,
    UiPreflightedApplicationPack, UiQualifiedApplicationPack, UiQualifiedFontFaceReceipt,
    UiQualifiedFontFamilyReceipt, UiQualifiedFontPackReceipt,
};
use crate::font_collection::{face::UiQualifiedFontFaceInput, UiFontFaceSource};

pub(in crate::font_collection) fn qualify(
    preflighted: UiPreflightedApplicationPack,
    generation: UiFontCollectionGeneration,
) -> Result<UiQualifiedApplicationPack, crate::UiFontCollectionAdmissionDenial> {
    use crate::UiFontCollectionAdmissionDenial as Denial;
    let UiPreflightedApplicationPack {
        definition,
        face_digests,
        ..
    } = preflighted;
    if definition.name.is_empty() || definition.faces.is_empty() {
        return Err(Denial::MalformedPackDefinition);
    }
    let mut qualified = Vec::with_capacity(definition.faces.len());
    for (face, digest) in definition
        .faces
        .into_vec()
        .into_iter()
        .zip(face_digests.into_vec())
    {
        let metadata = validate_face_definition(&face)?;
        qualified.push((face, digest, metadata));
    }
    qualified.sort_by(|(left, left_digest, _), (right, right_digest, _)| {
        face_key(left, left_digest).cmp(&face_key(right, right_digest))
    });
    if qualified.windows(2).any(|pair| {
        let left = &pair[0].0;
        let right = &pair[1].0;
        left.family == right.family
            && left.weight == right.weight
            && left.width_milli_percent == right.width_milli_percent
            && left.slant == right.slant
    }) {
        return Err(Denial::AmbiguousFaceSelection);
    }
    let pack_identity = pack_identity(&definition.name, &qualified);
    let families = family_receipts(pack_identity, &qualified);
    let family_by_name = families
        .iter()
        .map(|family| (family.name.clone(), family.identity))
        .collect::<BTreeMap<_, _>>();
    let mut sources = Vec::with_capacity(qualified.len());
    let mut face_receipts = Vec::with_capacity(qualified.len());
    for (face, digest, metadata) in qualified {
        let family = family_by_name[&face.family];
        let identity = UiQualifiedFontFaceIdentity::from_application_text_mechanics(
            digest,
            face.face_index,
            face_selection_digest(pack_identity, family, &face, digest),
        );
        face_receipts.push(UiQualifiedFontFaceReceipt {
            identity,
            family,
            weight: face.weight,
            width_milli_percent: face.width_milli_percent,
            slant: face.slant,
            family_names: metadata.family_names,
            style_names: metadata.style_names,
            axes: metadata.axes,
            feature_tags: metadata.feature_tags,
            coverage_range_count: metadata.coverage_range_count,
            intrinsic_color: metadata.intrinsic_color,
            max_glyphs_per_input_byte: u32::try_from(metadata.max_glyphs_per_input_byte)
                .map_err(|_| Denial::GlyphExpansionCapacityExceeded)?,
        });
        sources.push(UiFontFaceSource {
            bytes: face.bytes,
            face_index: face.face_index,
            identity,
            family,
            pack: Some(pack_identity),
            weight: face.weight,
            width_milli_percent: face.width_milli_percent,
            slant: face.slant,
            emoji: false,
            intrinsic_color: metadata.intrinsic_color,
            last_resort: false,
            max_glyphs_per_input_byte: metadata.max_glyphs_per_input_byte,
        });
    }
    Ok(UiQualifiedApplicationPack {
        receipt: UiQualifiedFontPackReceipt {
            identity: pack_identity,
            collection_generation: generation,
            families,
            faces: face_receipts.into_boxed_slice(),
        },
        sources: sources.into_boxed_slice(),
    })
}

fn face_selection_digest(
    pack: UiQualifiedFontPackIdentity,
    family: UiQualifiedFontFamilyIdentity,
    face: &UiApplicationFontFaceDefinition,
    font_bytes_digest: [u8; 32],
) -> [u8; 32] {
    let mut hash = UiCountedSha256::new();
    hash.update(b"worth-ui-application-font-face-selection-v3\0");
    hash.update(pack.digest());
    hash.update(family.digest());
    hash.update(font_bytes_digest);
    hash.update(face.face_index.to_le_bytes());
    hash.update(face.weight.to_le_bytes());
    hash.update(face.width_milli_percent.to_le_bytes());
    hash.update([slant_rank(face.slant)]);
    hash.finalize().into()
}

fn face_key<'face>(
    face: &'face UiApplicationFontFaceDefinition,
    digest: &'face [u8; 32],
) -> (&'face str, u16, u32, u8, &'face [u8; 32], u32) {
    (
        &face.family,
        face.weight,
        face.width_milli_percent,
        slant_rank(face.slant),
        digest,
        face.face_index,
    )
}

fn pack_identity(
    name: &str,
    faces: &[(
        UiApplicationFontFaceDefinition,
        [u8; 32],
        super::metadata_inventory::UiApplicationFaceMetadata,
    )],
) -> UiQualifiedFontPackIdentity {
    let mut hash = UiCountedSha256::new();
    hash.update(b"worth-ui-application-font-pack-v3\0");
    hash_framed(&mut hash, b"pack-name", name.as_bytes());
    hash.update((faces.len() as u64).to_le_bytes());
    for (face, digest, metadata) in faces {
        hash_framed(&mut hash, b"family", face.family.as_bytes());
        hash.update(digest);
        hash.update(face.face_index.to_le_bytes());
        hash.update(face.weight.to_le_bytes());
        hash.update(face.width_milli_percent.to_le_bytes());
        hash.update([slant_rank(face.slant)]);
        hash_framed(
            &mut hash,
            b"license-identifier",
            face.license.identifier.as_bytes(),
        );
        hash.update(UiCountedSha256::digest(face.license.notice.as_bytes()));
        hash.update([u8::from(metadata.intrinsic_color)]);
    }
    UiQualifiedFontPackIdentity::from_text_mechanics(hash.finalize().into())
}

fn family_receipts(
    pack: UiQualifiedFontPackIdentity,
    faces: &[(
        UiApplicationFontFaceDefinition,
        [u8; 32],
        super::metadata_inventory::UiApplicationFaceMetadata,
    )],
) -> Box<[UiQualifiedFontFamilyReceipt]> {
    faces
        .iter()
        .map(|(face, _, _)| face.family.clone())
        .collect::<std::collections::BTreeSet<_>>()
        .into_iter()
        .map(|name| {
            let mut hash = UiCountedSha256::new();
            hash.update(b"worth-ui-application-font-family-v3\0");
            hash.update(pack.digest());
            hash_framed(&mut hash, b"family-name", name.as_bytes());
            UiQualifiedFontFamilyReceipt {
                name,
                identity: UiQualifiedFontFamilyIdentity::from_text_mechanics(
                    hash.finalize().into(),
                ),
            }
        })
        .collect::<Vec<_>>()
        .into_boxed_slice()
}

fn hash_framed(hash: &mut UiCountedSha256, domain: &[u8], value: &[u8]) {
    hash.update((domain.len() as u64).to_le_bytes());
    hash.update(domain);
    hash.update((value.len() as u64).to_le_bytes());
    hash.update(value);
}

fn slant_rank(slant: UiFontSlant) -> u8 {
    match slant {
        UiFontSlant::Upright => 0,
        UiFontSlant::Italic => 1,
        UiFontSlant::Oblique => 2,
    }
}

impl UiFontFaceSource {
    pub(in crate::font_collection) fn as_face_input(&self) -> UiQualifiedFontFaceInput {
        UiQualifiedFontFaceInput {
            bytes: self.bytes.clone(),
            face_index: self.face_index,
            identity: self.identity,
            family: self.family,
            pack: self.pack,
            weight: self.weight,
            width_milli_percent: self.width_milli_percent,
            slant: self.slant,
            emoji: self.emoji,
            intrinsic_color: self.intrinsic_color,
            last_resort: self.last_resort,
        }
    }
}

#[cfg(test)]
pub(in crate::font_collection) mod tests {
    use std::sync::Arc;

    use super::*;

    #[test]
    pub(in crate::font_collection) fn pack_identity_frames_variable_fields_at_real_definition_boundaries(
    ) {
        let candidate = |pack_name: &str, family: &str| {
            let face = UiApplicationFontFaceDefinition {
                family: Arc::from(family),
                bytes: Arc::from([]),
                face_index: 0,
                weight: 400,
                width_milli_percent: 100_000,
                slant: UiFontSlant::Upright,
                license: super::super::UiApplicationFontLicenseRecord {
                    identifier: Arc::from("OFL-1.1"),
                    notice: Arc::from("same owned notice"),
                },
            };
            let metadata = super::super::metadata_inventory::UiApplicationFaceMetadata {
                family_names: Box::new([]),
                style_names: Box::new([]),
                axes: Box::new([]),
                feature_tags: Box::new([]),
                coverage_range_count: 1,
                intrinsic_color: false,
                max_glyphs_per_input_byte: 1,
            };
            pack_identity(pack_name, &[(face, [7; 32], metadata)])
        };

        assert_ne!(candidate("a", "bc"), candidate("ab", "c"));
    }
}
