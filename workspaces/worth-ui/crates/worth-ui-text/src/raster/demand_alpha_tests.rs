use std::sync::Arc;

use sha2::{Digest, Sha256};
use worth_ui_host_contract::{
    UiFontCollectionLineageIdentity, UiGlyphRasterDemandRecord, UiGlyphRasterKeyInput,
    UiGlyphRasterLane, UiGlyphRasterSource, UiMountedCanonicalBox, UiMountedCanonicalBoxInput,
    UiMountedCoordinateSpace, UiMountedLogicalDamage, UiMountedRgba8, UiMountedTextForegroundSpan,
    UiMountedTextPaintSpanIdentity, UiTextOriginalRange, UiTextProfileGeneration,
    UiTextScaleGeneration,
};

use super::*;
use crate::{
    qualify_text_layout, UiGlobalFontCollection, UiTextAlignment, UiTextBaseDirection,
    UiTextOverflow, UiTextParagraphAdmissionInput, UiTextParagraphConstraints,
    UiTextParagraphConstraintsInput, UiTextStyleSpan, UiTextWrap,
};

pub(super) fn layout_for(source: &str) -> crate::UiQualifiedTextLayout {
    let (fonts, _) = UiGlobalFontCollection::admit_qualified_profile().unwrap();
    let fonts = Arc::new(fonts);
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

pub(super) fn full_damage() -> UiMountedLogicalDamage {
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

fn clipped_away_damage() -> UiMountedLogicalDamage {
    UiMountedLogicalDamage::from_runtime_mounting(
        UiMountedCanonicalBox::canonicalize(UiMountedCanonicalBoxInput {
            x: 1_000.0,
            y: 1_000.0,
            width: 1.0,
            height: 1.0,
            coordinate_space: UiMountedCoordinateSpace::HostSurface,
        })
        .unwrap(),
    )
}

fn paint_for(source: &str, identity_byte: u8) -> UiMountedTextForegroundSpan {
    UiMountedTextForegroundSpan::from_runtime_mounting(
        UiTextOriginalRange::new(0, u32::try_from(source.len()).unwrap()).unwrap(),
        UiMountedRgba8::new(255, 255, 255, 255),
        UiMountedTextPaintSpanIdentity::from_runtime_mounting([identity_byte; 32]),
    )
}

pub(super) struct DemandScenario<'a> {
    pub(super) source: &'a str,
    pub(super) damage: &'a [UiMountedLogicalDamage],
    pub(super) dpi_milli: u32,
    pub(super) lane: UiGlyphRasterLane,
}

pub(super) fn demand_for(
    layout: &crate::UiQualifiedTextLayout,
    scenario: DemandScenario<'_>,
) -> UiGlyphRasterDemandBatch {
    demand_for_at(layout, scenario, UiGlyphRasterPlacement::default())
}

pub(super) fn demand_for_spans(
    layout: &crate::UiQualifiedTextLayout,
    scenario: DemandScenario<'_>,
    paint_spans: &[UiMountedTextForegroundSpan],
) -> Result<UiGlyphRasterDemandBatch, super::UiGlyphRasterDemandDenial> {
    derive_glyph_raster_demand(
        layout,
        UiGlyphRasterDemandRequest {
            paint_spans,
            selection: crate::UiGlyphRasterDemandSelection::LogicalDamage(scenario.damage),
            scale: UiGlyphRasterScale::new(
                scenario.dpi_milli,
                layout.view().text_scale_generation(),
            )
            .unwrap(),
            placement: UiGlyphRasterPlacement::default(),
            lane: scenario.lane,
        },
    )
}

fn demand_for_at(
    layout: &crate::UiQualifiedTextLayout,
    scenario: DemandScenario<'_>,
    placement: UiGlyphRasterPlacement,
) -> UiGlyphRasterDemandBatch {
    derive_glyph_raster_demand(
        layout,
        UiGlyphRasterDemandRequest {
            paint_spans: &[paint_for(scenario.source, 31)],
            selection: crate::UiGlyphRasterDemandSelection::LogicalDamage(scenario.damage),
            scale: UiGlyphRasterScale::new(
                scenario.dpi_milli,
                layout.view().text_scale_generation(),
            )
            .unwrap(),
            placement,
            lane: scenario.lane,
        },
    )
    .unwrap()
}

#[test]
pub(super) fn mounted_translation_controls_damage_selection_and_fractional_raster_phase() {
    let source = "WORTH";
    let layout = layout_for(source);
    let damage = [UiMountedLogicalDamage::from_runtime_mounting(
        UiMountedCanonicalBox::canonicalize(UiMountedCanonicalBoxInput {
            x: 996.0,
            y: -24.0,
            width: 220.0,
            height: 80.0,
            coordinate_space: UiMountedCoordinateSpace::HostSurface,
        })
        .unwrap(),
    )];
    let scenario = || DemandScenario {
        source,
        damage: &damage,
        dpi_milli: 1_000,
        lane: UiGlyphRasterLane::Ordinary,
    };
    assert!(demand_for(&layout, scenario()).records().is_empty());
    let quarter = demand_for_at(
        &layout,
        scenario(),
        UiGlyphRasterPlacement::from_mounted_logical(1_000.25, 0.0).unwrap(),
    );
    let three_quarters = demand_for_at(
        &layout,
        scenario(),
        UiGlyphRasterPlacement::from_mounted_logical(1_000.75, 0.0).unwrap(),
    );
    assert!(!quarter.records().is_empty());
    assert_eq!(quarter.records().len(), three_quarters.records().len());
    assert_ne!(
        quarter.records()[0].key().fractional_origin(),
        three_quarters.records()[0].key().fractional_origin()
    );
    assert_ne!(quarter.identity(), three_quarters.identity());
}

#[test]
pub(super) fn demand_uses_layout_owned_lineage_and_alpha_raster_reuses_exact_misses() {
    let source = "WORTH WORTH";
    let layout = layout_for(source);
    let demand = demand_for(
        &layout,
        DemandScenario {
            source,
            damage: &[full_damage()],
            dpi_milli: 1_000,
            lane: UiGlyphRasterLane::Ordinary,
        },
    );
    assert!(!demand.records().is_empty());
    assert!(demand.cost().ordinary().layout_visits() >= demand.records().len() as u32);
    assert!(demand.records().iter().all(|record| {
        record.key().font_collection_generation() == layout.view().font_collection_generation()
            && record.key().font_collection_lineage().digest()
                == layout.pinned_font_collection().identity_digest()
            && record.key().dpi_milli() == 1_000
    }));
    let raster = rasterize_alpha_outline(&layout, &demand).unwrap();
    let records = raster.batch().records();
    assert!(!records.is_empty());
    assert!(raster.cost().ordinary().outline_evaluations() >= records.len() as u32);
    assert!(raster.cost().ordinary().produced_bytes() > 0);
    assert_eq!(
        raster.cost().ordinary().rasterized_glyphs(),
        u32::try_from(records.len()).unwrap()
    );
    assert!(records.iter().all(|record| {
        let digest: [u8; 32] = Sha256::digest(record.pixels()).into();
        matches!(
            record.key().source(),
            UiGlyphRasterSource::AlphaOutline | UiGlyphRasterSource::LastResort
        ) && usize::try_from(record.stride()).unwrap() * record.extent().height() as usize
            == record.pixels().len()
            && record.digest().bytes() == digest
    }));
}

#[test]
pub(super) fn demand_keeps_whole_clusters_and_defers_color_sources() {
    for source in ["a\u{301}", "العربية", "कर्म", "\u{0378}"] {
        let layout = layout_for(source);
        let demand = demand_for(
            &layout,
            DemandScenario {
                source,
                damage: &[full_damage()],
                dpi_milli: 1_000,
                lane: UiGlyphRasterLane::Ordinary,
            },
        );
        assert!(!demand.records().is_empty(), "{source:?} must demand ink");
        assert!(demand.records().iter().all(|record| {
            let range = record.attribution().original_range();
            range.start() < range.end() && range.end() <= source.len() as u32
        }));
        if source == "\u{0378}" {
            assert!(demand
                .records()
                .iter()
                .all(|record| record.key().source() == UiGlyphRasterSource::LastResort));
        }
    }
    let source = "👩\u{200d}💻";
    let layout = layout_for(source);
    let demand = demand_for(
        &layout,
        DemandScenario {
            source,
            damage: &[full_damage()],
            dpi_milli: 1_000,
            lane: UiGlyphRasterLane::Ordinary,
        },
    );
    assert!(demand
        .records()
        .iter()
        .any(|record| record.key().source() == UiGlyphRasterSource::ColorBitmap));
    let alpha = rasterize_alpha_outline(&layout, &demand).unwrap();
    assert!(alpha.batch().records().is_empty());
    assert_eq!(alpha.cost().ordinary().rasterized_glyphs(), 0);
}

#[test]
fn clipping_denies_no_work_and_repeated_keys_are_miss_only() {
    let clipped_layout = layout_for("WORTH");
    let clipped = demand_for(
        &clipped_layout,
        DemandScenario {
            source: "WORTH",
            damage: &[clipped_away_damage()],
            dpi_milli: 1_000,
            lane: UiGlyphRasterLane::Ordinary,
        },
    );
    assert!(clipped.records().is_empty());
    assert!(clipped.cost().ordinary().layout_visits() > 0);
    assert_eq!(clipped.cost().ordinary().demanded_glyphs(), 0);
    let clipped_alpha = rasterize_alpha_outline(&clipped_layout, &clipped).unwrap();
    assert!(clipped_alpha.batch().records().is_empty());
    assert_eq!(clipped_alpha.cost().ordinary().outline_evaluations(), 0);

    let repeated_layout = layout_for("W\nW");
    let repeated = demand_for(
        &repeated_layout,
        DemandScenario {
            source: "W\nW",
            damage: &[full_damage()],
            dpi_milli: 1_000,
            lane: UiGlyphRasterLane::Ordinary,
        },
    );
    assert!(repeated.records().len() >= 2);
    let alpha = rasterize_alpha_outline(&repeated_layout, &repeated).unwrap();
    assert!(alpha.batch().records().len() < repeated.records().len());
    assert_eq!(
        alpha.cost().ordinary().rasterized_glyphs(),
        u32::try_from(alpha.batch().records().len()).unwrap()
    );
    assert!(repeated.cost().ordinary().cache_hits() > 0);
}

#[test]
pub(super) fn same_generation_foreign_lineage_is_denied_before_outline_work() {
    let layout = layout_for("W");
    let demand = demand_for(
        &layout,
        DemandScenario {
            source: "W",
            damage: &[full_damage()],
            dpi_milli: 1_000,
            lane: UiGlyphRasterLane::Ordinary,
        },
    );
    let original = *demand.records().first().unwrap();
    let key = original.key();
    let foreign_key = UiGlyphRasterKey::from_text_mechanics(UiGlyphRasterKeyInput {
        font_collection: key.font_collection_generation(),
        font_collection_lineage: UiFontCollectionLineageIdentity::from_text_mechanics([0xA5; 32]),
        profile: key.profile_generation(),
        face: key.face(),
        glyph_id: key.glyph_id(),
        variations: key.variations(),
        palette: key.palette(),
        size: key.size(),
        source: key.source(),
        dpi_milli: key.dpi_milli(),
        origin: key.fractional_origin(),
    })
    .unwrap();
    let forged = UiGlyphRasterDemandRecord::from_text_mechanics(
        foreign_key,
        original.attribution(),
        original.extent(),
    )
    .unwrap();
    let identity = super::demand_identity::demand_identity(
        layout.identity(),
        demand.scale(),
        demand.placement(),
        demand.lane(),
        demand.scope(),
        &[forged],
    );
    let forged_demand = UiGlyphRasterDemandBatch::from_text_mechanics(
        identity,
        layout.identity(),
        demand.scale(),
        demand.placement(),
        demand.lane(),
        demand.scope(),
        [forged],
    )
    .unwrap();
    assert!(matches!(
        rasterize_alpha_outline(&layout, &forged_demand),
        Err(UiGlyphRasterizationDenial::ForeignCollectionLineage)
    ));
}
