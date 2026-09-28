//! A group finds each member it shows elsewhere, whatever order layout
//! visits its members in.
use super::UiShownScrollGroup;
use crate::mounting::UiMountedScrollRegionBoxes;
use worth_ui_host_contract::{
    UiMountedCanonicalBox, UiMountedCanonicalBoxInput, UiMountedCoordinateSpace,
    UiMountedInstanceIdentity,
};

fn rect(x: f32, y: f32) -> UiMountedCanonicalBox {
    UiMountedCanonicalBox::canonicalize(UiMountedCanonicalBoxInput {
        x,
        y,
        width: 100.0,
        height: 50.0,
        coordinate_space: UiMountedCoordinateSpace::Viewport,
    })
    .unwrap()
}

#[test]
fn members_shown_elsewhere_are_found_in_any_layout_order() {
    let first = UiMountedInstanceIdentity::mint_unbound().unwrap();
    let second = UiMountedInstanceIdentity::mint_unbound().unwrap();
    let third = UiMountedInstanceIdentity::mint_unbound().unwrap();
    let region = rect(0.0, 0.0);
    // Layout visits the later-mounted members first.
    let shown = UiShownScrollGroup::new(
        UiMountedScrollRegionBoxes::new(region, region),
        vec![(third, rect(300.0, 0.0)), (second, rect(200.0, 0.0))],
    );
    assert_eq!(shown.viewport_showing(second), rect(200.0, 0.0));
    assert_eq!(shown.viewport_showing(third), rect(300.0, 0.0));
    assert_eq!(shown.viewport_showing(first), region);
}
