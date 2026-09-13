use super::geometry::project_clipped_region;
use super::record::{UiHitTestRegionRecord, UiVisibleRegionRecord};
use super::{UiHitTestRegionIndex, UiVisibleRegionIndex};
use std::collections::BTreeMap;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum UiSpatialValidationDenial {
    ProtocolMismatch,
    InvalidGeometry,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct UiSpatialIndexBuildCost {
    region_records_examined: usize,
    retained_structural_bytes: usize,
}

pub(crate) struct UiValidatedSpatialIndexes {
    visible: UiVisibleRegionIndex,
    hit_test: UiHitTestRegionIndex,
    cost: UiSpatialIndexBuildCost,
}

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
struct UiHostRowKey([u64; 14]);

enum UiExpectedPaintRow {
    Appearance(crate::mounting::UiMountedAppearancePaintBasis),
    Unsupported(crate::mounting::UiMountedUnsupportedPaintBasis),
}

pub(crate) fn validate_and_index(
    capture_identity: u64,
    expected: &crate::mounting::UiMountedVisualRegionBasis,
    observed: &[worth_ui_host_contract::UiHostRealizedRegion],
    transform: worth_ui_host_contract::UiHostCoordinateTransform,
) -> Result<UiValidatedSpatialIndexes, UiSpatialValidationDenial> {
    let expected_appearance_rows = expected.appearance_paint();
    let expected_unsupported_rows = expected.unsupported_paint();
    let expected_hit_rows = expected.hit_test();
    if expected_appearance_rows.len() + expected_unsupported_rows.len() + expected_hit_rows.len()
        != observed.len()
    {
        return Err(UiSpatialValidationDenial::ProtocolMismatch);
    }
    let mut expected_paint =
        paint_rows_by_key(&expected_appearance_rows, &expected_unsupported_rows);
    let expected_hit_mechanics = expected_hit_rows
        .iter()
        .map(crate::mounting::UiMountedHitTestPresentation::mechanic)
        .collect::<Vec<_>>();
    let mut expected_hit = hit_rows_by_key(&expected_hit_mechanics);
    let mut visible =
        Vec::with_capacity(expected_appearance_rows.len() + expected_unsupported_rows.len());
    let mut hit_test = Vec::with_capacity(expected_hit_rows.len());
    for region in observed {
        match region.participation() {
            worth_ui_host_contract::UiHostRealizedRegionParticipation::Paint => {
                let mechanic = take_matching(&mut expected_paint, observed_key(*region))?;
                if let Some(projected) =
                    project_clipped_region(region.bounds(), region.clip(), transform)
                        .map_err(|_| UiSpatialValidationDenial::InvalidGeometry)?
                {
                    visible.push(match mechanic {
                        UiExpectedPaintRow::Appearance(mechanic) => {
                            UiVisibleRegionRecord::appearance(mechanic, region.clip(), projected)
                        }
                        UiExpectedPaintRow::Unsupported(mechanic) => {
                            UiVisibleRegionRecord::unsupported(mechanic, region.clip(), projected)
                        }
                    });
                }
            }
            worth_ui_host_contract::UiHostRealizedRegionParticipation::HitTest => {
                let mechanic = take_matching(&mut expected_hit, observed_key(*region))?;
                if let Some(projected) =
                    project_clipped_region(region.bounds(), region.clip(), transform)
                        .map_err(|_| UiSpatialValidationDenial::InvalidGeometry)?
                {
                    hit_test.push(UiHitTestRegionRecord::validated(mechanic, projected));
                }
            }
        }
    }
    if expected_paint.values().any(|rows| !rows.is_empty())
        || expected_hit.values().any(|rows| !rows.is_empty())
    {
        return Err(UiSpatialValidationDenial::ProtocolMismatch);
    }
    UiValidatedSpatialIndexes::build(capture_identity, visible, hit_test, observed.len())
}

fn paint_rows_by_key(
    appearance: &[crate::mounting::UiMountedAppearancePaintBasis],
    unsupported: &[crate::mounting::UiMountedUnsupportedPaintBasis],
) -> BTreeMap<UiHostRowKey, Vec<UiExpectedPaintRow>> {
    let mut rows = BTreeMap::<_, Vec<_>>::new();
    for row in appearance {
        rows.entry(row_key(
            row.node_receipt(),
            row.bounds(),
            row.clip(),
            row.semantic_order(),
        ))
        .or_default()
        .push(UiExpectedPaintRow::Appearance(*row));
    }
    for row in unsupported {
        rows.entry(row_key(
            row.node_receipt(),
            row.bounds(),
            row.clip(),
            row.semantic_order(),
        ))
        .or_default()
        .push(UiExpectedPaintRow::Unsupported(*row));
    }
    rows
}

fn hit_rows_by_key(
    expected: &[worth_ui_host_contract::UiMountedHitTestMechanic],
) -> BTreeMap<UiHostRowKey, Vec<worth_ui_host_contract::UiMountedHitTestMechanic>> {
    let mut rows = BTreeMap::<_, Vec<_>>::new();
    for row in expected {
        rows.entry(row_key(
            row.node_receipt(),
            row.bounds(),
            row.clip_bounds(),
            row.order().rank(),
        ))
        .or_default()
        .push(*row);
    }
    rows
}

fn take_matching<Row>(
    expected: &mut BTreeMap<UiHostRowKey, Vec<Row>>,
    key: UiHostRowKey,
) -> Result<Row, UiSpatialValidationDenial> {
    expected
        .get_mut(&key)
        .and_then(Vec::pop)
        .ok_or(UiSpatialValidationDenial::ProtocolMismatch)
}

fn observed_key(region: worth_ui_host_contract::UiHostRealizedRegion) -> UiHostRowKey {
    row_key(
        region.mounted_receipt(),
        region.bounds(),
        region.clip(),
        region.semantic_order(),
    )
}

fn row_key(
    receipt: worth_ui_host_contract::UiMountedNodeReceiptIdentity,
    bounds: worth_ui_host_contract::UiMountedCanonicalBox,
    clip: worth_ui_host_contract::UiMountedCanonicalBox,
    order: u32,
) -> UiHostRowKey {
    UiHostRowKey([
        receipt.diagnostic_value(),
        u64::from(bounds.x().to_bits()),
        u64::from(bounds.y().to_bits()),
        u64::from(bounds.width().to_bits()),
        u64::from(bounds.height().to_bits()),
        coordinate_space_key(bounds.coordinate_space()),
        posture_key(bounds.posture()),
        u64::from(clip.x().to_bits()),
        u64::from(clip.y().to_bits()),
        u64::from(clip.width().to_bits()),
        u64::from(clip.height().to_bits()),
        coordinate_space_key(clip.coordinate_space()),
        posture_key(clip.posture()),
        u64::from(order),
    ])
}

fn coordinate_space_key(space: worth_ui_host_contract::UiMountedCoordinateSpace) -> u64 {
    match space {
        worth_ui_host_contract::UiMountedCoordinateSpace::Viewport => 1,
        worth_ui_host_contract::UiMountedCoordinateSpace::Window => 2,
        worth_ui_host_contract::UiMountedCoordinateSpace::GraphNodeLocal => 3,
        worth_ui_host_contract::UiMountedCoordinateSpace::HostSurface => 4,
        worth_ui_host_contract::UiMountedCoordinateSpace::PortalLayer => 5,
    }
}

fn posture_key(posture: worth_ui_host_contract::UiMountedGeometryPosture) -> u64 {
    match posture {
        worth_ui_host_contract::UiMountedGeometryPosture::Area => 1,
        worth_ui_host_contract::UiMountedGeometryPosture::Empty => 2,
        worth_ui_host_contract::UiMountedGeometryPosture::Offscreen => 3,
    }
}

impl UiValidatedSpatialIndexes {
    fn build(
        capture_identity: u64,
        visible_records: Vec<UiVisibleRegionRecord>,
        hit_test_records: Vec<UiHitTestRegionRecord>,
        region_records_examined: usize,
    ) -> Result<Self, UiSpatialValidationDenial> {
        let visible = UiVisibleRegionIndex::build(capture_identity, visible_records);
        let hit_test = UiHitTestRegionIndex::build(capture_identity, hit_test_records);
        let retained_structural_bytes = visible
            .retained_structural_bytes()
            .and_then(|bytes| bytes.checked_add(hit_test.retained_structural_bytes()?))
            .ok_or(UiSpatialValidationDenial::InvalidGeometry)?;
        Ok(Self {
            visible,
            hit_test,
            cost: UiSpatialIndexBuildCost {
                region_records_examined,
                retained_structural_bytes,
            },
        })
    }

    pub(crate) fn into_parts(
        self,
    ) -> (
        UiVisibleRegionIndex,
        UiHitTestRegionIndex,
        UiSpatialIndexBuildCost,
    ) {
        (self.visible, self.hit_test, self.cost)
    }
}

impl UiSpatialIndexBuildCost {
    pub(crate) const fn region_records_examined(self) -> usize {
        self.region_records_examined
    }

    #[cfg(test)]
    pub(crate) const fn retained_structural_bytes(self) -> usize {
        self.retained_structural_bytes
    }
}
