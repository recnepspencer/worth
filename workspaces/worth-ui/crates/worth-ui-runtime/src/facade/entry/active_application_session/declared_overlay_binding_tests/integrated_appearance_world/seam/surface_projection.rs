use super::super::World;
use super::mounted_surface;
use worth_ui_host_contract::UiMountedSurfaceBorderSide;

const PRIMARY: &str = "workspace.region.primary";
const SECONDARY: &str = "workspace.region.secondary";

#[test]
fn offset_region_omission_uses_owner_surface_coordinates() {
    let mut world = World::launch_seam();
    let placements = [
        (world.instances[0], [0.0, 108.0, 10.0, 4.0]),
        (world.instances[1], [10.0, 100.0, 20.0, 20.0]),
        (world.instances[2], [100.0, 0.0, 10.0, 10.0]),
        (world.instances[4], [200.0, 0.0, 10.0, 10.0]),
    ];
    let regions = [
        (world.instances[0], PRIMARY, [0.0, 0.0, 10.0, 4.0]),
        (world.instances[1], SECONDARY, [0.0, 5.0, 10.0, 10.0]),
        (world.instances[2], PRIMARY, [0.0, 0.0, 10.0, 10.0]),
        (world.instances[4], PRIMARY, [0.0, 0.0, 10.0, 10.0]),
    ];
    let receipt = submit_custom_regions(&mut world, 6, &placements, &regions).unwrap();
    assert_eq!(receipt.seam_adjacencies_visited(), 1);

    let target = world.instances[1];
    let loser = mounted_output_surface(&mut world, target);
    assert!(loser.border_edges().left());
    assert_eq!(loser.border_omissions().len(), 1);
    let omission = loser.border_omissions()[0];
    assert_eq!(omission.side(), UiMountedSurfaceBorderSide::Left);
    assert_eq!((omission.start(), omission.end()), (8_000, 12_000));
    assert_eq!(native_surface_border(&loser, 10, 109), 0);
    let _ = world.session.shutdown();
}

#[test]
fn internal_region_seam_preserves_the_owner_outer_border() {
    let mut world = World::launch_seam();
    let placements = [
        (world.instances[0], [0.0, 5.0, 10.0, 10.0]),
        (world.instances[1], [0.0, 0.0, 20.0, 20.0]),
        (world.instances[2], [100.0, 0.0, 10.0, 10.0]),
        (world.instances[4], [200.0, 0.0, 10.0, 10.0]),
    ];
    let regions = [
        (world.instances[0], PRIMARY, [0.0, 0.0, 10.0, 10.0]),
        (world.instances[1], SECONDARY, [10.0, 5.0, 10.0, 10.0]),
        (world.instances[2], PRIMARY, [0.0, 0.0, 10.0, 10.0]),
        (world.instances[4], PRIMARY, [0.0, 0.0, 10.0, 10.0]),
    ];
    let receipt = submit_custom_regions(&mut world, 6, &placements, &regions).unwrap();
    assert_eq!(receipt.seam_adjacencies_visited(), 1);

    let target = world.instances[1];
    let owner = mounted_output_surface(&mut world, target);
    assert_eq!(
        owner.border_edges(),
        worth_ui_host_contract::UiMountedSurfaceBorderEdges::ALL
    );
    assert!(owner.border_omissions().is_empty());
    let _ = world.session.shutdown();
}

#[test]
fn internal_winning_region_cannot_remove_the_only_exterior_border() {
    let mut world = World::launch_seam();
    let placements = [
        (world.instances[0], [0.0, 0.0, 20.0, 20.0]),
        (world.instances[1], [10.0, 5.0, 10.0, 10.0]),
        (world.instances[2], [100.0, 0.0, 10.0, 10.0]),
        (world.instances[4], [200.0, 0.0, 10.0, 10.0]),
    ];
    let regions = [
        (world.instances[0], PRIMARY, [0.0, 5.0, 10.0, 10.0]),
        (world.instances[1], SECONDARY, [0.0, 0.0, 10.0, 10.0]),
        (world.instances[2], PRIMARY, [0.0, 0.0, 10.0, 10.0]),
        (world.instances[4], PRIMARY, [0.0, 0.0, 10.0, 10.0]),
    ];
    let receipt = submit_custom_regions(&mut world, 6, &placements, &regions).unwrap();
    assert_eq!(receipt.region_lookup_steps(), 24);

    let target = world.instances[1];
    let exterior_owner = mounted_output_surface(&mut world, target);
    assert_eq!(
        exterior_owner.border_edges(),
        worth_ui_host_contract::UiMountedSurfaceBorderEdges::ALL
    );
    assert!(exterior_owner.border_omissions().is_empty());
    let _ = world.session.shutdown();
}

