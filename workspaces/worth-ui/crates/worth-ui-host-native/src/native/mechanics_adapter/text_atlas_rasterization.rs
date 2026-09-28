//! Scoped miss grouping and raster callback crossing for admitted atlas work.

use std::collections::BTreeMap;

use worth_ui_host_contract::{
    UiGlyphRasterCallbackDenial, UiGlyphRasterDemandIdentity, UiGlyphRasterLane,
    UiGlyphRasterMissRasterizer, UiGlyphRasterMissSelectionView, UiGlyphRasterTransactionDenial,
};

use crate::native::text_atlas::{
    canonical_raster_key_bytes, UiNativeTextAtlasTransactionPlan, UiNativeTextAtlasUpload,
};

pub(super) fn rasterize_misses(
    plan: &UiNativeTextAtlasTransactionPlan,
    rasterizer: &mut dyn UiGlyphRasterMissRasterizer,
) -> Result<Vec<UiNativeTextAtlasUpload>, UiGlyphRasterTransactionDenial> {
    let mut uploads = Vec::new();
    for group in miss_groups(plan)? {
        let mut sink = super::text_atlas_upload_sink::UploadSink::new(&group);
        let selection = UiGlyphRasterMissSelectionView::from_text_mechanics(
            group.demand,
            group.layout,
            group.lane,
            &group.records,
        );
        rasterizer
            .rasterize(selection, &mut sink)
            .map_err(callback_denial)?;
        sink.finish()?;
        uploads.extend(sink.uploads);
    }
    Ok(uploads)
}

pub(super) struct MissGroup {
    pub(super) demand: UiGlyphRasterDemandIdentity,
    pub(super) layout: worth_ui_host_contract::UiQualifiedTextLayoutIdentity,
    pub(super) lane: UiGlyphRasterLane,
    pub(super) dpi_milli: u32,
    pub(super) text_scale: worth_ui_host_contract::UiTextScaleGeneration,
    pub(super) records: Box<[worth_ui_host_contract::UiGlyphRasterDemandRecord]>,
}

fn miss_groups(
    plan: &UiNativeTextAtlasTransactionPlan,
) -> Result<Vec<MissGroup>, UiGlyphRasterTransactionDenial> {
    let mut groups: BTreeMap<([u8; 32], [u8; 32], u8, u32, u64), Vec<_>> = BTreeMap::new();
    for demand in plan.miss_demands() {
        let record = demand
            .source_record()
            .ok_or(UiGlyphRasterTransactionDenial::MalformedDemand)?;
        let key = (
            demand.source_identity().digest(),
            demand.source_layout().digest(),
            lane_byte(demand.source_lane()),
            demand.source_dpi_milli(),
            demand.source_text_scale().get(),
        );
        groups.entry(key).or_default().push(record);
    }
    Ok(groups
        .into_iter()
        .map(
            |((demand, layout, lane, dpi_milli, text_scale), mut records)| {
                records.sort_by_cached_key(|record| canonical_raster_key_bytes(record.key()));
                MissGroup {
                    demand: UiGlyphRasterDemandIdentity::from_text_mechanics(demand),
                    layout:
                        worth_ui_host_contract::UiQualifiedTextLayoutIdentity::from_text_mechanics(
                            layout,
                        ),
                    lane: if lane == 0 {
                        UiGlyphRasterLane::Ordinary
                    } else {
                        UiGlyphRasterLane::Reconstruction
                    },
                    dpi_milli,
                    text_scale: worth_ui_host_contract::UiTextScaleGeneration::new(text_scale)
                        .expect("grouped demand retains nonzero text scale"),
                    records: records.into_boxed_slice(),
                }
            },
        )
        .collect())
}

fn lane_byte(lane: UiGlyphRasterLane) -> u8 {
    match lane {
        UiGlyphRasterLane::Ordinary => 0,
        UiGlyphRasterLane::Reconstruction => 1,
    }
}

fn callback_denial(denial: UiGlyphRasterCallbackDenial) -> UiGlyphRasterTransactionDenial {
    match denial {
        UiGlyphRasterCallbackDenial::BatchRejected(_) => {
            UiGlyphRasterTransactionDenial::RasterBatchMismatch
        }
        UiGlyphRasterCallbackDenial::DemandMismatch
        | UiGlyphRasterCallbackDenial::RasterizationDenied
        | UiGlyphRasterCallbackDenial::Rejected => UiGlyphRasterTransactionDenial::CallbackRejected,
    }
}
