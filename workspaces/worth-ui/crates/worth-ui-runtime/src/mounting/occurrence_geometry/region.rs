use std::collections::{BTreeMap, BTreeSet};
use worth_ui_dsl::UiMosaicRegionDeclarationIdentity;
use worth_ui_host_contract::{UiMountedCanonicalBox, UiMountedInstanceIdentity};

/// One executed Mosaic region's completed rectangle, relative to the exact
/// mounted owner occurrence. The completion boundary validates the executed
/// region identity against that owner's current admitted layout.
#[derive(Clone, Debug, PartialEq)]
pub struct UiMountedMosaicRegionGeometry {
    owner: UiMountedInstanceIdentity,
    declaration: UiMosaicRegionDeclarationIdentity,
    executed_region: Box<str>,
    bounds: UiMountedCanonicalBox,
}

pub(super) fn complete_regions(
    batch: &super::UiMountedSurfaceGeometryBatch,
    resolved: &BTreeMap<UiMountedInstanceIdentity, UiMountedCanonicalBox>,
    identity: &crate::mounting::UiMountedIdentityState,
) -> Result<
    (
        BTreeMap<
            UiMosaicRegionDeclarationIdentity,
            Vec<(
                UiMountedInstanceIdentity,
                worth_ui_host_contract::UiMountIncarnation,
                UiMountedCanonicalBox,
            )>,
        >,
        BTreeMap<
            (UiMountedInstanceIdentity, UiMosaicRegionDeclarationIdentity),
            UiMountedCanonicalBox,
        >,
        BTreeMap<UiMountedInstanceIdentity, super::UiMountedSurfacePaintPosture>,
        usize,
        usize,
    ),
    super::UiMountedOccurrenceGeometryDenial,
> {
    use super::UiMountedOccurrenceGeometryDenial as Denial;
    use worth_ui_host_contract::{UiMountedCanonicalBoxInput, UiMountedCoordinateSpace};
    let mut seen = BTreeSet::new();
    let mut regions = BTreeMap::<_, Vec<_>>::new();
    let mut exact_regions = BTreeMap::new();
    let mut paint_regions = Vec::new();
    for region in batch.regions() {
        let parent = resolved
            .get(&region.owner())
            .ok_or(Denial::UnknownMountedInstance)?;
        if !seen.insert((region.owner(), region.executed_region())) {
            return Err(Denial::DuplicateRegionOccurrence);
        }
        let local = region.bounds();
        if local.coordinate_space() != UiMountedCoordinateSpace::GraphNodeLocal {
            return Err(Denial::ParentCoordinateSpaceMismatch);
        }
        let bounds = UiMountedCanonicalBox::canonicalize(UiMountedCanonicalBoxInput {
            x: parent.x() + local.x(),
            y: parent.y() + local.y(),
            width: local.width(),
            height: local.height(),
            coordinate_space: UiMountedCoordinateSpace::HostSurface,
        })
        .map_err(|_| Denial::ParentCoordinateSpaceMismatch)?;
        let incarnation = identity
            .projection_instance(region.owner())
            .ok_or(Denial::UnknownMountedInstance)?
            .mount_incarnation();
        if exact_regions
            .insert((region.owner(), region.declaration()), bounds)
            .is_some()
        {
            return Err(Denial::AmbiguousMosaicRegionGeometry);
        }
        if let Some(seam) = batch.seam_paint() {
            let kind = seam
                .region_kind(region.owner(), region.declaration())
                .ok_or(Denial::UnknownExecutedRegion)?;
            paint_regions.push(super::region_paint::UiMountedRegionPaintInput::new(
                region.owner(),
                region.declaration(),
                kind.clone(),
                bounds,
                *parent,
            ));
        }
        regions.entry(region.declaration()).or_default().push((
            region.owner(),
            incarnation,
            bounds,
        ));
    }
    let (paint_postures, seam_index_rows, seam_adjacencies_visited) =
        super::region_paint::derive(batch, &paint_regions)?.into_parts();
    Ok((
        regions,
        exact_regions,
        paint_postures,
        seam_index_rows,
        seam_adjacencies_visited,
    ))
}

impl UiMountedMosaicRegionGeometry {
    pub fn new(
        owner: UiMountedInstanceIdentity,
        declaration: UiMosaicRegionDeclarationIdentity,
        executed_region: impl Into<Box<str>>,
        bounds: UiMountedCanonicalBox,
    ) -> Self {
        Self {
            owner,
            declaration,
            executed_region: executed_region.into(),
            bounds,
        }
    }
    pub fn owner(&self) -> UiMountedInstanceIdentity {
        self.owner
    }
    pub fn declaration(&self) -> UiMosaicRegionDeclarationIdentity {
        self.declaration
    }
    pub fn executed_region(&self) -> &str {
        &self.executed_region
    }
    pub fn bounds(&self) -> UiMountedCanonicalBox {
        self.bounds
    }
}
