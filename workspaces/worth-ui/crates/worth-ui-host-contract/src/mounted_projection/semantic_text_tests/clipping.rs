use super::*;

#[test]
fn rows_scrolled_past_the_surface_origin_retain_intrinsic_clip_for_later_reveal() {
    // The live geometry of a scroll row carried left of the viewport origin: it
    // keeps positive, finite extent while visible coverage becomes empty.
    let mut input = fixture();
    input.bounds = canonical_box(-462.0, 418.0, 420.0, 23.0);
    input.clip_bounds = input.bounds;
    input.origin_x = -462.0;
    input.origin_y = 426.0;
    assert_eq!(
        input.bounds.posture(),
        crate::UiMountedGeometryPosture::Offscreen
    );

    let row = complete(input.clone());
    assert_eq!(row.bounds(), input.bounds);

    let scroll_region = crate::UiAppearanceClip::new(290_000, 693_000, 768_000, 269_000).unwrap();
    let clipped = row
        .clipped_to_appearance_ancestor(scroll_region)
        .unwrap()
        .unwrap();
    assert_eq!(clipped.bounds(), input.bounds);
    assert_eq!(clipped.intrinsic_clip_bounds(), input.clip_bounds);
    assert_eq!(clipped.clip_bounds().width(), 0.0);
    assert_eq!(clipped.clip_bounds().height(), 0.0);
}

#[test]
fn identical_visible_clips_do_not_hide_changed_intrinsic_text_clipping() {
    let mut input = fixture();
    let first = complete(input.clone());
    input.clip_bounds = canonical_box(32.0, 32.0, 80.0, 96.0);
    let second = complete(input);
    let ancestor = crate::UiAppearanceClip::new(32_000, 32_000, 40_000, 96_000).unwrap();
    let first = first
        .clipped_to_appearance_ancestor(ancestor)
        .unwrap()
        .unwrap();
    let second = second
        .clipped_to_appearance_ancestor(ancestor)
        .unwrap()
        .unwrap();
    assert_eq!(first.clip_bounds(), second.clip_bounds());
    assert!(!first.same_retained_paint_meaning(&second));
}
