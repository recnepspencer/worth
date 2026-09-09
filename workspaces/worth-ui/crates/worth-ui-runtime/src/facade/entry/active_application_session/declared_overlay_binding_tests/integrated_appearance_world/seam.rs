use super::{geometry, World};
use worth_ui_host_contract::{
    UiMountedAppearanceColor, UiMountedAppearanceMechanic, UiMountedInstanceIdentity,
    UiMountedSurfaceAppearanceMechanic, UiUnpublishedAppearanceFrameProjection,
};

use crate::mounting::{
    UiMountedLayoutRevision, UiMountedOccurrenceGeometry, UiMountedSurfaceGeometryBatch,
};

#[path = "seam/partial.rs"]
mod partial;
#[path = "seam/surface_projection.rs"]
mod surface_projection;

#[test]
fn mounted_mosaic_seam_paints_once_with_exterior_radii_at_qualified_scales() {
    let mut world = World::launch_seam();
    let declarations = region_declarations(&world);
    geometry::install_seam_pair(
        &mut world.session,
        world.surfaces,
        world.instances,
        declarations,
    );
    let frame = world.prepare_surface(world.surfaces[0]);
    let profile = worth_ui_host_native::staged_appearance_capability_report();
    let output =
        frame.lower_unpublished_appearance_with_profile_for_test(profile.appearance_profile());
    let owner = mounted_surface(&output, world.instances[0]);
    let loser = mounted_surface(&output, world.instances[1]);

    assert_eq!(owner.radii().corners(), [10_000, 0, 0, 10_000]);
    assert_eq!(loser.radii().corners(), [0, 10_000, 10_000, 0]);
    assert!(owner.border_edges().right());
    assert!(!loser.border_edges().left());
    assert!(loser.border_edges().top() && loser.border_edges().right());
    assert!(loser.border_edges().bottom());
    assert_native_seam_samples(&owner, &loser);
    assert_headless_seam_samples(&output, world.instances[0], world.instances[1]);
    let _ = world.session.shutdown();
}

#[test]
fn same_kind_shared_edge_denial_preserves_the_mounted_predecessor() {
    let mut world = World::launch_seam();
    let declarations = region_declarations(&world);
    geometry::install_seam_pair(
        &mut world.session,
        world.surfaces,
        world.instances,
        declarations,
    );
    let target = world.instances[0];
    let before = current_surface(&mut world, target);
    let placements = [
        (world.instances[0], [0.0, 0.0, 100.0, 100.0]),
        (world.instances[2], [100.0, 0.0, 100.0, 100.0]),
        (world.instances[1], [300.0, 0.0, 100.0, 100.0]),
        (world.instances[4], [500.0, 0.0, 100.0, 100.0]),
    ];
    let denied = seam_batch(&mut world, 5, &placements);
    assert_eq!(
        world
            .session
            .begin_mounted_layout()
            .complete_surface_geometry(denied),
        Err(crate::mounting::UiMountedOccurrenceGeometryDenial::UndeclaredMosaicSharedEdge)
    );
    let after = current_surface(&mut world, target);
    assert_same_surface_meaning(&after, &before);
    let _ = world.session.shutdown();
}

fn assert_same_surface_meaning(
    left: &UiMountedSurfaceAppearanceMechanic,
    right: &UiMountedSurfaceAppearanceMechanic,
) {
    assert_eq!(left.bounds(), right.bounds());
    assert_eq!(left.clip(), right.clip());
    assert_eq!(left.surface_paint_order(), right.surface_paint_order());
    assert_eq!(left.radii(), right.radii());
    assert_eq!(left.border_edges(), right.border_edges());
    assert_eq!(left.border_omissions(), right.border_omissions());
    assert_eq!(left.paint(), right.paint());
    assert_eq!(left.opacity(), right.opacity());
    assert_eq!(left.projection().identity(), right.projection().identity());
    assert_eq!(left.projection().revision(), right.projection().revision());
}

