pub(super) fn transform_presented_box(
    bounds: worth_ui_host_contract::UiMountedCanonicalBox,
    source: crate::runtime::motion::UiMotionSemanticGeometry,
    sampled: crate::mounting::presentation::motion_sampling::UiPresentationSampledGeometry,
) -> worth_ui_host_contract::UiMountedCanonicalBox {
    transform_presented_components(bounds, source, sampled.components())
}

pub(super) fn transform_presented_components(
    bounds: worth_ui_host_contract::UiMountedCanonicalBox,
    source: crate::runtime::motion::UiMotionSemanticGeometry,
    sampled_components: [f32; 4],
) -> worth_ui_host_contract::UiMountedCanonicalBox {
    let source_components = source.components();
    assert_eq!(
        source.coordinate_space(),
        bounds.coordinate_space(),
        "Portal motion and presented hit geometry share a coordinate space"
    );
    let scale_x = sampled_components[2] / source_components[2];
    let scale_y = sampled_components[3] / source_components[3];
    worth_ui_host_contract::UiMountedCanonicalBox::canonicalize(
        worth_ui_host_contract::UiMountedCanonicalBoxInput {
            x: sampled_components[0] + (bounds.x() - source_components[0]) * scale_x,
            y: sampled_components[1] + (bounds.y() - source_components[1]) * scale_y,
            width: bounds.width() * scale_x,
            height: bounds.height() * scale_y,
            coordinate_space: bounds.coordinate_space(),
        },
    )
    .expect("validated Portal sample transform keeps hit geometry canonical")
}
