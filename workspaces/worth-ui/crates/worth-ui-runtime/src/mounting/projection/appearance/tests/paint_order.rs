use super::*;

#[test]
fn order_only_changes_update_surface_and_outline_with_visual_damage() {
    let (mut initial, ids) = node_input([12, 34, 56, 255], 7, true);
    initial.nodes[0].text_foregrounds = Box::new([]);
    initial.nodes[0].surface_paint_order = Some(65_536);
    let portal_group = UiMountedInstanceIdentity::mint_unbound().unwrap();
    initial.nodes[0].portal_group = Some(portal_group);
    let mut sidecar = UiMountedAppearanceSidecar::default();
    let before = sidecar.mount(initial).unwrap();
    let (mut next, _) = node_input_for([12, 34, 56, 255], 7, true, Some(&ids));
    next.nodes[0].text_foregrounds = Box::new([]);
    next.nodes[0].surface_paint_order = Some(u32::MAX);
    next.nodes[0].portal_group = Some(portal_group);
    let after = sidecar.mount(next).unwrap();

    for (work, expected) in [(&before, 65_536), (&after, u32::MAX)] {
        let [UiMountedAppearanceMechanic::Surface(surface), UiMountedAppearanceMechanic::Outline(outline)] =
            work.successor().mechanics()
        else {
            panic!("surface and outline remain separate ordered mechanics");
        };
        assert_eq!(surface.surface_paint_order(), expected);
        assert_eq!(outline.surface_paint_order(), expected);
        assert_eq!(surface.portal_group(), Some(portal_group));
        assert_eq!(outline.portal_group(), Some(portal_group));
        assert!(!outline.participates_in_hit_testing());
    }
    assert_eq!(after.changes().len(), 2);
    assert!(!after.damage().is_empty());
    assert_eq!(after.damage(), before.damage());
    assert!(!sidecar.last_delta().unwrap().output_suppressed());
}

#[test]
fn missing_order_denies_surface_and_outline_only_without_replacing_predecessor() {
    let (initial, ids) = node_input([12, 34, 56, 255], 7, true);
    let mut sidecar = UiMountedAppearanceSidecar::default();
    sidecar.mount(initial).unwrap();
    let predecessor = sidecar.current().unwrap().frame().clone();
    for outline_only in [false, true] {
        let (mut next, _) = node_input_for([255, 0, 0, 255], 8, outline_only, Some(&ids));
        next.nodes[0].surface_paint_order = None;
        next.nodes[0].text_foregrounds = Box::new([]);
        if outline_only {
            next.nodes[0].surface_paint = None;
        }
        assert_eq!(
            sidecar.mount(next),
            Err(crate::mounting::UiMountedAppearanceLoweringDenial::SurfacePaintOrderUnavailable),
        );
        assert_eq!(sidecar.current().unwrap().frame(), &predecessor);
    }
}
