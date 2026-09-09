use std::collections::BTreeMap;

use worth_ui_host_contract::{
    UiMountedCanonicalBox, UiMountedInstanceIdentity, UiMountedSurfaceBorderEdges,
};

use super::{
    UiMountedOccurrenceGeometryDenial as Denial, UiMountedSurfaceGeometryBatch,
    UiMountedSurfacePaintPosture,
};

mod boundary_index;

#[derive(Clone, Copy)]
enum Edge {
    Top,
    Right,
    Bottom,
    Left,
}

struct PaintRow {
    owner: UiMountedInstanceIdentity,
    declaration: worth_ui_dsl::UiMosaicRegionDeclarationIdentity,
    kind: crate::capability::MosaicRegionKindId,
    bounds: UiMountedCanonicalBox,
    surface_bounds: UiMountedCanonicalBox,
    edges: [bool; 4],
    omissions: [Vec<(f32, f32)>; 4],
}

pub(super) struct UiMountedRegionPaintInput {
    owner: UiMountedInstanceIdentity,
    declaration: worth_ui_dsl::UiMosaicRegionDeclarationIdentity,
    kind: crate::capability::MosaicRegionKindId,
    bounds: UiMountedCanonicalBox,
    surface_bounds: UiMountedCanonicalBox,
}

impl UiMountedRegionPaintInput {
    pub(super) fn new(
        owner: UiMountedInstanceIdentity,
        declaration: worth_ui_dsl::UiMosaicRegionDeclarationIdentity,
        kind: crate::capability::MosaicRegionKindId,
        bounds: UiMountedCanonicalBox,
        surface_bounds: UiMountedCanonicalBox,
    ) -> Self {
        Self {
            owner,
            declaration,
            kind,
            bounds,
            surface_bounds,
        }
    }
}

pub(super) struct UiMountedRegionPaintCompletion {
    postures: BTreeMap<UiMountedInstanceIdentity, UiMountedSurfacePaintPosture>,
    index_rows: usize,
    adjacencies_visited: usize,
}

#[derive(Clone, Copy)]
struct SharedBoundary {
    before: usize,
    before_edge: Edge,
    after: usize,
    after_edge: Edge,
    start: f32,
    end: f32,
}

pub(super) fn derive(
    batch: &UiMountedSurfaceGeometryBatch,
    regions: &[UiMountedRegionPaintInput],
) -> Result<UiMountedRegionPaintCompletion, Denial> {
    let Some(seam) = batch.seam_paint() else {
        return Ok(UiMountedRegionPaintCompletion {
            postures: BTreeMap::new(),
            index_rows: 0,
            adjacencies_visited: 0,
        });
    };
    derive_contract(seam.contract(), regions)
}

fn derive_contract(
    contract: &crate::capability::MosaicSeamPaintContract,
    regions: &[UiMountedRegionPaintInput],
) -> Result<UiMountedRegionPaintCompletion, Denial> {
    let mut rows = regions
        .iter()
        .map(|region| PaintRow {
            owner: region.owner,
            declaration: region.declaration,
            kind: region.kind.clone(),
            bounds: region.bounds,
            surface_bounds: region.surface_bounds,
            edges: [true; 4],
            omissions: std::array::from_fn(|_| Vec::new()),
        })
        .collect::<Vec<_>>();
    rows.sort_by_key(|row| (row.owner, row.declaration));
    let (boundaries, index_rows) = boundary_index::shared_boundaries(&rows);
    for boundary in &boundaries {
        if surface_edge_basis(&rows[boundary.before], boundary.before_edge).is_none()
            || surface_edge_basis(&rows[boundary.after], boundary.after_edge).is_none()
        {
            continue;
        }
        let owner = contract
            .paint_owner_for(&rows[boundary.before].kind, &rows[boundary.after].kind)
            .ok_or(Denial::UndeclaredMosaicSharedEdge)?;
        if owner == &rows[boundary.before].kind {
            omit_boundary(
                &mut rows[boundary.after],
                boundary.after_edge,
                boundary.start,
                boundary.end,
            );
        } else {
            omit_boundary(
                &mut rows[boundary.before],
                boundary.before_edge,
                boundary.start,
                boundary.end,
            );
        }
    }
    let postures = aggregate_owner_postures(contract, rows);
    Ok(UiMountedRegionPaintCompletion {
        postures,
        index_rows,
        adjacencies_visited: boundaries.len(),
    })
}

fn omit_boundary(row: &mut PaintRow, edge: Edge, start: f32, end: f32) {
    let Some((origin, extent)) = surface_edge_basis(row, edge) else {
        return;
    };
    let relative = ((start - origin).max(0.0), (end - origin).min(extent));
    if relative.0 >= relative.1 {
        return;
    }
    let edge_index = index(edge);
    if relative.0 <= 0.0 && relative.1 >= extent {
        row.edges[edge_index] = false;
        row.omissions[edge_index].clear();
    } else if row.edges[edge_index] {
        row.omissions[edge_index].push(relative);
    }
}

