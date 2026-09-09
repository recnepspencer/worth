use super::*;

#[test]
fn mounting_emits_the_single_composed_factor_for_each_own_node_mechanic() {
    // These are mounted-mechanic fixtures. Real Motion target binding and
    // original UTF-8 text production are separate integration obligations.
    for (appearance, motion, expected) in [
        (40_000, 32_768, 20_000),
        (40_000, 8_192, 5_000),
        (20_000, 8_192, 2_500),
    ] {
        let (mut input, _) = node_input([12, 34, 56, 255], 7, true);
        input.nodes[0].appearance_opacity = UiMountedAppearanceOpacity::from_units(appearance);
        input.nodes[0].motion_opacity = Some(motion);
        for foreground in &mut input.nodes[0].text_foregrounds {
            foreground.motion_opacity = Some(motion);
        }
        let work = UiMountedAppearanceSidecar::default().mount(input).unwrap();
        let mut actual = Vec::new();
        for mechanic in work.successor().mechanics() {
            use worth_ui_host_contract::UiMountedAppearanceMechanic as Mechanic;
            let (family, opacity) = match mechanic {
                Mechanic::Surface(mechanic) => ("surface", mechanic.opacity()),
                Mechanic::Outline(mechanic) => ("outline", mechanic.opacity()),
                Mechanic::TextForeground(mechanic) => ("text", mechanic.opacity()),
                _ => panic!("fixture must emit only its own node mechanics"),
            };
            let final_opacity: worth_ui_host_contract::UiMountedPresentationOpacity = opacity;
            actual.push((family, final_opacity.units()));
        }
        actual.sort_unstable();
        assert_eq!(
            actual,
            [
                ("outline", expected),
                ("surface", expected),
                ("text", expected)
            ]
        );
    }
}
