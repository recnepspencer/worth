use super::*;
use crate::mounting::presentation::motion_sampling::UiMountedMotionSampler;
use crate::mounting::spatial_index::UiMountedSpatialBudget;
use crate::runtime::motion::{UiMotionCommitReceipt, UiMotionDeclaration, UiMotionTargetIdentity};

#[test]
fn presented_index_portal_motion_selects_only_children_and_keeps_trigger_stationary() {
    let owner = UiMountedInstanceIdentity::mint_unbound().unwrap();
    let child = UiMountedInstanceIdentity::mint_unbound().unwrap();
    let surface = UiSemanticSurfaceIdentity::mint_unbound().unwrap();
    let binding = UiSurfaceBindingGeneration::mint_unbound().unwrap();
    let frame = UiMountedFrameIdentity::mint_unbound().unwrap();
    let (fonts, _) = worth_ui_text::UiGlobalFontCollection::admit_qualified_profile().unwrap();
    let projection = projection_frame_with_identity(
        frame,
        portal_semantic_projection(owner, child, surface, binding),
        surface,
        binding,
        owner,
        child,
        Arc::new(fonts),
        Default::default(),
        vec![portal_overlay(frame, owner, surface, binding)],
        1,
    );
    let basis = projection
        .visual_region_basis()
        .for_binding(binding, projection.receipt_basis.clone());
    let hits = basis.hit_test();
    let portal = hits
        .iter()
        .find(|hit| hit.mechanic().mounted_instance() == child)
        .unwrap()
        .portal()
        .unwrap();
    let presentation = UiHostObservationPresentationBasis::new(
        UiHostSurfaceIdentity::mint_unbound().unwrap(),
        frame,
        binding,
        UiHostPresentationEpoch::issued_by_host(1),
    );
    let target =
        UiMotionTargetIdentity::from_family_owner(surface, owner, portal.portal_identity());
    let b = portal.bounds();
    for size in [64, 4096] {
        let mut index = basis.presented_hits.clone();
        // Canonically completed unrelated rows make a broad surface scan observable.
        for n in 0..size {
            let unrelated = UiMotionTargetIdentity::from_family_owner(
                surface,
                UiMountedInstanceIdentity::mint_unbound().unwrap(),
                77,
            );
            let hit = crate::mounting::retention::motion_sampling_hit_test_mechanic_for_test(
                presentation,
                unrelated,
                [2000.0 + n as f32 * 20.0, 0.0, 10.0, 10.0],
            );
            let row = crate::mounting::UiPresentedHitTestRow::from_mounted(
                crate::mounting::UiMountedHitTestPresentation::for_test(hit),
            );
            index.replace_base(row.mounted_instance(), Some(row));
        }
        let retained = index.clone();
        let reserved = index.retained_structural_bytes();
        let mut sampler = UiMountedMotionSampler::default();
        sampler
            .install(UiMotionCommitReceipt::for_sampling_test_transition(
                501,
                target,
                presentation,
                Some([b.x(), b.y() + 1000.0, b.width(), b.height()]),
                true,
                Some([b.x(), b.y(), b.width(), b.height()]),
                true,
                UiMotionDeclaration::portal_entrance(),
                None,
            ))
            .unwrap();
        let prepared = sampler.prepare_tick(1, presentation).unwrap();
        sampler.commit_prepared(prepared);
        let work = index.apply_motion(&sampler, presentation, &[target]);
        assert_eq!(work.motion_members_visited, 1);
        assert_eq!(work.motion_rows_projected, 1);
        assert_eq!(
            work.motion_tracks_considered, 0,
            "Portal uses exact target lookup"
        );
        assert!(work.node_copies < 128 && work.map_key_probes < 128);
        assert_eq!(index.retained_structural_bytes(), reserved);
        assert!(!has(&index, binding, [40.0, 80.0], child));
        assert!(has(&index, binding, [40.0, 1080.0], child));
        assert!(has(&retained, binding, [40.0, 80.0], child));
        assert!(has(&index, binding, [40.0, 30.0], owner));
        assert!(!has(&index, binding, [40.0, 1030.0], owner));
        let prepared = sampler.prepare_tick(500, presentation).unwrap();
        sampler.commit_prepared(prepared);
        index.apply_motion(&sampler, presentation, &[target]);
        assert!(has(&index, binding, [40.0, 80.0], child));
        assert!(!has(&index, binding, [40.0, 1080.0], child));
    }
}

fn has(
    index: &crate::mounting::presented_hit_index::UiPresentedHitIndex,
    binding: UiSurfaceBindingGeneration,
    point: [f64; 2],
    instance: UiMountedInstanceIdentity,
) -> bool {
    index
        .at_point(
            binding,
            point,
            UiMountedSpatialBudget {
                node_visits: 1024,
                candidates: 256,
            },
        )
        .unwrap()
        .rows
        .iter()
        .any(|row| row.mounted_instance() == instance)
}
