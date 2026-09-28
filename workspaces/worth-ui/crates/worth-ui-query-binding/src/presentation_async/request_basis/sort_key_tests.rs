//! The typed sort keys order raster keys and pins exactly as their labeled
//! identity parts did, so sorting formats nothing and identities hold.
use worth_query::facade::foundation::WorthQueryAsyncRequestIdentityPart as Part;
use worth_ui_host_contract::{
    UiFontCollectionGeneration, UiFontCollectionLineageIdentity, UiGlyphRasterFractionalOrigin,
    UiGlyphRasterKey, UiGlyphRasterKeyInput, UiGlyphRasterPalette, UiGlyphRasterPinRequest,
    UiGlyphRasterSize, UiGlyphRasterSource, UiGlyphVariationCoordinates,
    UiQualifiedFontFaceIdentity, UiQualifiedTextLayoutIdentity, UiQualifiedTextVariationRecord,
    UiTextProfileGeneration,
};

use super::super::identity_parts::pin_sort_key;
use super::super::WorthUiPresentationPinBasis;
use super::{key_sort_key, raster_source_ordinal};

/// The labeled parts raster keys were once sorted by: the reference order.
fn labeled_parts(key: &UiGlyphRasterKey) -> Vec<Part> {
    let face = key.face();
    let origin = key.fractional_origin();
    let mut parts = vec![
        Part::unsigned("font-generation", key.font_collection_generation().get()),
        Part::bytes32("font-lineage", key.font_collection_lineage().digest()),
        Part::unsigned("profile", key.profile_generation().get()),
        Part::bytes32("font-bytes", face.font_bytes_digest()),
        Part::unsigned("face-index", u64::from(face.face_index())),
        Part::bytes32("selection", face.selection_digest()),
        Part::unsigned("glyph", u64::from(key.glyph_id())),
        Part::unsigned("palette", u64::from(key.palette().index())),
        Part::unsigned("size", u64::from(key.size().millipoints())),
        Part::unsigned("source", raster_source_ordinal(key.source())),
        Part::unsigned("dpi", u64::from(key.dpi_milli())),
        Part::unsigned("origin-x", u64::from(origin.x_over_64() as u16)),
        Part::unsigned("origin-y", u64::from(origin.y_over_64() as u16)),
    ];
    for (index, axis) in key.variations().records().enumerate() {
        parts.extend([
            Part::bytes4(format!("axis.{index:02}.tag"), axis.axis()),
            Part::unsigned(
                format!("axis.{index:02}.value"),
                u64::from(axis.value_milli() as u32),
            ),
        ]);
    }
    parts
}

/// Keys that differ in every field, in both directions of each signed one,
/// and in how many variation axes they carry.
fn varied_keys() -> Vec<UiGlyphRasterKey> {
    const SOURCES: [UiGlyphRasterSource; 4] = [
        UiGlyphRasterSource::LastResort,
        UiGlyphRasterSource::AlphaOutline,
        UiGlyphRasterSource::ColorBitmap,
        UiGlyphRasterSource::ColorOutline,
    ];
    let axes = [
        UiQualifiedTextVariationRecord::from_text_mechanics(*b"wght", 700_000),
        UiQualifiedTextVariationRecord::from_text_mechanics(*b"wdth", -25_000),
        UiQualifiedTextVariationRecord::from_text_mechanics(*b"opsz", 12_000),
    ];
    (0..96_u32)
        .map(|seed| {
            let pick = |bits: u32, modulus: u32| (seed.wrapping_mul(bits) >> 3) % modulus;
            let byte = |bits| u8::try_from(pick(bits, 3)).unwrap();
            let signed = |bits| i16::try_from(pick(bits, 5)).unwrap() * 16 - 32;
            let axis_count = usize::try_from(pick(29, 4)).unwrap();
            UiGlyphRasterKey::from_text_mechanics(UiGlyphRasterKeyInput {
                font_collection: UiFontCollectionGeneration::new(1 + u64::from(pick(3, 2)))
                    .unwrap(),
                font_collection_lineage: UiFontCollectionLineageIdentity::from_text_mechanics(
                    [byte(5); 32],
                ),
                profile: UiTextProfileGeneration::new(1 + u64::from(pick(7, 2))).unwrap(),
                face: UiQualifiedFontFaceIdentity::from_text_mechanics([byte(11); 32], pick(13, 2)),
                glyph_id: pick(17, 3) * 70_000 + pick(19, 4),
                variations: UiGlyphVariationCoordinates::from_records(&axes[..axis_count]).unwrap(),
                palette: UiGlyphRasterPalette::new(u16::try_from(pick(23, 2)).unwrap()),
                size: UiGlyphRasterSize::from_millipoints(9_000 + pick(31, 3) * 3_000).unwrap(),
                source: SOURCES[usize::try_from(pick(37, 4)).unwrap()],
                dpi_milli: 1_000 + pick(41, 2) * 500,
                origin: UiGlyphRasterFractionalOrigin::from_sixty_fourths(signed(43), signed(47)),
            })
            .unwrap()
        })
        .collect()
}

#[test]
fn raster_keys_sort_as_their_labeled_parts_did() {
    let mut typed = varied_keys();
    let mut labeled = typed.clone();
    typed.sort_by_cached_key(key_sort_key);
    labeled.sort_by_cached_key(labeled_parts);
    assert_eq!(typed, labeled);
    assert!(typed
        .windows(2)
        .any(|pair| pair[0].variations().len() != pair[1].variations().len()));
}

#[test]
fn pins_sort_by_layout_and_then_as_their_keys_did() {
    let layouts = [[9; 32], [2; 32]].map(UiQualifiedTextLayoutIdentity::from_text_mechanics);
    let mut typed = varied_keys()
        .into_iter()
        .enumerate()
        .map(|(index, key)| {
            WorthUiPresentationPinBasis::from_runtime(UiGlyphRasterPinRequest::from_text_mechanics(
                layouts[index % 2],
                key,
            ))
        })
        .collect::<Vec<_>>();
    let mut labeled = typed.clone();
    typed.sort_by_cached_key(pin_sort_key);
    labeled.sort_by_cached_key(|pin| {
        let mut parts = vec![Part::bytes32("layout", pin.layout().digest())];
        parts.extend(labeled_parts(&pin.key()));
        parts
    });
    assert_eq!(typed, labeled);
    assert_eq!(typed[0].layout(), layouts[1], "layout orders first");
}
