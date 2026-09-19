use std::sync::Arc;

use sha2::{Digest, Sha256};
use worth_ui_host_contract::{
    UiGlyphRasterDemandRecord, UiGlyphRasterKeyInput, UiMountedCanonicalBox,
    UiMountedCanonicalBoxInput, UiMountedCoordinateSpace, UiMountedLogicalDamage, UiMountedRgba8,
    UiMountedTextForegroundSpan, UiMountedTextPaintSpanIdentity, UiTextOriginalRange,
    UiTextProfileGeneration, UiTextScaleGeneration,
};

use super::super::{
    admit_intrinsic_color_transaction, derive_glyph_raster_demand, rasterize_alpha_outline,
    UiGlyphRasterDemandRequest, UiGlyphRasterLane, UiGlyphRasterPlacement, UiGlyphRasterScale,
    UiGlyphRasterSource,
};
use super::*;
use crate::{
    qualify_text_layout, UiGlobalFontCollection, UiTextAlignment, UiTextBaseDirection,
    UiTextOverflow, UiTextParagraphAdmissionInput, UiTextParagraphConstraints,
    UiTextParagraphConstraintsInput, UiTextStyleSpan, UiTextWrap,
};

#[test]
pub(super) fn qualified_color_transaction_preserves_cluster_and_rgba_identity() {
    let source = "👩\u{200d}💻";
    let layout = layout_for(source);
    let demand = demand_for(&layout, source);
    let range = UiTextOriginalRange::new(0, source.len() as u32).unwrap();
    assert!(demand
        .records()
        .iter()
        .any(|record| record.key().source() == UiGlyphRasterSource::ColorBitmap));
    assert!(demand
        .records()
        .iter()
        .all(|record| record.attribution().original_range() == range));

    let inputs = [(&layout, &demand)];
    let admission = admit_intrinsic_color_transaction(&inputs).unwrap();
    let transaction = rasterize_intrinsic_color_transaction(&inputs, &admission).unwrap();
    assert_eq!(
        transaction.batches()[0]
            .cost()
            .ordinary()
            .outline_evaluations(),
        0
    );
    assert!(
        transaction.batches()[0]
            .cost()
            .ordinary()
            .bitmap_source_evaluations()
            > 0
    );
    let records = transaction.batches()[0].batch().records();

    assert!(!records.is_empty());
    assert!(records.iter().all(|record| {
        let digest: [u8; 32] = Sha256::digest(record.pixels()).into();
        record.key().source() == UiGlyphRasterSource::ColorBitmap
            && record.cluster() == range
            && record.stride() == record.extent().width() * 4
            && record.pixels().len()
                == usize::try_from(record.stride()).unwrap()
                    * usize::try_from(record.extent().height()).unwrap()
            && record.digest().bytes() == digest
            && record
                .pixels()
                .chunks_exact(4)
                .any(|pixel| pixel[3] != 0 && (pixel[0] != pixel[1] || pixel[1] != pixel[2]))
    }));
}

#[test]
pub(super) fn intrinsic_color_ignores_mounted_foreground_tint() {
    let source = "👩\u{200d}💻";
    let layout = layout_for(source);
    let red = demand_for_foreground(&layout, source, UiMountedRgba8::new(255, 0, 0, 255));
    let blue = demand_for_foreground(&layout, source, UiMountedRgba8::new(0, 0, 255, 255));
    assert_eq!(red.identity(), blue.identity());
    let red = rasterize_intrinsic_color(&layout, &red).unwrap();
    let blue = rasterize_intrinsic_color(&layout, &blue).unwrap();
    let red_records = red.batch().records();
    let blue_records = blue.batch().records();
    assert_eq!(red_records.len(), blue_records.len());
    for (red, blue) in red_records.iter().zip(blue_records) {
        assert_eq!(red.key(), blue.key());
        assert_eq!(red.pixels(), blue.pixels());
        assert_eq!(red.digest(), blue.digest());
    }
}

