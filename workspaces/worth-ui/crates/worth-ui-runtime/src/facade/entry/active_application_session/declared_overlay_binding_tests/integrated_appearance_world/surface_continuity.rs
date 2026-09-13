use super::session::World;
use crate::mounting::*;
use worth_ui_host_contract::*;

#[test]
fn visible_neighbor_survives_predecessor_eviction_and_rejected_publication() {
    let defaults = UiMountedFrameRetentionBudget::default();
    let budget = UiMountedFrameRetentionBudget::new(UiMountedFrameRetentionBudgetInput {
        current: defaults.current(),
        in_flight: defaults.in_flight(),
        observation_basis: defaults.observation_basis(),
        predecessor_inspection: UiMountedRetentionClassBudget::new(2, 256 * 1024 * 1024),
        diagnostic: defaults.diagnostic(),
        visual_snapshot: defaults.visual_snapshot(),
        visual_overlay: defaults.visual_overlay(),
        expired_identity_limit: 64,
    });
    let mut world = World::launch_with_retention_budget(budget);
    let frame = world.prepare();
    world.publish(frame, 1, true);
    let neighbor = world
        .session
        .mounted
        .current_presentation_for_surface(world.surfaces[1])
        .unwrap();
    let first = world.prepare_surface(world.surfaces[0]);
    world.publish(first, 2, false);
    let obsolete = world
        .session
        .mounted
        .current_presentation_for_surface(world.surfaces[0])
        .unwrap();
    for tick in 3..7 {
        let frame = world.prepare_surface(world.surfaces[0]);
        world.publish(frame, tick, false);
        assert_eq!(
            world
                .session
                .mounted
                .current_presentation_for_surface(world.surfaces[1]),
            Some(neighbor)
        );
        assert_eq!(
            world
                .session
                .mounted
                .current_semantic_surface_for_presentation(neighbor),
            Ok(world.surfaces[1])
        );
        let mut work = UiHitTestSpatialWork::default();
        let row = world
            .session
            .mounted
            .current_presented_hit_row(neighbor, world.instances[3], &mut work)
            .unwrap();
        assert_eq!(row.mounted_instance(), world.instances[3]);
        assert!(
            world
                .session
                .mounted
                .admit_current_interaction_affinity(UiMountedInteractionAffinityInput {
                    surface: world.surfaces[1],
                    binding: neighbor.binding(),
                    mounted_instance: world.instances[3],
                    node_receipt: row.mounted().node_receipt(),
                })
                .is_ok(),
            "the live input target keeps the receipt actually presented on its surface"
        );
        assert!(
            world
                .session
                .mounted
                .current_presented_incarnation_receipt(
                    UiMountedIncarnationAffinityInput {
                        surface: world.surfaces[1],
                        binding: neighbor.binding(),
                        mounted_instance: world.instances[3],
                    },
                    neighbor,
                )
                .is_ok(),
            "retained physical geometry must still resolve a live mounted incarnation"
        );
    }
    assert_eq!(
        world
            .session
            .mounted
            .classify_interaction_presentation(obsolete),
        Err(UiPresentedFrameBasisDenial::Expired),
        "the small budget must actually evict superseded evidence"
    );
    let before = world
        .session
        .mounted
        .current_presentation_for_surface(world.surfaces[0])
        .unwrap();
    let frame = world.prepare_surface(world.surfaces[0]);
    world.host.push_rejected();
    let rejected = world.session.present_prepared_mounted_frame_internal(
        frame,
        UiPresentationDeadline::at_tick(u64::MAX),
        7,
    );
    assert!(matches!(
        rejected,
        UiMountedFrameOutcome::RejectedBeforeEffects(_)
    ));
    drop(rejected);
    assert_eq!(
        world
            .session
            .mounted
            .current_presentation_for_surface(world.surfaces[0]),
        Some(before)
    );
    assert_eq!(
        world
            .session
            .mounted
            .current_presentation_for_surface(world.surfaces[1]),
        Some(neighbor)
    );
    let frame = world.prepare_surface(world.surfaces[0]);
    world.publish(frame, 8, false);
    assert_eq!(
        world
            .session
            .mounted
            .current_presentation_for_surface(world.surfaces[1]),
        Some(neighbor)
    );
    let frame = world.prepare_surface(world.surfaces[1]);
    world.publish(frame, 9, false);
    assert!(world
        .session
        .mounted
        .current_semantic_surface_for_presentation(neighbor)
        .is_err());
    assert_eq!(
        world
            .session
            .mounted
            .classify_interaction_presentation(neighbor),
        Err(UiPresentedFrameBasisDenial::Expired),
        "repainting the last dependent surface releases its old evidence for eviction"
    );
    let _ = world.session.shutdown();
    assert_eq!(world.host.pending_presentation_count(), 0);
}
