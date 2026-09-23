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
        accepted.apply_scroll_translations(binding, &[(instance, [0.0, -60.0])]);
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
                .y(),
            40.0
        );
        assert_eq!(
            source
                .for_instance(binding, instance)
                .0
                .unwrap()
                .bounds()
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
                .y(),
            100.0
        );
    }
}
