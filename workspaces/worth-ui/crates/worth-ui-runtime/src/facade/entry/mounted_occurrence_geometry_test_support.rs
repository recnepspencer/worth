use worth_ui_host_contract::{
    UiMountedCanonicalBox, UiMountedCanonicalBoxInput, UiMountedCoordinateSpace,
    UiMountedInstanceIdentity, UiSemanticSurfaceIdentity,
};

pub(crate) fn install_nonoverlapping_surface_geometry(
    session: &mut super::WorthUiActiveApplicationSession,
    surface: UiSemanticSurfaceIdentity,
    layout_revision: u64,
    hierarchy: &[(UiMountedInstanceIdentity, Option<UiMountedInstanceIdentity>)],
) -> Vec<(UiMountedInstanceIdentity, UiMountedCanonicalBox)> {
    install_surface_geometry(
        session,
        surface,
        layout_revision,
        hierarchy,
        None,
        surface_viewport_bounds(),
    )
}

pub(crate) fn install_surface_geometry_with_viewport(
    session: &mut super::WorthUiActiveApplicationSession,
    surface: UiSemanticSurfaceIdentity,
    layout_revision: u64,
    viewport: UiMountedCanonicalBox,
) -> Vec<(UiMountedInstanceIdentity, UiMountedCanonicalBox)> {
    install_surface_geometry(session, surface, layout_revision, &[], None, viewport)
}

pub(crate) fn refresh_nonoverlapping_surface_geometry(
    session: &mut super::WorthUiActiveApplicationSession,
    surface: UiSemanticSurfaceIdentity,
) -> Vec<(UiMountedInstanceIdentity, UiMountedCanonicalBox)> {
    let revision = session
        .mounted
        .next_occurrence_geometry_revision_for_test(surface)
        .get();
    install_nonoverlapping_surface_geometry(session, surface, revision, &[])
}

pub(crate) fn install_shifted_surface_geometry(
    session: &mut super::WorthUiActiveApplicationSession,
    surface: UiSemanticSurfaceIdentity,
    layout_revision: u64,
    hierarchy: &[(UiMountedInstanceIdentity, Option<UiMountedInstanceIdentity>)],
    shifted_root: UiMountedInstanceIdentity,
    x_offset: f32,
) -> Vec<(UiMountedInstanceIdentity, UiMountedCanonicalBox)> {
    install_surface_geometry(
        session,
        surface,
        layout_revision,
        hierarchy,
        Some((shifted_root, x_offset)),
        surface_viewport_bounds(),
    )
}

