//! Cross-layout equivalence controls for alpha raster transactions.

use std::sync::Arc;

use worth_ui_host_contract::{
    UiMountedCanonicalBox, UiMountedCanonicalBoxInput, UiMountedCoordinateSpace,
    UiMountedLogicalDamage, UiMountedRgba8, UiMountedTextForegroundSpan,
    UiMountedTextPaintSpanIdentity, UiTextOriginalRange, UiTextProfileGeneration,
    UiTextScaleGeneration,
};

use super::{
    admit_alpha_outline_transaction, derive_glyph_raster_demand,
    rasterize_alpha_outline_transaction, UiGlyphRasterDemandBatch, UiGlyphRasterDemandRequest,
    UiGlyphRasterLane, UiGlyphRasterPlacement, UiGlyphRasterScale, UiGlyphRasterSource,
};
use crate::{
    qualify_text_layout, UiGlobalFontCollection, UiQualifiedTextLayout, UiTextAlignment,
    UiTextBaseDirection, UiTextOverflow, UiTextParagraphAdmissionInput, UiTextParagraphConstraints,
    UiTextParagraphConstraintsInput, UiTextStyleSpan, UiTextWrap,
};

#[test]
pub(super) fn distinct_layout_attributions_share_one_alpha_raster_key() {
    let (fonts, _) = UiGlobalFontCollection::admit_qualified_profile().unwrap();
    let fonts = Arc::new(fonts);
    let source = "W";
    let left = layout(source, 160_000, Arc::clone(&fonts));
    let right = layout(source, 200_000, fonts);
    assert_ne!(left.identity(), right.identity());
    let left_demand = demand(&left, source);
    let right_demand = demand(&right, source);
    let left_record = alpha_record(&left_demand);
    let right_record = right_demand
        .records()
        .iter()
        .copied()
        .find(|record| record.key() == left_record.key())
        .expect("second layout demands the same alpha raster key");
    assert_ne!(left_record.attribution(), right_record.attribution());

    let inputs = [(&left, &left_demand), (&right, &right_demand)];
    let admission = admit_alpha_outline_transaction(&inputs).unwrap();
    assert_eq!(admission.unique_records(), 1);
    let transaction = rasterize_alpha_outline_transaction(&inputs, &admission).unwrap();
    let shared = transaction
        .batches()
        .iter()
        .flat_map(|raster| raster.batch().records())
        .filter(|record| record.key() == left_record.key())
        .collect::<Vec<_>>();
    assert_eq!(shared.len(), 1);
    assert_eq!(shared[0].attribution(), left_record.attribution());
    assert_eq!(transaction.completion().batches().len(), 2);
    assert_eq!(
        transaction.completion().batches()[0].demand_identity(),
        left_demand.identity()
    );
    assert_eq!(
        transaction.completion().batches()[1].demand_identity(),
        right_demand.identity()
    );
    assert_eq!(transaction.batches()[1].cost().ordinary().cache_hits(), 1);

    let transaction = rasterize_alpha_outline_transaction(&inputs, &admission).unwrap();
    let mut swapped = transaction.into_batches().into_vec();
    swapped.swap(0, 1);
    assert_eq!(
        super::alpha_transaction_completion::complete_alpha_raster_transaction(
            &admission, &swapped,
        ),
        Err(super::UiGlyphRasterizationDenial::TransactionOutputMismatch),
        "alpha completion must authenticate batch layout/demand attribution",
    );
}

fn alpha_record(
    demand: &UiGlyphRasterDemandBatch,
) -> worth_ui_host_contract::UiGlyphRasterDemandRecord {
    demand
        .records()
        .iter()
        .copied()
        .find(|record| {
            matches!(
                record.key().source(),
                UiGlyphRasterSource::AlphaOutline | UiGlyphRasterSource::LastResort
            )
        })
        .expect("qualified alpha demand")
}

fn layout(
    source: &str,
    width_millipoints: u32,
    fonts: Arc<UiGlobalFontCollection>,
) -> UiQualifiedTextLayout {
    let constraints = UiTextParagraphConstraints::new(UiTextParagraphConstraintsInput {
        language: Arc::from("und"),
        base_direction: UiTextBaseDirection::Auto,
        wrap: UiTextWrap::UnicodeWord,
        alignment: UiTextAlignment::Start,
        overflow: UiTextOverflow::Clip,
        font_size_millipoints: 14_000,
        width_millipoints,
        line_height_millipoints: 18_000,
        letter_spacing_millipoints: 0,
        word_spacing_millipoints: 0,
        tab_interval_millipoints: 56_000,
        maximum_lines: 2,
    })
    .unwrap();
    let source: Arc<str> = Arc::from(source);
    let styles = Box::new([UiTextStyleSpan::whole_paragraph(&source, &constraints).unwrap()]);
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

fn demand(layout: &UiQualifiedTextLayout, source: &str) -> UiGlyphRasterDemandBatch {
    derive_glyph_raster_demand(
        layout,
        UiGlyphRasterDemandRequest {
            paint_spans: &[UiMountedTextForegroundSpan::from_runtime_mounting(
                UiTextOriginalRange::new(0, u32::try_from(source.len()).unwrap()).unwrap(),
                UiMountedRgba8::new(255, 255, 255, 255),
                UiMountedTextPaintSpanIdentity::from_runtime_mounting([0x51; 32]),
            )],
            selection: crate::UiGlyphRasterDemandSelection::LogicalDamage(&[
                UiMountedLogicalDamage::from_runtime_mounting(
                    UiMountedCanonicalBox::canonicalize(UiMountedCanonicalBoxInput {
                        x: -4.0,
                        y: -24.0,
                        width: 220.0,
                        height: 80.0,
                        coordinate_space: UiMountedCoordinateSpace::HostSurface,
                    })
                    .unwrap(),
                ),
            ]),
            scale: UiGlyphRasterScale::new(1_000, layout.view().text_scale_generation()).unwrap(),
            placement: UiGlyphRasterPlacement::default(),
            lane: UiGlyphRasterLane::Ordinary,
        },
    )
    .unwrap()
}
