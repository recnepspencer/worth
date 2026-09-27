//! A hit mechanic standing for one Motion target, for tests that sample it.

pub(crate) fn motion_sampling_hit_test_mechanic_for_test(
    presentation: worth_ui_host_contract::UiHostObservationPresentationBasis,
    target: crate::runtime::motion::UiMotionTargetIdentity,
    components: [f32; 4],
) -> worth_ui_host_contract::UiMountedHitTestMechanic {
    let bounds = worth_ui_host_contract::UiMountedCanonicalBox::canonicalize(
        worth_ui_host_contract::UiMountedCanonicalBoxInput {
            x: components[0],
            y: components[1],
            width: components[2],
            height: components[3],
            coordinate_space: worth_ui_host_contract::UiMountedCoordinateSpace::Viewport,
        },
    )
    .expect("test hit-test geometry is canonical");
    let receipt =
        worth_ui_host_contract::UiMountedNodeReceiptIssuer::mint_for(presentation.frame())
            .expect("test frame identity is non-zero")
            .receipt_for(target.mounted_instance());
    worth_ui_host_contract::UiMountedHitTestMechanic::complete_from_runtime_mounting(
        worth_ui_host_contract::UiMountedHitTestCompletionInput {
            frame: presentation.frame(),
            surface: target.semantic_surface(),
            binding: presentation.binding(),
            mounted_instance: target.mounted_instance(),
            node_receipt: receipt,
            bounds,
            clip_bounds: bounds,
            order: worth_ui_host_contract::UiMountedHitTestOrder::from_runtime_plan(1),
        },
    )
    .expect("test hit-test mechanic is coherent")
}