#[test]
pub(super) fn variation_selector_stays_in_one_intrinsic_color_cluster() {
    let source = "❤️";
    assert_eq!(source.as_bytes(), &[0xE2, 0x9D, 0xA4, 0xEF, 0xB8, 0x8F]);
    let layout = layout_for(source);
    let demand = demand_for(&layout, source);
    let range = UiTextOriginalRange::new(0, source.len() as u32).unwrap();
    assert!(!demand.records().is_empty());
    assert!(demand.records().iter().all(|record| {
        record.attribution().original_range() == range
            && matches!(
                record.key().source(),
                UiGlyphRasterSource::ColorOutline | UiGlyphRasterSource::ColorBitmap
            )
    }));
    let raster = rasterize_intrinsic_color(&layout, &demand).unwrap();
    assert!(raster
        .batch()
        .records()
        .iter()
        .all(|record| record.cluster() == range));
}

#[test]
pub(super) fn text_and_emoji_variation_selectors_choose_distinct_raster_lanes() {
    let text = "\u{2764}\u{FE0E}";
    let emoji = "\u{2764}\u{FE0F}";
    let text_layout = layout_for(text);
    let emoji_layout = layout_for(emoji);
    let text_demand = demand_for(&text_layout, text);
    let emoji_demand = demand_for(&emoji_layout, emoji);

    assert!(!text_demand.records().is_empty());
    assert!(text_demand.records().iter().all(|record| matches!(
        record.key().source(),
        UiGlyphRasterSource::AlphaOutline | UiGlyphRasterSource::LastResort
    )));
    assert!(!emoji_demand.records().is_empty());
    assert!(emoji_demand.records().iter().all(|record| matches!(
        record.key().source(),
        UiGlyphRasterSource::ColorOutline | UiGlyphRasterSource::ColorBitmap
    )));

    let text_alpha = rasterize_alpha_outline(&text_layout, &text_demand).unwrap();
    let text_color = rasterize_intrinsic_color(&text_layout, &text_demand).unwrap();
    let emoji_alpha = rasterize_alpha_outline(&emoji_layout, &emoji_demand).unwrap();
    let emoji_color = rasterize_intrinsic_color(&emoji_layout, &emoji_demand).unwrap();
    assert!(!text_alpha.batch().records().is_empty());
    assert!(text_color.batch().records().is_empty());
    assert!(emoji_alpha.batch().records().is_empty());
    assert!(!emoji_color.batch().records().is_empty());
    assert_ne!(text_demand.identity(), emoji_demand.identity());
}

#[test]
fn duplicated_color_demand_completes_as_one_aggregate_record_set() {
    let source = "👩\u{200d}💻";
    let layout = layout_for(source);
    let demand = demand_for(&layout, source);
    let admission =
        admit_intrinsic_color_transaction(&[(&layout, &demand), (&layout, &demand)]).unwrap();
    let transaction = rasterize_intrinsic_color_transaction(
        &[(&layout, &demand), (&layout, &demand)],
        &admission,
    )
    .unwrap();
    assert_eq!(
        transaction.batches()[0].batch().records().len(),
        usize::try_from(admission.unique_records()).unwrap()
    );
    assert!(transaction.batches()[1].batch().records().is_empty());
    assert_eq!(
        transaction.completion().unique_records(),
        admission.unique_records() as u32
    );
}