fn install_surface_geometry(
    session: &mut super::WorthUiActiveApplicationSession,
    surface: UiSemanticSurfaceIdentity,
    layout_revision: u64,
    hierarchy: &[(UiMountedInstanceIdentity, Option<UiMountedInstanceIdentity>)],
    shift: Option<(UiMountedInstanceIdentity, f32)>,
    viewport: UiMountedCanonicalBox,
) -> Vec<(UiMountedInstanceIdentity, UiMountedCanonicalBox)> {
    let mounted = session.inspect_mounted_identity();
    let mut instances = mounted
        .mounted_instances()
        .iter()
        .filter(|instance| instance.basis().semantic_surface_identity() == surface)
        .map(|instance| instance.identity())
        .collect::<Vec<_>>();
    instances.sort_unstable();
    let mut ordered = hierarchy
        .iter()
        .map(|(instance, _)| *instance)
        .collect::<Vec<_>>();
    let remaining = instances
        .into_iter()
        .filter(|instance| !ordered.contains(instance))
        .collect::<Vec<_>>();
    ordered.extend(remaining);
    let mut expected = ordered
        .iter()
        .enumerate()
        .map(|(index, instance)| (*instance, expected_nonoverlapping_bounds(index)))
        .collect::<std::collections::BTreeMap<_, _>>();
    if let Some((shifted_root, x_offset)) = shift {
        for instance in &ordered {
            if belongs_to_subtree(*instance, shifted_root, hierarchy) {
                let bounds = expected[instance];
                expected.insert(
                    *instance,
                    UiMountedCanonicalBox::canonicalize(UiMountedCanonicalBoxInput {
                        x: bounds.x() + x_offset,
                        y: bounds.y(),
                        width: bounds.width(),
                        height: bounds.height(),
                        coordinate_space: UiMountedCoordinateSpace::HostSurface,
                    })
                    .expect("fixture subtree shift remains finite surface geometry"),
                );
            }
        }
    }
    let rows = ordered
        .iter()
        .map(|instance| {
            let bounds = expected[instance];
            let parent = hierarchy
                .iter()
                .find_map(|(candidate, parent)| (*candidate == *instance).then_some(*parent))
                .flatten();
            let occurrence = match parent {
                Some(parent) => {
                    let parent_bounds = expected[&parent];
                    let local = UiMountedCanonicalBox::canonicalize(UiMountedCanonicalBoxInput {
                        x: bounds.x() - parent_bounds.x(),
                        y: bounds.y() - parent_bounds.y(),
                        width: bounds.width(),
                        height: bounds.height(),
                        coordinate_space: UiMountedCoordinateSpace::GraphNodeLocal,
                    })
                    .expect("fixture child coordinates are finite local geometry");
                    crate::mounting::UiMountedOccurrenceGeometry::parent_relative(
                        *instance, parent, local,
                    )
                }
                None => crate::mounting::UiMountedOccurrenceGeometry::surface(*instance, bounds),
            };
            (occurrence, (*instance, bounds))
        })
        .collect::<Vec<_>>();
    let surface_declaration = session
        .authored_overlay_bindings
        .bound_owners()
        .find_map(|(declaration, runtime, _)| (runtime == surface).then_some(declaration));
    let mut regions = Vec::new();
    if let Some(declaration) = surface_declaration {
        for instance in &ordered {
            let basis = session
                .mounted
                .current_mounted_identity_basis(*instance)
                .expect("fixture occurrence remains mounted");
            let bounds = expected[instance];
            for binding in session
                .application
                .mounted_region_declarations(declaration, basis.graph_node_identity())
                .0
            {
                regions.push(crate::mounting::UiMountedMosaicRegionGeometry::new(
                    *instance,
                    binding.declaration(),
                    binding.executed_region(),
                    UiMountedCanonicalBox::canonicalize(UiMountedCanonicalBoxInput {
                        x: 0.0,
                        y: 0.0,
                        width: bounds.width(),
                        height: bounds.height(),
                        coordinate_space: UiMountedCoordinateSpace::GraphNodeLocal,
                    })
                    .expect("fixture region is valid owner-local geometry"),
                ));
            }
        }
    }
    let mut layout = session.begin_mounted_layout();
    let basis = layout.basis(surface).unwrap();
    layout
        .complete_surface_geometry(
            crate::mounting::UiMountedSurfaceGeometryBatch::new(
                basis,
                crate::mounting::UiMountedLayoutRevision::new(layout_revision)
                    .expect("fixture revisions are nonzero"),
                viewport,
                rows.iter()
                    .map(|(occurrence, _)| *occurrence)
                    .collect::<Vec<_>>(),
            )
            .with_regions(regions),
        )
        .expect("completed occurrence geometry should match the mounted surface");
    rows.into_iter().map(|(_, expected)| expected).collect()
}

pub(crate) fn surface_viewport_bounds() -> UiMountedCanonicalBox {
    UiMountedCanonicalBox::canonicalize(UiMountedCanonicalBoxInput {
        x: 0.0,
        y: 0.0,
        width: 1_280.0,
        height: 720.0,
        coordinate_space: UiMountedCoordinateSpace::HostSurface,
    })
    .expect("fixture viewport is finite surface geometry")
}

fn belongs_to_subtree(
    mut instance: UiMountedInstanceIdentity,
    root: UiMountedInstanceIdentity,
    hierarchy: &[(UiMountedInstanceIdentity, Option<UiMountedInstanceIdentity>)],
) -> bool {
    loop {
        if instance == root {
            return true;
        }
        let Some(parent) = hierarchy
            .iter()
            .find_map(|(candidate, parent)| (*candidate == instance).then_some(*parent))
            .flatten()
        else {
            return false;
        };
        instance = parent;
    }
}

pub(crate) fn expected_nonoverlapping_bounds(index: usize) -> UiMountedCanonicalBox {
    let column = index % 32;
    let row = index / 32;
    UiMountedCanonicalBox::canonicalize(UiMountedCanonicalBoxInput {
        x: 8.0 + column as f32 * 36.0,
        y: 12.0 + row as f32 * 28.0,
        width: 28.0 + (index % 3) as f32,
        height: 20.0 + (index % 2) as f32,
        coordinate_space: UiMountedCoordinateSpace::HostSurface,
    })
    .expect("fixture coordinates are finite nonnegative geometry")
}

pub(crate) fn assert_prepared_surface_geometry(
    _session: &super::WorthUiActiveApplicationSession,
    frame: &crate::mounting::UiPreparedMountedFrame,
    _surface: UiSemanticSurfaceIdentity,
    expected: &[(UiMountedInstanceIdentity, UiMountedCanonicalBox)],
) {
    let projection = frame.projection_rc_for_test();
    for (instance, expected_bounds) in expected {
        let allocation = projection
            .appearance_allocation_for_test(*instance)
            .expect("every completed layout occurrence is projected");
        let worth_ui_host_contract::UiMountedAllocationProjection::Known { bounds, .. } =
            allocation
        else {
            panic!("completed occurrence geometry must project as known allocation")
        };
        assert_eq!(bounds, *expected_bounds);
    }
}
