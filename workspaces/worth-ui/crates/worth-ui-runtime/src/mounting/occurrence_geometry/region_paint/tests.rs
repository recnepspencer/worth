use super::*;
use crate::capability::{
    MosaicExteriorCorner, MosaicExteriorCornerPosture as Corner, MosaicRegionKindId,
    MosaicSeamPaintContract, MosaicSeamPaintOwner, MosaicSharedEdge,
};
use worth_ui_host_contract::{UiMountedCanonicalBoxInput, UiMountedCoordinateSpace};

#[test]
fn exact_owner_paints_shared_edge_and_only_exterior_corners_keep_radius() {
    let first = MosaicRegionKindId::new("region.first").unwrap();
    let second = MosaicRegionKindId::new("region.second").unwrap();
    let edge = MosaicSharedEdge::new(first.clone(), second.clone()).unwrap();
    let contract = MosaicSeamPaintContract::admit(
        [first.clone(), second.clone()],
        [edge.clone()],
        [MosaicSeamPaintOwner::new(edge, first.clone()).unwrap()],
        [
            MosaicExteriorCorner::new(first.clone(), Corner::TopLeft),
            MosaicExteriorCorner::new(first.clone(), Corner::BottomLeft),
            MosaicExteriorCorner::new(second.clone(), Corner::TopRight),
            MosaicExteriorCorner::new(second.clone(), Corner::BottomRight),
        ],
    )
    .unwrap();
    let first_owner = UiMountedInstanceIdentity::mint_unbound().unwrap();
    let second_owner = UiMountedInstanceIdentity::mint_unbound().unwrap();
    let postures = derive_contract(
        &contract,
        &[
            paint(first_owner, 1, first, bounds(0.0), bounds(0.0)),
            paint(second_owner, 2, second, bounds(10.0), bounds(10.0)),
        ],
    )
    .unwrap()
    .postures;

    let first = &postures[&first_owner];
    let second = &postures[&second_owner];
    assert_eq!(first.border_edges(), UiMountedSurfaceBorderEdges::ALL);
    assert_eq!(first.exterior_corners(), [true, false, false, true]);
    assert!(second.border_edges().top());
    assert!(second.border_edges().right());
    assert!(second.border_edges().bottom());
    assert!(!second.border_edges().left());
    assert_eq!(second.exterior_corners(), [false, true, true, false]);
}

#[test]
fn boundary_index_finds_each_neighbor_along_one_long_edge() {
    let primary = MosaicRegionKindId::new("region.primary").unwrap();
    let upper = MosaicRegionKindId::new("region.upper").unwrap();
    let lower = MosaicRegionKindId::new("region.lower").unwrap();
    let upper_edge = MosaicSharedEdge::new(primary.clone(), upper.clone()).unwrap();
    let lower_edge = MosaicSharedEdge::new(primary.clone(), lower.clone()).unwrap();
    let contract = MosaicSeamPaintContract::admit(
        [primary.clone(), upper.clone(), lower.clone()],
        [upper_edge.clone(), lower_edge.clone()],
        [
            MosaicSeamPaintOwner::new(upper_edge, primary.clone()).unwrap(),
            MosaicSeamPaintOwner::new(lower_edge, lower.clone()).unwrap(),
        ],
        [],
    )
    .unwrap();
    let primary_owner = UiMountedInstanceIdentity::mint_unbound().unwrap();
    let upper_owner = UiMountedInstanceIdentity::mint_unbound().unwrap();
    let lower_owner = UiMountedInstanceIdentity::mint_unbound().unwrap();
    let completion = derive_contract(
        &contract,
        &[
            paint(
                primary_owner,
                1,
                primary,
                rectangle(0.0, 0.0, 10.0, 20.0),
                rectangle(0.0, 0.0, 10.0, 20.0),
            ),
            paint(
                upper_owner,
                2,
                upper,
                rectangle(10.0, 0.0, 10.0, 8.0),
                rectangle(10.0, 0.0, 10.0, 8.0),
            ),
            paint(
                lower_owner,
                3,
                lower,
                rectangle(10.0, 12.0, 10.0, 8.0),
                rectangle(10.0, 12.0, 10.0, 8.0),
            ),
        ],
    )
    .unwrap();

    assert_eq!(completion.index_rows, 12);
    assert_eq!(completion.adjacencies_visited, 2);
    assert!(!completion.postures[&upper_owner].border_edges().left());
    assert!(completion.postures[&lower_owner].border_edges().left());
    assert!(completion.postures[&primary_owner].border_edges().right());
    let omission = completion.postures[&primary_owner].border_omissions()[0];
    assert_eq!(
        omission.side(),
        worth_ui_host_contract::UiMountedSurfaceBorderSide::Right
    );
    assert_eq!((omission.start(), omission.end()), (12.0, 20.0));
}

fn bounds(x: f32) -> UiMountedCanonicalBox {
    rectangle(x, 0.0, 10.0, 10.0)
}

fn paint(
    owner: UiMountedInstanceIdentity,
    declaration: u64,
    kind: MosaicRegionKindId,
    bounds: UiMountedCanonicalBox,
    surface_bounds: UiMountedCanonicalBox,
) -> UiMountedRegionPaintInput {
    UiMountedRegionPaintInput::new(
        owner,
        worth_ui_dsl::UiMosaicRegionDeclarationIdentity::new(declaration).unwrap(),
        kind,
        bounds,
        surface_bounds,
    )
}

fn rectangle(x: f32, y: f32, width: f32, height: f32) -> UiMountedCanonicalBox {
    UiMountedCanonicalBox::canonicalize(UiMountedCanonicalBoxInput {
        x,
        y,
        width,
        height,
        coordinate_space: UiMountedCoordinateSpace::HostSurface,
    })
    .unwrap()
}