#[test]
pub(super) fn source_substitution_is_rejected_by_positioned_glyph_provenance() {
    let source = "👩\u{200d}💻";
    let layout = layout_for(source);
    let demand = demand_for(&layout, source);
    let (index, original) = demand
        .records()
        .iter()
        .copied()
        .enumerate()
        .find(|(_, record)| {
            matches!(
                record.key().source(),
                UiGlyphRasterSource::ColorOutline | UiGlyphRasterSource::ColorBitmap
            )
        })
        .unwrap();
    let key = original.key();
    let source = match key.source() {
        UiGlyphRasterSource::ColorOutline => UiGlyphRasterSource::ColorBitmap,
        UiGlyphRasterSource::ColorBitmap => UiGlyphRasterSource::ColorOutline,
        _ => unreachable!(),
    };
    let forged_key = UiGlyphRasterKey::from_text_mechanics(UiGlyphRasterKeyInput {
        font_collection: key.font_collection_generation(),
        font_collection_lineage: key.font_collection_lineage(),
        profile: key.profile_generation(),
        face: key.face(),
        glyph_id: key.glyph_id(),
        variations: key.variations(),
        palette: key.palette(),
        size: key.size(),
        source,
        dpi_milli: key.dpi_milli(),
        origin: key.fractional_origin(),
    })
    .unwrap();
    let forged = UiGlyphRasterDemandRecord::from_text_mechanics(
        forged_key,
        original.attribution(),
        original.extent(),
    )
    .unwrap();
    assert_eq!(
        super::super::qualified_raster_admission::candidate_for_record(
            &layout, &demand, index, forged,
        ),
        Err(UiGlyphRasterizationDenial::ForeignDemandRecord)
    );
}

#[test]
pub(super) fn every_unicode_17_rgi_sequence_crosses_intrinsic_color_owner() {
    let (fonts, _) = UiGlobalFontCollection::admit_qualified_profile().unwrap();
    let fonts = Arc::new(fonts);
    let mut color_records = 0_usize;
    for source in crate::font_collection::profile_data::unicode_17_rgi_emoji() {
        let layout = layout_with_fonts(source, Arc::clone(&fonts));
        assert_independent_rgi_glyph_mapping(source, &layout);
        let demand = demand_for(&layout, source);
        let range = UiTextOriginalRange::new(0, source.len() as u32).unwrap();
        assert!(
            !demand.records().is_empty(),
            "RGI sequence produced no demand"
        );
        assert!(demand.records().iter().all(|record| {
            record.attribution().original_range() == range
                && matches!(
                    record.key().source(),
                    UiGlyphRasterSource::ColorOutline | UiGlyphRasterSource::ColorBitmap
                )
        }));
        color_records += demand.records().len();
        let raster = rasterize_intrinsic_color(&layout, &demand).unwrap();
        assert!(
            !raster.batch().records().is_empty(),
            "RGI sequence produced no raster"
        );
        for record in raster.batch().records() {
            let demand_record = demand
                .records()
                .iter()
                .find(|candidate| candidate.key() == record.key())
                .expect("raster output must come from this demand");
            assert_eq!(record.attribution(), demand_record.attribution());
            assert_eq!(record.cluster(), range);
            assert!(matches!(
                record.key().source(),
                UiGlyphRasterSource::ColorOutline | UiGlyphRasterSource::ColorBitmap
            ));
            assert_eq!(record.stride(), record.extent().width() * 4);
            assert!(!record.pixels().is_empty());
            assert!(record.pixels().chunks_exact(4).any(|pixel| pixel[3] != 0));
            let digest: [u8; 32] = Sha256::digest(record.pixels()).into();
            assert_eq!(record.digest().bytes(), digest);
        }
    }
    assert_eq!(
        crate::font_collection::profile_data::unicode_17_rgi_emoji().len(),
        3_953
    );
    assert!(color_records > 0);
}