#[test]
fn seam_boundary_index_reports_sparse_work_at_512_occurrences() {
    let mut world = World::launch_seam();
    let mut primary = vec![world.instances[0], world.instances[2], world.instances[4]];
    let mut secondary = vec![world.instances[1]];
    for _ in 0..254 {
        primary.push(
            world
                .session
                .mount_instance(
                    world.session.mounted_graph_node(world.graphs[0]).unwrap(),
                    world.surfaces[0],
                )
                .unwrap(),
        );
        secondary.push(
            world
                .session
                .mount_instance(
                    world.session.mounted_graph_node(world.graphs[1]).unwrap(),
                    world.surfaces[0],
                )
                .unwrap(),
        );
    }
    assert_eq!(primary.len() + secondary.len(), 512);
    let mut placements = Vec::with_capacity(512);
    for (index, (primary, secondary)) in primary
        .iter()
        .copied()
        .zip(secondary.iter().copied())
        .enumerate()
    {
        let x = index as f32 * 3.0;
        placements.push((primary, [x, 0.0, 1.0, 1.0]));
        placements.push((secondary, [x + 1.0, 0.0, 1.0, 1.0]));
    }
    for (index, primary) in primary.iter().copied().skip(secondary.len()).enumerate() {
        placements.push((primary, [2_000.0 + index as f32 * 2.0, 0.0, 1.0, 1.0]));
    }
    let batch = seam_batch(&mut world, 4, &placements);
    let receipt = world
        .session
        .begin_mounted_layout()
        .complete_surface_geometry(batch)
        .unwrap();
    assert_eq!(receipt.region_lookup_steps(), 3_072);
    assert_eq!(receipt.seam_index_rows(), 2_048);
    assert_eq!(receipt.seam_adjacencies_visited(), 255);
    let _ = world.session.shutdown();
}

fn current_surface(
    world: &mut World,
    target: UiMountedInstanceIdentity,
) -> UiMountedSurfaceAppearanceMechanic {
    let frame = world.prepare_surface(world.surfaces[0]);
    let profile = worth_ui_host_native::staged_appearance_capability_report();
    let output =
        frame.lower_unpublished_appearance_with_profile_for_test(profile.appearance_profile());
    mounted_surface(&output, target)
}

pub(super) fn seam_batch(
    world: &mut World,
    revision: u64,
    placements: &[(UiMountedInstanceIdentity, [f32; 4])],
) -> UiMountedSurfaceGeometryBatch {
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
            UiMountedOccurrenceGeometry::surface(*instance, geometry::canonical(*bounds))
        })
        .collect::<Vec<_>>();
    let regions = placements
        .iter()
        .map(|(instance, bounds)| {
            let mounted = world
                .session
                .mounted
                .current_mounted_identity_basis(*instance)
                .unwrap();
            let bindings = world
                .session
                .application
                .mounted_region_declarations(surface_declaration, mounted.graph_node_identity());
            let binding = bindings.0.iter().next().unwrap();
            crate::mounting::UiMountedMosaicRegionGeometry::new(
                *instance,
                binding.declaration(),
                binding.executed_region(),
                canonical_local([0.0, 0.0, bounds[2], bounds[3]]),
            )
        })
        .collect::<Vec<_>>();
    let basis = world.session.begin_mounted_layout().basis(surface).unwrap();
    UiMountedSurfaceGeometryBatch::new(
        basis,
        UiMountedLayoutRevision::new(revision).unwrap(),
        geometry::canonical([0.0, 0.0, 3_000.0, 10.0]),
        occurrences,
    )
    .with_regions(regions)
}

fn canonical_local(
    [x, y, width, height]: [f32; 4],
) -> worth_ui_host_contract::UiMountedCanonicalBox {
    worth_ui_host_contract::UiMountedCanonicalBox::canonicalize(
        worth_ui_host_contract::UiMountedCanonicalBoxInput {
            x,
            y,
            width,
            height,
            coordinate_space: worth_ui_host_contract::UiMountedCoordinateSpace::GraphNodeLocal,
        },
    )
    .unwrap()
}

pub(super) fn mounted_surface(
    output: &UiUnpublishedAppearanceFrameProjection,
    target: UiMountedInstanceIdentity,
) -> UiMountedSurfaceAppearanceMechanic {
    output
        .fragments()
        .iter()
        .flat_map(|fragment| fragment.work().successor().mechanics())
        .find_map(|mechanic| match mechanic {
            UiMountedAppearanceMechanic::Surface(surface)
                if surface.node_receipt().mounted_instance() == target =>
            {
                Some(surface.clone())
            }
            _ => None,
        })
        .expect("the mounted Mosaic occurrence emits its Surface mechanic")
}

