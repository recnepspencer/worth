use std::collections::HashSet;

use worth_ui_host_contract::{
    UiGlyphRasterAttribution, UiGlyphRasterDemandRecord, UiGlyphRasterKey,
    UiQualifiedTextLayoutIdentity, UiTextOriginalRange,
};

use super::{
    DemandBatchAdmission, UiGlyphRasterDemandBatch, UiGlyphRasterDemandDenial,
    UiGlyphRasterDemandProvenance, UiGlyphRasterDemandRequest, UiGlyphRasterDemandSelection,
    MAX_DEMAND_RECORDS,
};
use crate::raster::cost::{UiGlyphRasterLaneCost, UiGlyphRasterLaneCostInput};
use crate::raster::demand_candidate::candidate_for_positioned;
use crate::raster::demand_geometry::{contains_range, damage_intersects};
use crate::raster::demand_identity::demand_identity;
use crate::raster::planning_geometry::predicted_extent;
use crate::UiQualifiedTextLayout;

struct DerivedDemand {
    layout: UiQualifiedTextLayoutIdentity,
    scale: super::UiGlyphRasterScale,
    placement: crate::raster::placement::UiGlyphRasterPlacement,
    lane: worth_ui_host_contract::UiGlyphRasterLane,
    scope: worth_ui_host_contract::UiGlyphRasterDemandScope,
    records: Vec<UiGlyphRasterDemandRecord>,
    provenance: Vec<UiGlyphRasterDemandProvenance>,
    cost: UiGlyphRasterLaneCost,
}

#[derive(Default)]
struct DemandCounters {
    layout_visits: u32,
    demanded_glyphs: u32,
    face_resource_lookups: u32,
    unique_keys: HashSet<UiGlyphRasterKey>,
}

impl DemandCounters {
    fn cost(self) -> UiGlyphRasterLaneCost {
        UiGlyphRasterLaneCost::from_text_mechanics(UiGlyphRasterLaneCostInput {
            layout_visits: self.layout_visits,
            outer_traversals: self.layout_visits,
            validation_checks: self.demanded_glyphs,
            provenance_checks: 0,
            demanded_glyphs: self.demanded_glyphs,
            face_resource_lookups: self.face_resource_lookups,
            outline_evaluations: 0,
            bitmap_source_evaluations: 0,
            retained_scans: 0,
            cache_hits: self
                .demanded_glyphs
                .saturating_sub(u32::try_from(self.unique_keys.len()).unwrap_or(u32::MAX)),
            cache_misses: 0,
            rasterized_glyphs: 0,
            rasterized_texels: 0,
            produced_bytes: 0,
        })
    }
}

impl UiGlyphRasterDemandBatch {
    fn from_derived_records(derived: DerivedDemand) -> Result<Self, UiGlyphRasterDemandDenial> {
        let identity = demand_identity(
            derived.layout,
            derived.scale,
            derived.placement,
            derived.lane,
            derived.scope,
            &derived.records,
        );
        Self::admit_records(DemandBatchAdmission {
            identity,
            layout: derived.layout,
            scale: derived.scale,
            placement: derived.placement,
            lane: derived.lane,
            scope: derived.scope,
            records: derived.records,
            provenance: derived.provenance,
            lane_cost: derived.cost,
        })
    }
}

/// Derive exact paragraph-local raster demand from the durable qualified
/// layout and the mounted damage/paint attribution views.
pub fn derive_glyph_raster_demand(
    layout: &UiQualifiedTextLayout,
    request: UiGlyphRasterDemandRequest<'_>,
) -> Result<UiGlyphRasterDemandBatch, UiGlyphRasterDemandDenial> {
    let derived = collect_demand(layout, request)?;
    UiGlyphRasterDemandBatch::from_derived_records(derived)
}

fn collect_demand(
    layout: &UiQualifiedTextLayout,
    request: UiGlyphRasterDemandRequest<'_>,
) -> Result<DerivedDemand, UiGlyphRasterDemandDenial> {
    if request.scale.dpi_milli() == 0 {
        return Err(UiGlyphRasterDemandDenial::ZeroDpi);
    }
    if request.scale.text_scale_generation() != layout.view().text_scale_generation() {
        return Err(UiGlyphRasterDemandDenial::LayoutScaleMismatch);
    }
    let (records, provenance, counters) = collect_demand_records(layout, request)?;
    Ok(DerivedDemand {
        layout: layout.identity(),
        scale: request.scale,
        placement: request.placement,
        lane: request.lane,
        scope: request.selection.scope(),
        records,
        provenance,
        cost: counters.cost(),
    })
}

fn collect_demand_records(
    layout: &UiQualifiedTextLayout,
    request: UiGlyphRasterDemandRequest<'_>,
) -> Result<
    (
        Vec<UiGlyphRasterDemandRecord>,
        Vec<UiGlyphRasterDemandProvenance>,
        DemandCounters,
    ),
    UiGlyphRasterDemandDenial,
> {
    let layout_identity = layout.identity();
    let mut records = Vec::new();
    let mut provenance = Vec::new();
    let mut counters = DemandCounters::default();
    for (positioned_index, positioned) in layout.positioned_glyphs().iter().enumerate() {
        counters.layout_visits = counters.layout_visits.saturating_add(1);
        if let UiGlyphRasterDemandSelection::LogicalDamage(damage) = request.selection {
            if !damage_intersects(positioned.ink_bounds(), request.placement, damage) {
                continue;
            }
        }
        let Some(candidate) =
            candidate_for_positioned(layout, positioned_index, request.scale, request.placement)?
        else {
            continue;
        };
        let span_count = request
            .paint_spans
            .iter()
            .filter(|span| contains_range(span.original_range(), candidate.original_range))
            .count();
        if span_count != 1 {
            return Err(UiGlyphRasterDemandDenial::PaintSpanMismatch);
        }
        counters.face_resource_lookups = counters.face_resource_lookups.saturating_add(1);
        counters.demanded_glyphs = counters.demanded_glyphs.saturating_add(1);
        if records.len() == MAX_DEMAND_RECORDS {
            return Err(UiGlyphRasterDemandDenial::DemandCapacityExceeded);
        }
        let extent = predicted_extent(layout, &candidate)
            .map_err(|_| UiGlyphRasterDemandDenial::RasterGeometryUnavailable)?;
        records.push(demand_record(
            candidate.key,
            layout_identity,
            candidate.original_range,
            extent,
        ));
        provenance.push(UiGlyphRasterDemandProvenance {
            positioned_glyph_index: candidate.positioned_glyph_index,
            glyph_index: candidate.glyph_index,
        });
        counters.unique_keys.insert(candidate.key);
    }
    Ok((records, provenance, counters))
}

fn demand_record(
    key: UiGlyphRasterKey,
    layout: UiQualifiedTextLayoutIdentity,
    original_range: UiTextOriginalRange,
    extent: worth_ui_host_contract::UiGlyphRasterExtent,
) -> UiGlyphRasterDemandRecord {
    UiGlyphRasterDemandRecord::from_text_mechanics(
        key,
        UiGlyphRasterAttribution::from_text_mechanics(layout, original_range),
        extent,
    )
    .expect("qualified raster geometry has bounded staged bytes")
}