fn assert_independent_rgi_glyph_mapping(source: &str, layout: &crate::UiQualifiedTextLayout) {
    let view = layout.view();
    let run = view
        .logical_runs()
        .first()
        .expect("RGI sequence has one qualified run");
    let resource = layout
        .artifact()
        .face_resource(run.face())
        .expect("qualified run retains its exact face bytes");
    let face_index = run.face().face_index();
    let face = rustybuzz::Face::from_slice(resource.bytes(), face_index)
        .expect("qualified profile face is independently parseable");
    let mut buffer = rustybuzz::UnicodeBuffer::new();
    buffer.push_str(source);
    buffer.guess_segment_properties();
    let oracle = rustybuzz::shape(&face, &[], buffer);
    let expected = oracle
        .glyph_infos()
        .iter()
        .map(|glyph| glyph.glyph_id)
        .collect::<Vec<_>>();
    let actual = view
        .glyphs()
        .iter()
        .map(|glyph| glyph.glyph_id())
        .collect::<Vec<_>>();
    assert_eq!(
        actual, expected,
        "independent RGI glyph mapping drifted: {source}"
    );
}

pub(super) fn layout_for(source: &str) -> crate::UiQualifiedTextLayout {
    let (fonts, _) = UiGlobalFontCollection::admit_qualified_profile().unwrap();
    layout_with_fonts(source, Arc::new(fonts))
}

fn layout_with_fonts(
    source: &str,
    fonts: Arc<UiGlobalFontCollection>,
) -> crate::UiQualifiedTextLayout {
    let constraints = UiTextParagraphConstraints::new(UiTextParagraphConstraintsInput {
        language: Arc::from("und"),
        base_direction: UiTextBaseDirection::Auto,
        wrap: UiTextWrap::UnicodeWord,
        alignment: UiTextAlignment::Start,
        overflow: UiTextOverflow::Clip,
        font_size_millipoints: 14_000,
        width_millipoints: 160_000,
        line_height_millipoints: 18_000,
        letter_spacing_millipoints: 0,
        word_spacing_millipoints: 0,
        tab_interval_millipoints: 56_000,
        maximum_lines: 2,
    })
    .unwrap();
    let source: Arc<str> = Arc::from(source);
    let styles =
        Box::new([UiTextStyleSpan::whole_paragraph(source.as_ref(), &constraints).unwrap()]);
    qualify_text_layout(
        UiTextParagraphAdmissionInput {
            source,
            constraints,
            profile_generation: UiTextProfileGeneration::new(1).unwrap(),
            font_collection_generation: fonts.generation(),
            text_scale_generation: UiTextScaleGeneration::new(1).unwrap(),
            styles,
        },
        fonts,
    )
    .unwrap()
}

pub(super) fn demand_for(
    layout: &crate::UiQualifiedTextLayout,
    source: &str,
) -> UiGlyphRasterDemandBatch {
    demand_for_foreground(layout, source, UiMountedRgba8::new(255, 255, 255, 255))
}

fn demand_for_foreground(
    layout: &crate::UiQualifiedTextLayout,
    source: &str,
    foreground: UiMountedRgba8,
) -> UiGlyphRasterDemandBatch {
    derive_glyph_raster_demand(
        layout,
        UiGlyphRasterDemandRequest {
            paint_spans: &[UiMountedTextForegroundSpan::from_runtime_mounting(
                UiTextOriginalRange::new(0, source.len() as u32).unwrap(),
                foreground,
                UiMountedTextPaintSpanIdentity::from_runtime_mounting([7; 32]),
            )],
            selection: crate::UiGlyphRasterDemandSelection::LogicalDamage(&[full_damage()]),
            scale: UiGlyphRasterScale::new(1_000, layout.view().text_scale_generation()).unwrap(),
            placement: UiGlyphRasterPlacement::default(),
            lane: UiGlyphRasterLane::Ordinary,
        },
    )
    .unwrap()
}

fn full_damage() -> UiMountedLogicalDamage {
    UiMountedLogicalDamage::from_runtime_mounting(
        UiMountedCanonicalBox::canonicalize(UiMountedCanonicalBoxInput {
            x: -4.0,
            y: -24.0,
            width: 220.0,
            height: 80.0,
            coordinate_space: UiMountedCoordinateSpace::HostSurface,
        })
        .unwrap(),
    )
}