#[test]
fn one_owner_with_multiple_regions_aggregates_only_exterior_omissions() {
    let mut world = World::launch_multi_region_seam();
    let placements = [
        (world.instances[0], [10.0, 0.0, 20.0, 20.0]),
        (world.instances[1], [100.0, 0.0, 10.0, 10.0]),
        (world.instances[2], [0.0, 10.0, 10.0, 5.0]),
        (world.instances[4], [200.0, 0.0, 10.0, 10.0]),
    ];
    let regions = [
        (world.instances[0], PRIMARY, [0.0, 0.0, 10.0, 10.0]),
        (world.instances[0], SECONDARY, [0.0, 10.0, 10.0, 10.0]),
        (world.instances[1], SECONDARY, [0.0, 0.0, 10.0, 10.0]),
        (world.instances[2], PRIMARY, [0.0, 0.0, 10.0, 5.0]),
        (world.instances[4], PRIMARY, [0.0, 0.0, 10.0, 10.0]),
    ];
    let receipt = submit_custom_regions(&mut world, 6, &placements, &regions).unwrap();
    assert_eq!(receipt.region_lookup_steps(), 22);

    let owner = world
        .session
        .mounted
        .current_surface_paint_posture_for_test(world.instances[0])
        .unwrap();
    assert!(owner.border_edges().left());
    assert!(owner.border_edges().top());
    assert_eq!(owner.border_omissions().len(), 1);
    let omission = owner.border_omissions()[0];
    assert_eq!(omission.side(), UiMountedSurfaceBorderSide::Left);
    assert_eq!((omission.start(), omission.end()), (10.0, 15.0));

    let incomplete_regions = [regions[0], regions[2], regions[3], regions[4]];
    assert_eq!(
        submit_custom_regions(&mut world, 7, &placements, &incomplete_regions).unwrap_err(),
        crate::mounting::UiMountedOccurrenceGeometryDenial::MissingMosaicRegionGeometry
    );
    assert_eq!(
        world
            .session
            .mounted
            .current_surface_paint_posture_for_test(world.instances[0])
            .as_ref(),
        Some(&owner)
    );
    let duplicate_regions = [
        regions[0], regions[0], regions[1], regions[2], regions[3], regions[4],
    ];
    assert_eq!(
        submit_custom_regions(&mut world, 8, &placements, &duplicate_regions).unwrap_err(),
        crate::mounting::UiMountedOccurrenceGeometryDenial::DuplicateRegionOccurrence
    );
    assert_eq!(
        world
            .session
            .mounted
            .current_surface_paint_posture_for_test(world.instances[0])
            .as_ref(),
        Some(&owner)
    );
    let _ = world.session.shutdown();
}

fn mounted_output_surface(
    world: &mut World,
    instance: worth_ui_host_contract::UiMountedInstanceIdentity,
) -> worth_ui_host_contract::UiMountedSurfaceAppearanceMechanic {
    let frame = world.prepare_surface(world.surfaces[0]);
    let profile = worth_ui_host_native::appearance_capability_report();
    let output =
        frame.lower_unpublished_appearance_with_profile_for_test(profile.appearance_profile());
    mounted_surface(&output, instance)
}

fn submit_custom_regions(
    world: &mut World,
    revision: u64,
    placements: &[(worth_ui_host_contract::UiMountedInstanceIdentity, [f32; 4])],
    region_specs: &[(
        worth_ui_host_contract::UiMountedInstanceIdentity,
        &str,
        [f32; 4],
    )],
) -> Result<
    crate::mounting::UiMountedLayoutCompletionReceipt,
    crate::mounting::UiMountedOccurrenceGeometryDenial,
> {
    let surface = world.surfaces[0];
    let surface_declaration = world
        .session
        .application
        .authored_overlay_material()
        .overlay_declaration_bindings()
        .surface_named("workspace.surface.overlay")
        .unwrap();
    let occurrences = placements
        .iter()
        .map(|(instance, bounds)| {
            crate::mounting::UiMountedOccurrenceGeometry::surface(
                *instance,
                super::super::geometry::canonical(*bounds),
            )
        })
        .collect::<Vec<_>>();
    let regions = region_specs
        .iter()
        .map(|(instance, kind, bounds)| {
            let mounted = world
                .session
                .mounted
                .current_mounted_identity_basis(*instance)
                .unwrap();
            let bindings = world
                .session
                .application
                .mounted_region_declarations(surface_declaration, mounted.graph_node_identity());
            let binding = bindings
                .0
                .iter()
                .find(|binding| binding.region_kind() == *kind)
                .unwrap();
            crate::mounting::UiMountedMosaicRegionGeometry::new(
                *instance,
                binding.declaration(),
                binding.executed_region(),
                super::canonical_local(*bounds),
            )
        })
        .collect::<Vec<_>>();
    let basis = world.session.begin_mounted_layout().basis(surface).unwrap();
    let batch = crate::mounting::UiMountedSurfaceGeometryBatch::new(
        basis,
        crate::mounting::UiMountedLayoutRevision::new(revision).unwrap(),
        super::geometry::canonical([0.0, 0.0, 1_000.0, 1_000.0]),
        occurrences,
    )
    .with_regions(regions);
    world
        .session
        .begin_mounted_layout()
        .complete_surface_geometry(batch)
}

fn native_surface_border(
    surface: &worth_ui_host_contract::UiMountedSurfaceAppearanceMechanic,
    x: i64,
    y: i64,
) -> u16 {
    worth_ui_host_native::certify_mounted_surface_sample(surface, 1_000, x, y)
        .unwrap()
        .border_coverage()
}
