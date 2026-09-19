//! Text-owned glyph-raster demand derivation.
//!
//! Demand is derived from the qualified layout and mounted damage. Runtime
//! selects the layout and supplies inert paint/damage views; it cannot mint a
//! key or reinterpret a glyph. The private provenance beside each borrowed
//! record lets raster admission validate only the selected paragraph-local
//! records.

use worth_ui_host_contract::{
    UiGlyphRasterDemandBatchView, UiGlyphRasterDemandBatchViewInput, UiGlyphRasterDemandIdentity,
    UiGlyphRasterDemandRecord, UiGlyphRasterDemandScope, UiGlyphRasterLane, UiMountedLogicalDamage,
    UiMountedTextForegroundSpan, UiQualifiedTextLayoutIdentity, UiTextScaleGeneration,
};

use super::cost::UiGlyphRasterLaneCost;
#[cfg(test)]
use super::demand_identity::demand_identity;
use super::key::UiGlyphRasterKeyDenial;
use super::placement::UiGlyphRasterPlacement;
use crate::{UiGlobalTextProfile, UiQualifiedTextLayout};

#[path = "demand/derivation.rs"]
mod derivation;

pub use derivation::derive_glyph_raster_demand;

const MAX_DEMAND_RECORDS: usize = UiGlobalTextProfile::MAX_GLYPHS;

