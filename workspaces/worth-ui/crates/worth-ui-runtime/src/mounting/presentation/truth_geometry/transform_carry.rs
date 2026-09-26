//! A presentation transform carried by another.
//!
//! The host shows a command through one transform per command. When two
//! Motions move the same command, the outer one moves whatever the inner one
//! shows: the transforms compose, and a clip the inner one applies after
//! moving is carried along with what it clips.

use super::logical_rect::UiLogicalRect;
use worth_ui_host_contract::{UiMountedCanonicalBox, UiMountedPresentationTransform};

/// Where `outer` shows `bounds`, drawn in the space `outer` maps. `None`
/// when `bounds` is drawn in another space, or `outer` carries it beyond
/// finite geometry.
pub(in crate::mounting::presentation) fn carried_box(
    bounds: UiMountedCanonicalBox,
    outer: UiMountedPresentationTransform,
) -> Option<UiMountedCanonicalBox> {
    let (from, to) = (outer.source(), outer.sampled());
    if bounds.coordinate_space() != from.coordinate_space() {
        return None;
    }
    if from == to {
        return Some(bounds);
    }
    // A transform's source has area, so the map it makes is defined.
    UiLogicalRect::from_box(bounds)
        .mapped(UiLogicalRect::from_box(from), UiLogicalRect::from_box(to))
        .map(UiLogicalRect::canonical_box)
}

/// `inner` and then `outer`: `outer` moves where `inner` puts its source.
/// `None` when the two are drawn in different spaces.
pub(in crate::mounting::presentation) fn carried_transform(
    inner: UiMountedPresentationTransform,
    outer: UiMountedPresentationTransform,
) -> Option<UiMountedPresentationTransform> {
    let sampled = carried_box(inner.sampled(), outer)?;
    UiMountedPresentationTransform::from_runtime_sampling(inner.source(), sampled).ok()
}

#[cfg(test)]
mod tests {
    use super::*;
    use worth_ui_host_contract::{UiMountedCanonicalBoxInput, UiMountedCoordinateSpace};

    fn boxed([x, y, width, height]: [f32; 4]) -> UiMountedCanonicalBox {
        UiMountedCanonicalBox::canonicalize(UiMountedCanonicalBoxInput {
            x,
            y,
            width,
            height,
            coordinate_space: UiMountedCoordinateSpace::Viewport,
        })
        .unwrap()
    }

    fn transform(source: [f32; 4], sampled: [f32; 4]) -> UiMountedPresentationTransform {
        UiMountedPresentationTransform::from_runtime_sampling(boxed(source), boxed(sampled))
            .unwrap()
    }

    #[test]
    fn an_outer_transform_moves_what_the_inner_one_shows() {
        // Scroll lifts content 6 points; a Portal entrance halves its box
        // about the Portal's origin and moves it down 20.
        let scroll = transform([0.0, 0.0, 1.0, 1.0], [0.0, -6.0, 1.0, 1.0]);
        let portal = transform([100.0, 100.0, 200.0, 100.0], [100.0, 120.0, 100.0, 50.0]);
        let carried = carried_transform(scroll, portal).unwrap();
        assert_eq!(carried.source(), boxed([0.0, 0.0, 1.0, 1.0]));
        assert_eq!(carried.sampled(), boxed([50.0, 67.0, 0.5, 0.5]));
        // A point the scroll puts at (140, 146) the Portal shows at (120, 143).
        assert_eq!(
            carried_box(boxed([140.0, 146.0, 10.0, 10.0]), portal),
            Some(boxed([120.0, 143.0, 5.0, 5.0]))
        );
    }

    #[test]
    fn a_resting_outer_transform_changes_nothing() {
        let scroll = transform([0.0, 0.0, 1.0, 1.0], [0.0, -6.1, 1.0, 1.0]);
        let resting = transform([100.3, 100.7, 200.0, 100.0], [100.3, 100.7, 200.0, 100.0]);
        assert_eq!(carried_transform(scroll, resting), Some(scroll));
    }

    #[test]
    fn transforms_in_different_spaces_do_not_compose() {
        let viewport = transform([0.0, 0.0, 1.0, 1.0], [0.0, 1.0, 1.0, 1.0]);
        let window = UiMountedCanonicalBox::canonicalize(UiMountedCanonicalBoxInput {
            x: 0.0,
            y: 0.0,
            width: 1.0,
            height: 1.0,
            coordinate_space: UiMountedCoordinateSpace::Window,
        })
        .unwrap();
        let other = UiMountedPresentationTransform::from_runtime_sampling(window, window).unwrap();
        assert_eq!(carried_transform(viewport, other), None);
    }
}