fn surface_edge_basis(row: &PaintRow, edge: Edge) -> Option<(f32, f32)> {
    let region = row.bounds;
    let surface = row.surface_bounds;
    match edge {
        Edge::Top if region.y() == surface.y() => Some((surface.x(), surface.width())),
        Edge::Right if region.x() + region.width() == surface.x() + surface.width() => {
            Some((surface.y(), surface.height()))
        }
        Edge::Bottom if region.y() + region.height() == surface.y() + surface.height() => {
            Some((surface.x(), surface.width()))
        }
        Edge::Left if region.x() == surface.x() => Some((surface.y(), surface.height())),
        _ => None,
    }
}

#[derive(Default)]
struct OwnerPaintPosture {
    rows: usize,
    edges: [bool; 4],
    omissions: [Vec<(f32, f32)>; 4],
    corners: [bool; 4],
}

fn aggregate_owner_postures(
    contract: &crate::capability::MosaicSeamPaintContract,
    rows: Vec<PaintRow>,
) -> BTreeMap<UiMountedInstanceIdentity, UiMountedSurfacePaintPosture> {
    let mut owners = BTreeMap::<UiMountedInstanceIdentity, OwnerPaintPosture>::new();
    for row in rows {
        let owner = owners.entry(row.owner).or_default();
        if owner.rows == 0 {
            owner.edges = [true; 4];
        }
        owner.rows += 1;
        for edge in [Edge::Top, Edge::Right, Edge::Bottom, Edge::Left] {
            let edge_index = index(edge);
            if !row.edges[edge_index] {
                owner.edges[edge_index] = false;
                owner.omissions[edge_index].clear();
            } else if owner.edges[edge_index] {
                owner.omissions[edge_index].extend(row.omissions[edge_index].iter().copied());
            }
        }
        let corners = surface_corners(contract, &row);
        for (target, source) in owner.corners.iter_mut().zip(corners) {
            *target |= source;
        }
    }
    owners
        .into_iter()
        .map(|(owner, posture)| {
            let edges = UiMountedSurfaceBorderEdges::from_runtime_mosaic(
                posture.edges[0],
                posture.edges[1],
                posture.edges[2],
                posture.edges[3],
            );
            (
                owner,
                UiMountedSurfacePaintPosture::mosaic(
                    edges,
                    canonical_omissions(posture.omissions),
                    posture.corners,
                ),
            )
        })
        .collect()
}

fn canonical_omissions(
    omissions: [Vec<(f32, f32)>; 4],
) -> Box<[super::UiMountedCanonicalBorderOmission]> {
    use worth_ui_host_contract::UiMountedSurfaceBorderSide as Side;
    let mut output = Vec::new();
    for (edge, side) in [
        (Edge::Top, Side::Top),
        (Edge::Right, Side::Right),
        (Edge::Bottom, Side::Bottom),
        (Edge::Left, Side::Left),
    ] {
        let mut spans = omissions[index(edge)].clone();
        spans.sort_by(|left, right| left.0.total_cmp(&right.0).then(left.1.total_cmp(&right.1)));
        let mut merged = Vec::<(f32, f32)>::new();
        for span in spans {
            if let Some(last) = merged.last_mut().filter(|last| span.0 <= last.1) {
                last.1 = last.1.max(span.1);
            } else {
                merged.push(span);
            }
        }
        output.extend(
            merged
                .into_iter()
                .map(|(start, end)| super::UiMountedCanonicalBorderOmission::new(side, start, end)),
        );
    }
    output.into_boxed_slice()
}

impl UiMountedRegionPaintCompletion {
    pub(super) fn into_parts(
        self,
    ) -> (
        BTreeMap<UiMountedInstanceIdentity, UiMountedSurfacePaintPosture>,
        usize,
        usize,
    ) {
        (self.postures, self.index_rows, self.adjacencies_visited)
    }
}

#[cfg(test)]
mod tests;

const fn index(edge: Edge) -> usize {
    match edge {
        Edge::Top => 0,
        Edge::Right => 1,
        Edge::Bottom => 2,
        Edge::Left => 3,
    }
}

fn surface_corners(
    contract: &crate::capability::MosaicSeamPaintContract,
    row: &PaintRow,
) -> [bool; 4] {
    use crate::capability::MosaicExteriorCornerPosture as Corner;
    let mut corners = [false; 4];
    for corner in contract
        .exterior_corners()
        .iter()
        .filter(|corner| corner.region() == &row.kind)
    {
        let index = match corner.posture() {
            Corner::TopLeft => 0,
            Corner::TopRight => 1,
            Corner::BottomRight => 2,
            Corner::BottomLeft => 3,
        };
        if region_corner_matches_surface(row, index) {
            corners[index] = true;
        }
    }
    corners
}

fn region_corner_matches_surface(row: &PaintRow, corner: usize) -> bool {
    let region = row.bounds;
    let surface = row.surface_bounds;
    let region_right = region.x() + region.width();
    let region_bottom = region.y() + region.height();
    let surface_right = surface.x() + surface.width();
    let surface_bottom = surface.y() + surface.height();
    match corner {
        0 => region.x() == surface.x() && region.y() == surface.y(),
        1 => region_right == surface_right && region.y() == surface.y(),
        2 => region_right == surface_right && region_bottom == surface_bottom,
        3 => region.x() == surface.x() && region_bottom == surface_bottom,
        _ => false,
    }
}
