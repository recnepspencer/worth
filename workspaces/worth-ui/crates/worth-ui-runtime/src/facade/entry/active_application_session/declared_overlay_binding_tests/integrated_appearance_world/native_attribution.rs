use super::World;
use worth_ui_host_contract::{UiMountedFrameIdentity, UiPresentationDeadline};

#[test]
fn native_observed_provenance_requires_exact_accepted_surface_and_receipt() {
    let mut world = World::launch();
    for _ in world.surfaces {
        world.host.push_native_display_presented();
    }
    let request = world.session.mounted_frame_request();
    let outcome = world
        .session
        .execute_mounted_frame(
            request,
            UiPresentationDeadline::at_tick(u64::MAX),
            1,
            |_| {},
        )
        .unwrap_or_else(|_| panic!("public mounted entry prepares the observed world"));
    assert!(matches!(
        outcome,
        crate::mounting::UiMountedFrameOutcome::Published(_)
    ));
    drop(outcome);
    let projection = world
        .session
        .mounted
        .current_projection_rc_for_test()
        .unwrap();
    for (surface, instance) in [(0, 0), (1, 3)] {
        let presentation = world
            .session
            .mounted
            .current_presentation_for_surface(world.surfaces[surface])
            .unwrap();
        let view = projection.view_for(presentation.binding()).unwrap();
        let node = view
            .nodes()
            .iter()
            .find(|node| node.mounted_instance() == world.instances[instance])
            .unwrap();
        let receipt = node.node_receipt().diagnostic_value();
        let instance = node.mounted_instance().diagnostic_value();
        let mounted = &world.session.mounted;
        let valid = mounted
            .native_observed_paint_attribution(
                view.frame(),
                presentation.binding(),
                view.surface().diagnostic_value(),
                instance,
                receipt,
            )
            .expect("accepted mounting attaches provenance to the observed node");
        assert_eq!(valid.mounted_instance, node.mounted_instance());
        assert_eq!(valid.node_receipt, node.node_receipt());
        for (frame, observed_surface, observed_instance, observed_receipt) in [
            (
                UiMountedFrameIdentity::mint_unbound().unwrap(),
                view.surface().diagnostic_value(),
                instance,
                receipt,
            ),
            (
                view.frame(),
                world.surfaces[1 - surface].diagnostic_value(),
                instance,
                receipt,
            ),
            (
                view.frame(),
                view.surface().diagnostic_value(),
                world.instances[2].diagnostic_value(),
                receipt,
            ),
            (
                view.frame(),
                view.surface().diagnostic_value(),
                instance,
                receipt + 1,
            ),
        ] {
            assert!(
                mounted
                    .native_observed_paint_attribution(
                        frame,
                        presentation.binding(),
                        observed_surface,
                        observed_instance,
                        observed_receipt,
                    )
                    .is_none(),
                "an observation cannot manufacture accepted mounting provenance"
            );
        }
    }
    let _ = world.session.shutdown();
    assert_eq!(world.host.pending_presentation_count(), 0);
}
