use super::*;
use worth_ui_host_contract::*;

#[test]
fn scroll_succession_preserves_reused_rows_but_not_new_receipts_at_identical_bounds() {
    for size in [64, 4096] {
        let frame = UiMountedFrameIdentity::mint_unbound().unwrap();
        let issuer = UiMountedNodeReceiptIssuer::mint_for(frame).unwrap();
        let binding = UiSurfaceBindingGeneration::mint_unbound().unwrap();
        let surface = UiSemanticSurfaceIdentity::mint_unbound().unwrap();
        let mut source = UiPresentedHitIndex::default();
        let mut instances = Vec::new();
        for rank in 0..size {
            let instance = UiMountedInstanceIdentity::mint_unbound().unwrap();
            source.replace_base(
                instance,
                Some(super::super::tests::row(
                    issuer,
                    binding,
                    surface,
                    instance,
                    rank,
                    [0.0, 100.0, 10.0, 10.0],
                )),
            );
            instances.push(instance);
        }
        let instance = instances[0];
        let mut accepted = source.clone();
        accepted.apply_scroll_translations(
            binding,
            &[(
                instance,
                crate::mounting::presentation::UiScrollPoseShift::between(
                    crate::runtime::scroll::UiScrollOffset::origin(),
                    crate::runtime::scroll::UiScrollOffset::new(
                        0,
                        60 * worth_ui_host_contract::UI_HOST_SURFACE_POSITION_SUBPIXELS_PER_UNIT,
                    )
                    .unwrap(),
                ),
            )],
        );
        let mut reused = source.clone();
        let work = reused.inherit_accepted_scroll(&accepted, binding);
        assert_eq!(work.scroll_rows_displaced(), 1);
        assert!(work.succession_cursor_steps() < 128);
        assert_eq!(
            reused
                .for_instance(binding, instance)
                .0
                .unwrap()
                .bounds()
                .platform_box()
                .y(),
            40.0
        );
        assert_eq!(
            source
                .for_instance(binding, instance)
                .0
                .unwrap()
                .bounds()
                .platform_box()
                .y(),
            100.0
        );
        assert_eq!(
            reused
                .inherit_accepted_scroll(&accepted, binding)
                .scroll_rows_displaced(),
            0
        );

        // An accepted return to origin has the same source rectangle but fresh
        // provenance. The predecessor's -60 displacement must not be inherited.
        let successor =
            UiMountedNodeReceiptIssuer::mint_for(UiMountedFrameIdentity::mint_unbound().unwrap())
                .unwrap();
        let mut relaid = source.clone();
        relaid.replace_base(
            instance,
            Some(super::super::tests::row(
                successor,
                binding,
                surface,
                instance,
                0,
                [0.0, 100.0, 10.0, 10.0],
            )),
        );
        assert_eq!(
            relaid
                .inherit_accepted_scroll(&accepted, binding)
                .scroll_rows_displaced(),
            0
        );
        assert_eq!(
            relaid
                .for_instance(binding, instance)
                .0
                .unwrap()
                .bounds()
                .platform_box()
                .y(),
            100.0
        );
    }
}

#[test]
fn scroll_poses_too_small_to_move_a_row_still_accumulate() {
    let frame = UiMountedFrameIdentity::mint_unbound().unwrap();
    let issuer = UiMountedNodeReceiptIssuer::mint_for(frame).unwrap();
    let binding = UiSurfaceBindingGeneration::mint_unbound().unwrap();
    let surface = UiSemanticSurfaceIdentity::mint_unbound().unwrap();
    let instance = UiMountedInstanceIdentity::mint_unbound().unwrap();
    let pose = |subpixels| crate::runtime::scroll::UiScrollOffset::new(0, subpixels).unwrap();
    let mut stepped = UiPresentedHitIndex::default();
    // Far from the origin one subpixel is below what a row's box resolves.
    stepped.replace_base(
        instance,
        Some(super::super::tests::row(
            issuer,
            binding,
            surface,
            instance,
            0,
            [0.0, 100_000.0, 10.0, 10.0],
        )),
    );
    let mut settled = stepped.clone();
    for step in 0..8 {
        stepped.apply_scroll_translations(
            binding,
            &[(
                instance,
                crate::mounting::presentation::UiScrollPoseShift::between(
                    pose(step),
                    pose(step + 1),
                ),
            )],
        );
    }
    settled.apply_scroll_translations(
        binding,
        &[(
            instance,
            crate::mounting::presentation::UiScrollPoseShift::between(pose(0), pose(8)),
        )],
    );
    let y = |index: &UiPresentedHitIndex| {
        index
            .for_instance(binding, instance)
            .0
            .unwrap()
            .bounds()
            .platform_box()
            .y()
    };
    assert!(y(&settled) < 100_000.0);
    assert_eq!(y(&stepped), y(&settled));
    assert_eq!(
        stepped.committed_scroll_translation(binding, instance).0,
        settled.committed_scroll_translation(binding, instance).0,
    );
}
