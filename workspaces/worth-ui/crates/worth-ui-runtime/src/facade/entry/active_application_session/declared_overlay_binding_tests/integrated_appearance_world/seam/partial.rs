use super::super::World;
use super::{mounted_surface, seam_batch, surface_frame};
use worth_ui_host_contract::{UiMountedAppearanceColor, UiMountedSurfaceBorderSide};

#[test]
fn partial_shared_edge_omits_only_the_losing_interval_in_both_hosts() {
    let mut world = World::launch_seam();
    let placements = [
        (world.instances[0], [0.0, 40.0, 100.0, 20.0]),
        (world.instances[1], [100.0, 0.0, 100.0, 100.0]),
        (world.instances[2], [400.0, 0.0, 100.0, 100.0]),
        (world.instances[4], [600.0, 0.0, 100.0, 100.0]),
    ];
    let batch = seam_batch(&mut world, 4, &placements);
    world
        .session
        .begin_mounted_layout()
        .complete_surface_geometry(batch)
        .unwrap();
    let frame = world.prepare_surface(world.surfaces[0]);
    let profile = worth_ui_host_native::appearance_capability_report();
    let output =
        frame.lower_unpublished_appearance_with_profile_for_test(profile.appearance_profile());
    let owner = mounted_surface(&output, world.instances[0]);
    let loser = mounted_surface(&output, world.instances[1]);

    assert!(loser.border_edges().left());
    assert_eq!(loser.border_omissions().len(), 1);
    let omission = loser.border_omissions()[0];
    assert_eq!(omission.side(), UiMountedSurfaceBorderSide::Left);
    assert_eq!((omission.start(), omission.end()), (40_000, 60_000));
    assert_native_partial_edge(&owner, &loser);
    assert_headless_partial_edge(&output, world.instances[0], world.instances[1]);
    let _ = world.session.shutdown();
}

#[test]
fn changed_partial_seam_emits_replacement_damage_and_new_host_coverage() {
    let mut world = World::launch_seam();
    let declarations = super::region_declarations(&world);
    super::super::geometry::install_seam_pair(
        &mut world.session,
        world.surfaces,
        world.instances,
        declarations,
    );
    let initial = world.prepare_surface(world.surfaces[0]);
    let profile = worth_ui_host_native::appearance_capability_report();
    let before_output =
        initial.lower_unpublished_appearance_with_profile_for_test(profile.appearance_profile());
    let before = mounted_surface(&before_output, world.instances[1]);
    world.publish(initial, 100, true);

    let placements = [
        (world.instances[0], [0.0, 40.0, 100.0, 20.0]),
        (world.instances[1], [100.0, 0.0, 100.0, 100.0]),
        (world.instances[2], [400.0, 0.0, 100.0, 100.0]),
        (world.instances[4], [600.0, 0.0, 100.0, 100.0]),
    ];
    let batch = seam_batch(&mut world, 5, &placements);
    world
        .session
        .begin_mounted_layout()
        .complete_surface_geometry(batch)
        .unwrap();
    let successor = world.prepare_surface(world.surfaces[0]);
    let output =
        successor.lower_unpublished_appearance_with_profile_for_test(profile.appearance_profile());
    let after = mounted_surface(&output, world.instances[1]);
    let work = output
        .fragments()
        .iter()
        .find(|fragment| {
            matches!(
                fragment.identity(),
                worth_ui_host_contract::UiUnpublishedAppearanceFragmentIdentity::NodeReceipt {
                    successor: Some(receipt), ..
                } if receipt.mounted_instance() == world.instances[1]
            )
        })
        .expect("the changed loser owns exact delta work")
        .work();
    assert_eq!(
        work.posture(),
        worth_ui_host_contract::UiMountedAppearanceWorkPosture::Delta
    );
    assert!(work.changes().iter().any(|change| matches!(
        change,
        worth_ui_host_contract::UiMountedAppearanceMechanicChange::Replace {
            successor: worth_ui_host_contract::UiMountedAppearanceMechanic::Surface(_),
            ..
        }
    )));
    assert!(!work.damage().is_empty());
    assert_eq!(native_left_border(&before, 20), 0);
    assert!(native_left_border(&after, 20) > 0);
    let _ = world.session.shutdown();
}

fn native_left_border(
    surface: &worth_ui_host_contract::UiMountedSurfaceAppearanceMechanic,
    y: i64,
) -> u16 {
    worth_ui_host_native::certify_mounted_surface_sample(surface, 1_000, 100, y)
        .unwrap()
        .border_coverage()
}

fn assert_native_partial_edge(
    owner: &worth_ui_host_contract::UiMountedSurfaceAppearanceMechanic,
    loser: &worth_ui_host_contract::UiMountedSurfaceAppearanceMechanic,
) {
    for scale in [1_000, 1_250, 1_500, 2_000] {
        let boundary = i64::from(scale) / 10;
        let seam_y = i64::from(scale) / 20;
        let gap_y = i64::from(scale) / 50;
        let owner_seam = worth_ui_host_native::certify_mounted_surface_sample(
            owner,
            scale,
            boundary - 1,
            seam_y,
        )
        .unwrap();
        let loser_seam =
            worth_ui_host_native::certify_mounted_surface_sample(loser, scale, boundary, seam_y)
                .unwrap();
        let loser_gap =
            worth_ui_host_native::certify_mounted_surface_sample(loser, scale, boundary, gap_y)
                .unwrap();
        assert!(owner_seam.border_coverage() > 0);
        assert_eq!(loser_seam.border_coverage(), 0);
        assert!(loser_gap.border_coverage() > 0);
    }
}

fn assert_headless_partial_edge(
    output: &worth_ui_host_contract::UiUnpublishedAppearanceFrameProjection,
    owner: worth_ui_host_contract::UiMountedInstanceIdentity,
    loser: worth_ui_host_contract::UiMountedInstanceIdentity,
) {
    let transcript =
        worth_ui_host_headless::translate_unpublished_appearance_for_certification(output).unwrap();
    let owner_frame = surface_frame(&transcript, owner);
    let loser_frame = surface_frame(&transcript, loser);
    let transparent = UiMountedAppearanceColor::from_straight_srgba([0; 4]);
    let loser_center = loser_frame
        .reference_surface_at(loser, 150_000, 50_000)
        .unwrap();
    assert_ne!(
        owner_frame.reference_surface_at(owner, 99_500, 50_000),
        Some(transparent)
    );
    assert_eq!(
        loser_frame.reference_surface_at(loser, 100_500, 50_000),
        Some(loser_center)
    );
    assert_ne!(
        loser_frame.reference_surface_at(loser, 100_500, 20_000),
        Some(loser_center)
    );
}
