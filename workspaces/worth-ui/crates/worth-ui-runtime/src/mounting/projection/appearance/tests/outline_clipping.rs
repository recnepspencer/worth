use super::*;
use crate::mounting::{UiMountedAppearanceClip, UiMountedAppearanceClipDenial};

#[test]
fn ancestor_clip_restricts_outline_coverage_without_reducing_visual_damage() {
    // This focused lowering fixture supplies an admitted ancestor rectangle;
    // it does not claim to derive Mosaic or Portal membership.
    let (input, _) = node_input([12, 34, 56, 255], 7, true);
    let mut sidecar = UiMountedAppearanceSidecar::default();
    let work = sidecar.mount(input).unwrap();
    let outline = work
        .successor()
        .mechanics()
        .iter()
        .find_map(|mechanic| match mechanic {
            UiMountedAppearanceMechanic::Outline(outline) => Some(outline),
            _ => None,
        })
        .unwrap();
    let clip = outline.clip();
    assert_eq!(clip, UiAppearanceClip::new(0, 0, 90, 70).unwrap());
    // Allocation10,20..110,100; width4+offset2. Left stroke remains visible,
    // while the right and bottom strokes lie beyond this ancestor.
    let covered = |x, y| {
        x >= clip.x()
            && y >= clip.y()
            && i64::from(x) < i64::from(clip.x()) + i64::from(clip.width())
            && i64::from(y) < i64::from(clip.y()) + i64::from(clip.height())
    };
    assert!(covered(6, 50));
    assert!(!covered(114, 50));
    assert!(!covered(50, 104));
    assert_eq!(work.damage().len(), 1);
    let damage = &work.damage()[0];
    assert_eq!(
        (damage.x(), damage.y(), damage.width(), damage.height()),
        (3, 13, 114, 94)
    );
    assert!(!outline.participates_in_hit_testing());
}

#[test]
fn unresolved_ancestor_geometry_does_not_replace_retained_appearance() {
    let (initial, ids) = node_input([12, 34, 56, 255], 7, true);
    let mut sidecar = UiMountedAppearanceSidecar::default();
    sidecar.mount(initial).unwrap();
    let predecessor = sidecar.current().unwrap().frame().clone();
    for denial in [
        UiMountedAppearanceClipDenial::ExecutedPlanUnavailable,
        UiMountedAppearanceClipDenial::MosaicBindingUnavailable(
            crate::graph::UiGraphNodeIdentity::new(11),
        ),
        UiMountedAppearanceClipDenial::PortalBindingUnavailable(
            crate::graph::UiGraphNodeIdentity::new(11),
        ),
        UiMountedAppearanceClipDenial::ScrollBindingUnavailable(
            crate::graph::UiGraphNodeIdentity::new(11),
        ),
    ] {
        let (mut successor, _) = node_input_for([255, 0, 0, 255], 8, true, Some(&ids));
        successor.nodes[0].clip = UiMountedAppearanceClip::Unresolved(denial);
        assert_eq!(
            sidecar.mount(successor),
            Err(crate::mounting::UiMountedAppearanceLoweringDenial::AncestorClip(denial))
        );
        assert_eq!(sidecar.current().unwrap().frame(), &predecessor);
    }
}
