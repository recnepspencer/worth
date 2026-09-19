use super::*;
use crate::mounting::projection::appearance::UiMountedAppearanceGeometryScope;

#[test]
fn foreground_only_lowering_composes_divergent_exact_text_motion() {
    let (mut input, ids) = node_input([12, 34, 56, 255], 7, false);
    let first = input.nodes[0].text_foregrounds[0].command;
    let second =
        worth_ui_host_contract::UiMountedPaintCommandIdentity::semantic_text_from_correspondence(
            ids.instance,
            u16::MAX,
            None,
        );
    let mut second_foreground = input.nodes[0].text_foregrounds[0].clone();
    second_foreground.command = second;
    second_foreground.span =
        worth_ui_host_contract::UiMountedTextPaintSpanIdentity::from_runtime_mounting([9; 32]);
    let node = &mut input.nodes[0];
    node.surface_paint = None;
    node.surface_paint_order = None;
    node.outline = None;
    node.text_foregrounds =
        vec![node.text_foregrounds[0].clone(), second_foreground].into_boxed_slice();

    let motion =
        crate::mounting::presentation::UiAcceptedAppearanceMotion::from_text_commands_for_test(&[
            (first, Some(32_768)),
            (second, None),
        ]);
    let scope = UiMountedAppearanceGeometryScope::with_motion(&[], None, motion);

    input
        .compose_accepted_motion(&scope)
        .expect("exact text samples do not require an aggregate node sample");
    assert_eq!(input.nodes[0].motion_opacity, None);
    assert_eq!(
        input.nodes[0].text_foregrounds[0].motion_opacity,
        Some(32_768)
    );
    assert_eq!(input.nodes[0].text_foregrounds[1].motion_opacity, None);

    let mut sidecar = UiMountedAppearanceSidecar::default();
    let work = sidecar
        .mount(input)
        .expect("foreground-only exact motion should lower");
    let opacities = work
        .successor()
        .mechanics()
        .iter()
        .map(|mechanic| match mechanic {
            UiMountedAppearanceMechanic::TextForeground(text) => text.opacity().units(),
            _ => panic!("foreground-only input emitted another mechanic family"),
        })
        .collect::<Vec<_>>();
    assert_eq!(opacities, [20_000, 40_000]);
}
