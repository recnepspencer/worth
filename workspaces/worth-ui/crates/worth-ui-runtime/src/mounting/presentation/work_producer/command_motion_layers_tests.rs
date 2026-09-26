use super::*;
use worth_ui_host_contract::{
    UiMountedCanonicalBoxInput, UiMountedCoordinateSpace, UiMountedInstanceIdentity,
};

pub(in super::super) fn boxed([x, y, width, height]: [f32; 4]) -> UiMountedCanonicalBox {
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
    UiMountedPresentationTransform::from_runtime_sampling(boxed(source), boxed(sampled)).unwrap()
}

pub(in super::super) fn command() -> UiMountedPaintCommandIdentity {
    UiMountedPaintCommandIdentity::appearance_surface(
        UiMountedInstanceIdentity::mint_unbound().unwrap(),
    )
}

pub(in super::super) const RESTING: UiMountedAppearanceOpacity =
    UiMountedAppearanceOpacity::from_units(40_000);

/// A Scroll lifting content 6 points, clipped to its viewport.
pub(in super::super) fn scroll() -> UiCommandMotionLayer {
    UiCommandMotionLayer::scrolled(
        transform([0.0, 0.0, 1.0, 1.0], [0.0, -6.0, 1.0, 1.0]),
        boxed([100.0, 100.0, 200.0, 100.0]),
        u16::MAX,
    )
}

/// A Portal entrance halving its box about its origin, moved down 20, at
/// half its opacity.
pub(in super::super) fn portal() -> UiCommandMotionLayer {
    UiCommandMotionLayer::moved(
        Some(transform(
            [100.0, 100.0, 200.0, 100.0],
            [100.0, 120.0, 100.0, 50.0],
        )),
        32_768,
    )
}

pub(in super::super) fn layers(
    sampled: &[(UiCommandMotionLayerKind, UiCommandMotionLayer)],
) -> UiCommandMotionLayers {
    let mut layers = UiCommandMotionLayers::default();
    for (kind, layer) in sampled {
        layers.sample(*kind, *layer).unwrap();
    }
    layers
}

#[test]
fn one_layer_shows_exactly_what_that_layer_samples() {
    let command = command();
    let moved = transform([10.0, 10.0, 40.0, 20.0], [12.0, 9.0, 40.0, 20.0]);
    assert_eq!(
        layers(&[(
            UiCommandMotionLayerKind::Own,
            UiCommandMotionLayer::moved(Some(moved), 1_111)
        )])
        .change(command, RESTING),
        Ok(UiMountedPresentationSampleChange::from_runtime_sampling(
            command,
            Some(moved),
            compose_opacity(RESTING, 1_111),
        ))
    );
    let scrolled = scroll();
    assert_eq!(
        layers(&[(UiCommandMotionLayerKind::Scroll, scrolled)]).change(command, RESTING),
        Ok(
            UiMountedPresentationSampleChange::from_runtime_scroll_sampling(
                command,
                scrolled.transform.unwrap(),
                compose_opacity(RESTING, u16::MAX),
                scrolled.clip.unwrap(),
            )
            .unwrap()
        )
    );
}

#[test]
fn a_portal_moves_scrolled_content_with_its_clip_and_fades_it() {
    let command = command();
    let change = layers(&[
        (UiCommandMotionLayerKind::Portal, portal()),
        (UiCommandMotionLayerKind::Scroll, scroll()),
    ])
    .change(command, RESTING)
    .unwrap();
    let shown = change.transform().unwrap();
    assert_eq!(shown.source(), boxed([0.0, 0.0, 1.0, 1.0]));
    assert_eq!(shown.sampled(), boxed([50.0, 67.0, 0.5, 0.5]));
    // The viewport clips the scrolled content where the Portal shows it.
    assert_eq!(change.clip(), Some(boxed([100.0, 120.0, 100.0, 50.0])));
    assert_eq!(change.opacity(), compose_opacity(RESTING, 32_768));
}

#[test]
fn a_portal_layer_alone_shows_what_it_shows_among_the_layers() {
    let command = command();
    for units in [0, 1, 32_768, u16::MAX] {
        let portal = UiCommandMotionLayer::moved(
            Some(transform([0.0, 0.0, 2.0, 2.0], [50.0, 60.0, 1.0, 1.0])),
            units,
        );
        let held = layers(&[(UiCommandMotionLayerKind::Portal, portal)]);
        let alone = held.portal_only().unwrap();
        assert_eq!(alone.layers(), held);
        assert_eq!(
            alone.change(command, RESTING),
            held.change(command, RESTING).unwrap()
        );
    }
}

#[test]
fn only_scroll_clips() {
    for kind in [
        UiCommandMotionLayerKind::Own,
        UiCommandMotionLayerKind::Portal,
    ] {
        assert_eq!(
            UiCommandMotionLayers::default().sample(kind, scroll()),
            Err(Denial::InvalidGeometry)
        );
    }
}

#[test]
fn own_motion_inside_a_portal_multiplies_both_fades() {
    let command = command();
    let change = layers(&[
        (
            UiCommandMotionLayerKind::Own,
            UiCommandMotionLayer::moved(None, 32_768),
        ),
        (UiCommandMotionLayerKind::Portal, portal()),
    ])
    .change(command, UiMountedAppearanceOpacity::ONE)
    .unwrap();
    assert_eq!(change.clip(), None);
    assert_eq!(change.opacity().motion_units(), 16_384);
}

#[test]
fn a_tick_holds_every_layer_it_does_not_sample() {
    let held = layers(&[
        (UiCommandMotionLayerKind::Scroll, scroll()),
        (UiCommandMotionLayerKind::Portal, portal()),
    ]);
    let settled = UiCommandMotionLayer::moved(
        Some(transform(
            [100.0, 100.0, 200.0, 100.0],
            [100.0, 100.0, 200.0, 100.0],
        )),
        u16::MAX,
    );
    let shown = layers(&[(UiCommandMotionLayerKind::Portal, settled)]).over(held);
    assert_eq!(
        shown,
        layers(&[
            (UiCommandMotionLayerKind::Scroll, scroll()),
            (UiCommandMotionLayerKind::Portal, settled),
        ])
    );
    // A Portal at rest leaves the Scroll's change exactly as it was.
    let command = command();
    assert_eq!(
        shown.change(command, RESTING),
        layers(&[(UiCommandMotionLayerKind::Scroll, scroll())]).change(command, RESTING)
    );
    assert_eq!(shown.scroll_transform(), scroll().transform);
}

#[test]
fn two_motions_of_one_kind_cannot_move_one_command_in_one_tick() {
    let mut tick = layers(&[(UiCommandMotionLayerKind::Portal, portal())]);
    assert_eq!(
        tick.sample(UiCommandMotionLayerKind::Portal, portal()),
        Err(Denial::AmbiguousTargetCommands)
    );
    assert_eq!(
        tick.sample(UiCommandMotionLayerKind::Scroll, scroll()),
        Ok(())
    );
}

#[test]
fn what_one_layer_moves_the_layers_outside_it_carry() {
    let shown = layers(&[
        (UiCommandMotionLayerKind::Scroll, scroll()),
        (UiCommandMotionLayerKind::Portal, portal()),
    ]);
    let damage = boxed([140.0, 152.0, 10.0, 10.0]);
    assert_eq!(
        shown.carried_outside(UiCommandMotionLayerKind::Own, damage),
        Ok(boxed([120.0, 143.0, 5.0, 5.0]))
    );
    assert_eq!(
        shown.carried_outside(UiCommandMotionLayerKind::Portal, damage),
        Ok(damage)
    );
}