fn assert_native_seam_samples(
    owner: &UiMountedSurfaceAppearanceMechanic,
    loser: &UiMountedSurfaceAppearanceMechanic,
) {
    for (scale, boundary) in [(1_000, 100), (1_250, 125), (1_500, 150), (2_000, 200)] {
        let y = boundary / 2;
        let owner_seam =
            worth_ui_host_native::certify_mounted_surface_sample(owner, scale, boundary - 1, y)
                .unwrap();
        let loser_seam =
            worth_ui_host_native::certify_mounted_surface_sample(loser, scale, boundary, y)
                .unwrap();
        assert!(owner_seam.border_coverage() > 0);
        assert_eq!(loser_seam.border_coverage(), 0);

        let owner_exterior =
            worth_ui_host_native::certify_mounted_surface_sample(owner, scale, 0, 0).unwrap();
        let owner_interior =
            worth_ui_host_native::certify_mounted_surface_sample(owner, scale, boundary - 1, 0)
                .unwrap();
        let loser_interior =
            worth_ui_host_native::certify_mounted_surface_sample(loser, scale, boundary, 0)
                .unwrap();
        let loser_exterior =
            worth_ui_host_native::certify_mounted_surface_sample(loser, scale, boundary * 2 - 1, 0)
                .unwrap();
        assert_eq!(owner_exterior.fill_coverage(), 0);
        assert!(owner_interior.fill_coverage() > 0);
        assert!(loser_interior.fill_coverage() > 0);
        assert_eq!(loser_exterior.fill_coverage(), 0);
    }
}

fn assert_headless_seam_samples(
    output: &UiUnpublishedAppearanceFrameProjection,
    owner: UiMountedInstanceIdentity,
    loser: UiMountedInstanceIdentity,
) {
    let transcript =
        worth_ui_host_headless::translate_unpublished_appearance_for_certification(output).unwrap();
    let owner_frame = surface_frame(&transcript, owner);
    let loser_frame = surface_frame(&transcript, loser);
    let owner_center = owner_frame
        .reference_surface_at(owner, 50_000, 50_000)
        .unwrap();
    let loser_center = loser_frame
        .reference_surface_at(loser, 150_000, 50_000)
        .unwrap();
    let owner_seam = owner_frame
        .reference_surface_at(owner, 99_500, 50_000)
        .unwrap();
    let loser_seam = loser_frame
        .reference_surface_at(loser, 100_500, 50_000)
        .unwrap();
    assert_ne!(
        owner_seam, owner_center,
        "the declared owner paints the seam"
    );
    assert_eq!(
        loser_seam, loser_center,
        "the losing side does not duplicate it"
    );
    let transparent = UiMountedAppearanceColor::from_straight_srgba([0; 4]);
    assert_eq!(
        owner_frame.reference_surface_at(owner, 500, 500).unwrap(),
        transparent
    );
    assert_ne!(
        owner_frame
            .reference_surface_at(owner, 99_500, 500)
            .unwrap(),
        transparent
    );
}

pub(super) fn surface_frame<'a>(
    transcript: &'a worth_ui_host_headless::UiHeadlessUnpublishedAppearanceFrameTranscript,
    target: UiMountedInstanceIdentity,
) -> &'a worth_ui_host_headless::UiHeadlessAppearanceFrameTranscript {
    transcript
        .fragments()
        .iter()
        .map(|fragment| fragment.work().successor())
        .find(|frame| {
            frame.mechanics().iter().any(|mechanic| {
                matches!(
                    mechanic,
                    worth_ui_host_headless::UiHeadlessAppearanceMechanic::Surface(surface)
                        if surface.node_receipt().mounted_instance() == target
                )
            })
        })
        .expect("the exact ordinary Surface transcript is present")
}

fn region_declarations(world: &World) -> [worth_ui_dsl::UiMosaicRegionDeclarationIdentity; 2] {
    let bindings = world
        .session
        .application
        .authored_overlay_material()
        .overlay_declaration_bindings();
    ["workspace.surface.overlay", "workspace.surface.secondary"].map(|surface| {
        bindings
            .region_named(surface, "workspace.region.primary")
            .unwrap()
    })
}
