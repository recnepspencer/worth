use super::damage::UiNativeAppearanceDamageRect;
use crate::native::presentation::retained_raster::build_plan;
use crate::native::presentation::{
    appearance::UiNativeAppearanceReplayRegion,
    raster::UiNativeRasterBasis,
    retained_draw_list::{tests::DrawListWorld, UiNativeRetainedReplayPlan},
    UiNativeRetainedDrawList,
};
use worth_ui_host_contract::{
    UiHostSurfacePresentationDenial, UiMountedFrameIdentity, UiMountedRgba8,
};

#[test]
fn shared_replay_rejects_unconsumed_appearance_and_nontransparent_baseline() {
    let world = DrawListWorld::new();
    let frame = UiMountedFrameIdentity::mint_unbound().unwrap();
    let initial = world.initial(
        frame,
        [world.rect(frame, world.first, 10.0, UiMountedRgba8::new(1, 2, 3, 255))],
    );
    let mut retained = UiNativeRetainedDrawList::initial(&initial, &[]).unwrap();
    let basis = UiNativeRasterBasis::new([100, 100], 1.25);
    let atlas = crate::native::text_atlas::UiNativeTextAtlas::new();
    retained
        .initialize_physical_coverage(basis, &atlas)
        .unwrap();
    for (baseline, staged) in [([0; 4], false), ([0, 0, 0, 1], false), ([0; 4], true)] {
        let replay = UiNativeRetainedReplayPlan {
            physical_text_regions: Vec::new(),
            baseline_rgba8: baseline,
            regions: Box::new([]),
            staged_appearance_regions: if staged {
                Box::new([UiNativeAppearanceReplayRegion {
                    damage: UiNativeAppearanceDamageRect {
                        left: 1,
                        top: 1,
                        right: 2,
                        bottom: 2,
                    },
                    replay: Box::new([]),
                }])
            } else {
                Box::new([])
            },
            counters: Default::default(),
            identity_overlay_effect: false,
        };
        let result = build_plan(basis, &retained, replay, 0, &atlas);
        if baseline == [0; 4] && !staged {
            assert!(result.unwrap().operations.is_empty());
        } else {
            assert!(matches!(
                result,
                Err(UiHostSurfacePresentationDenial::MalformedProjection)
            ));
        }
    }
}
