use worth_ui::facade::app::{
    UiMountedCanonicalBox, UiMountedCanonicalBoxInput, UiMountedCoordinateSpace,
    UiMountedLayoutRevision, UiMountedOccurrenceGeometry, UiMountedSurfaceGeometryBatch,
    WorthUiActiveApplicationSession,
};
use worth_ui_runtime::facade::mounted::UiSemanticSurfaceIdentity;
use worth_ui_test_support::{
    WorthUiActiveSessionCertificationExt, WorthUiMountedIdentityCertificationExt,
};

pub(super) fn complete(
    session: &mut WorthUiActiveApplicationSession,
    surface: UiSemanticSurfaceIdentity,
) {
    let identity = session.inspect_mounted_identity();
    let graph = session.graph();
    let mounted = |name: &str| {
        let matches = identity
            .mounted_instances()
            .iter()
            .filter(|instance| {
                graph
                    .lookup()
                    .graph_node(instance.graph_node_identity())
                    .is_some_and(|node| {
                        node.value().declaration_identity().authored_semantic_name()
                            == format!("component:{name}")
                    })
            })
            .collect::<Vec<_>>();
        assert_eq!(
            matches.len(),
            1,
            "each authored component mounts exactly once"
        );
        matches[0].identity()
    };
    let owner = mounted(super::PAINT_AND_HIT);
    // The measured viewport is 160x96. Component contracts specify these two
    // insets; the Portal child carries its rectangle relative to the owner.
    let occurrences = [
        UiMountedOccurrenceGeometry::surface(
            mounted(super::PAINT_ONLY),
            rectangle(
                [0.0, 0.0, 160.0, 96.0],
                UiMountedCoordinateSpace::HostSurface,
            ),
        ),
        UiMountedOccurrenceGeometry::surface(
            owner,
            rectangle(
                [16.0, 12.0, 128.0, 72.0],
                UiMountedCoordinateSpace::HostSurface,
            ),
        ),
        UiMountedOccurrenceGeometry::parent_relative(
            mounted(super::HIT_ONLY),
            owner,
            rectangle(
                [8.0, 8.0, 144.0, 80.0],
                UiMountedCoordinateSpace::GraphNodeLocal,
            ),
        ),
        UiMountedOccurrenceGeometry::surface(
            mounted(super::NEITHER),
            rectangle(
                [144.0, 80.0, 8.0, 8.0],
                UiMountedCoordinateSpace::HostSurface,
            ),
        ),
    ];
    let regions = session.declared_region_layout_inputs(surface);
    assert_eq!(regions.len(), 1);
    assert_eq!(regions[0].owner(), owner);
    assert_eq!(regions[0].region_kind(), super::SCROLL_REGION);
    // Half the owner's height, so the content that owner carries does not fit
    // the region showing it and the reader has thirty-six points of block
    // travel. A region given a box the size of its owner is a region with
    // nothing to scroll, and the reveal this world proves needs a predecessor
    // offset a scroll could reach.
    let region = regions[0].geometry(rectangle(
        [0.0, 0.0, 128.0, 36.0],
        UiMountedCoordinateSpace::GraphNodeLocal,
    ));
    let mut layout = session.begin_mounted_layout();
    let basis = layout.basis(surface).unwrap();
    layout
        .complete_surface_geometry(
            UiMountedSurfaceGeometryBatch::new(
                basis,
                UiMountedLayoutRevision::new(1).unwrap(),
                rectangle(
                    [0.0, 0.0, 160.0, 96.0],
                    UiMountedCoordinateSpace::HostSurface,
                ),
                occurrences,
            )
            .with_regions([region]),
        )
        .expect("the Selection Portal completes exact component and region occurrences");
}

fn rectangle(
    [x, y, width, height]: [f32; 4],
    coordinate_space: UiMountedCoordinateSpace,
) -> UiMountedCanonicalBox {
    UiMountedCanonicalBox::canonicalize(UiMountedCanonicalBoxInput {
        x,
        y,
        width,
        height,
        coordinate_space,
    })
    .unwrap()
}