struct DemandBatchAdmission {
    identity: UiGlyphRasterDemandIdentity,
    layout: UiQualifiedTextLayoutIdentity,
    scale: UiGlyphRasterScale,
    placement: UiGlyphRasterPlacement,
    lane: UiGlyphRasterLane,
    scope: UiGlyphRasterDemandScope,
    records: Vec<UiGlyphRasterDemandRecord>,
    provenance: Vec<UiGlyphRasterDemandProvenance>,
    lane_cost: UiGlyphRasterLaneCost,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct UiGlyphRasterScale {
    dpi_milli: u32,
    text_scale: UiTextScaleGeneration,
}

/// Select complete layout coverage or only glyphs intersecting mounted damage.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum UiGlyphRasterDemandSelection<'a> {
    CompleteLayout,
    LogicalDamage(&'a [UiMountedLogicalDamage]),
}

impl UiGlyphRasterDemandSelection<'_> {
    pub const fn scope(self) -> UiGlyphRasterDemandScope {
        match self {
            Self::CompleteLayout => UiGlyphRasterDemandScope::CompleteLayout,
            Self::LogicalDamage(_) => UiGlyphRasterDemandScope::DamageFiltered,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct UiGlyphRasterDemandRequest<'a> {
    pub paint_spans: &'a [UiMountedTextForegroundSpan],
    pub selection: UiGlyphRasterDemandSelection<'a>,
    pub scale: UiGlyphRasterScale,
    pub placement: UiGlyphRasterPlacement,
    pub lane: UiGlyphRasterLane,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum UiGlyphRasterDemandDenial {
    Key(UiGlyphRasterKeyDenial),
    ZeroDpi,
    ForeignLayout,
    ForeignKey,
    LayoutScaleMismatch,
    ForeignCollectionLineage,
    DemandIdentityMismatch,
    DemandCapacityExceeded,
    PaintSpanMismatch,
    MissingCoverage,
    ForeignFace,
    OriginOverflow,
    RasterGeometryUnavailable,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct UiGlyphRasterDemandProvenance {
    pub(crate) positioned_glyph_index: usize,
    pub(crate) glyph_index: usize,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct UiGlyphRasterDemandBatch {
    identity: UiGlyphRasterDemandIdentity,
    layout: UiQualifiedTextLayoutIdentity,
    scale: UiGlyphRasterScale,
    placement: UiGlyphRasterPlacement,
    lane: UiGlyphRasterLane,
    scope: UiGlyphRasterDemandScope,
    records: Box<[UiGlyphRasterDemandRecord]>,
    provenance: Box<[UiGlyphRasterDemandProvenance]>,
    cost: super::UiGlyphRasterCost,
}

impl UiGlyphRasterScale {
    pub const fn new(dpi_milli: u32, text_scale: UiTextScaleGeneration) -> Option<Self> {
        if dpi_milli == 0 {
            None
        } else {
            Some(Self {
                dpi_milli,
                text_scale,
            })
        }
    }

    pub const fn dpi_milli(self) -> u32 {
        self.dpi_milli
    }

    pub const fn text_scale_generation(self) -> UiTextScaleGeneration {
        self.text_scale
    }
}

impl UiGlyphRasterDemandBatch {
    #[cfg(test)]
    pub(crate) fn from_text_mechanics(
        identity: UiGlyphRasterDemandIdentity,
        layout: UiQualifiedTextLayoutIdentity,
        scale: UiGlyphRasterScale,
        placement: UiGlyphRasterPlacement,
        lane: UiGlyphRasterLane,
        scope: UiGlyphRasterDemandScope,
        records: impl IntoIterator<Item = UiGlyphRasterDemandRecord>,
    ) -> Result<Self, UiGlyphRasterDemandDenial> {
        let records: Vec<_> = records.into_iter().collect();
        let expected = demand_identity(layout, scale, placement, lane, scope, &records);
        if identity != expected {
            return Err(UiGlyphRasterDemandDenial::DemandIdentityMismatch);
        }
        Self::admit_records(DemandBatchAdmission {
            identity,
            layout,
            scale,
            placement,
            lane,
            scope,
            records,
            provenance: Vec::new(),
            lane_cost: Default::default(),
        })
    }

    fn admit_records(input: DemandBatchAdmission) -> Result<Self, UiGlyphRasterDemandDenial> {
        let DemandBatchAdmission {
            identity,
            layout,
            scale,
            placement,
            lane,
            scope,
            records,
            provenance,
            lane_cost,
        } = input;
        if records.len() > MAX_DEMAND_RECORDS {
            return Err(UiGlyphRasterDemandDenial::DemandCapacityExceeded);
        }
        if !provenance.is_empty() && provenance.len() != records.len() {
            return Err(UiGlyphRasterDemandDenial::DemandIdentityMismatch);
        }
        if records.iter().any(|record| {
            record.attribution().layout() != layout || record.key().dpi_milli() != scale.dpi_milli()
        }) {
            return Err(UiGlyphRasterDemandDenial::ForeignKey);
        }
        let cost = match lane {
            UiGlyphRasterLane::Ordinary => {
                super::UiGlyphRasterCost::from_text_mechanics(lane_cost, Default::default())
            }
            UiGlyphRasterLane::Reconstruction => {
                super::UiGlyphRasterCost::from_text_mechanics(Default::default(), lane_cost)
            }
        };
        Ok(Self {
            identity,
            layout,
            scale,
            placement,
            lane,
            scope,
            records: records.into_boxed_slice(),
            provenance: provenance.into_boxed_slice(),
            cost,
        })
    }

    pub const fn identity(&self) -> UiGlyphRasterDemandIdentity {
        self.identity
    }

    pub const fn layout_identity(&self) -> UiQualifiedTextLayoutIdentity {
        self.layout
    }

    pub const fn scale(&self) -> UiGlyphRasterScale {
        self.scale
    }

    pub const fn placement(&self) -> UiGlyphRasterPlacement {
        self.placement
    }

    /// Return the exact positioned layout record that produced one admitted
    /// demand record.  Consumers cannot rediscover this association from the
    /// raster key or original range because repeated glyphs may share both.
    pub fn positioned_glyph_for_record(
        &self,
        layout: &UiQualifiedTextLayout,
        record_index: usize,
    ) -> Option<worth_ui_host_contract::UiPositionedTextGlyphRecord> {
        if layout.identity() != self.layout {
            return None;
        }
        let provenance = self.provenance.get(record_index)?;
        layout
            .positioned_glyphs()
            .get(provenance.positioned_glyph_index)
            .copied()
    }

    pub const fn lane(&self) -> UiGlyphRasterLane {
        self.lane
    }

    pub const fn scope(&self) -> UiGlyphRasterDemandScope {
        self.scope
    }

    pub fn records(&self) -> &[UiGlyphRasterDemandRecord] {
        &self.records
    }

    pub(crate) fn provenance(&self) -> &[UiGlyphRasterDemandProvenance] {
        &self.provenance
    }

    pub const fn cost(&self) -> super::UiGlyphRasterCost {
        self.cost
    }

    pub fn as_view(&self) -> UiGlyphRasterDemandBatchView<'_> {
        UiGlyphRasterDemandBatchView::from_text_mechanics(UiGlyphRasterDemandBatchViewInput {
            identity: self.identity,
            layout: self.layout,
            dpi_milli: self.scale.dpi_milli,
            text_scale: self.scale.text_scale,
            lane: self.lane,
            scope: self.scope,
            records: &self.records,
        })
        .expect("admitted demand preserves a nonzero DPI")
    }
}
