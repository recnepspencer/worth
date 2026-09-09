use super::*;
use crate::mounting::spatial_index::UiMountedSpatialBudget;

#[test]
fn completed_mechanics_derive_local_hit_candidates_and_preserve_abandoned_predecessors() {
    let (fonts, _) = worth_ui_text::UiGlobalFontCollection::admit_qualified_profile().unwrap();
    let fonts = Arc::new(fonts);
    let surface = UiSemanticSurfaceIdentity::mint_unbound().unwrap();
    let neighbor = UiSemanticSurfaceIdentity::mint_unbound().unwrap();
    let binding = UiSurfaceBindingGeneration::mint_unbound().unwrap();
    let neighbor_binding = UiSurfaceBindingGeneration::mint_unbound().unwrap();
    let instances = (0..65)
        .map(|_| UiMountedInstanceIdentity::mint_unbound().unwrap())
        .collect::<Vec<_>>();
    let target = instances[0];
    let nodes = instances
        .iter()
        .enumerate()
        .map(|(i, instance)| {
            node(
                *instance,
                if i == 64 { neighbor } else { surface },
                (i % 64) as u32,
            )
        })
        .collect();
    let semantic = projection(nodes, [(surface, binding), (neighbor, neighbor_binding)]);
    let mut source = UiMountedMechanicSource::default();
    let initial = apply(&mut source, &fonts, &semantic, &instances);
    assert_eq!(initial.hit_tests, 65);
    assert!(initial.hit_index_work.node_copies >= 65);
    assert!(initial.hit_index_work.map_key_probes > 0);
    let selected = query(&source, binding, [5.0, 5.0]);
    assert_eq!(selected.instances, [target]);
    assert!(
        selected.work.node_visits < 32,
        "source query must prune unrelated rows"
    );
    assert_eq!(
        query(&source, neighbor_binding, [5.0, 5.0]).instances,
        [instances[64]]
    );
    let predecessor = source.clone();
    let retained_view = source.visual_region_basis();
    assert_eq!(retained_view.hit_test().len(), 65);
    let unchanged = apply(&mut source, &fonts, &semantic, &[target]);
    assert_eq!(
        (
            unchanged.hit_index_work.node_visits,
            unchanged.hit_index_work.node_copies
        ),
        (0, 0)
    );

    let empty = projection(
        Vec::new(),
        [(surface, binding), (neighbor, neighbor_binding)],
    );
    let removed = apply(&mut source, &fonts, &empty, &[target]);
    assert!(removed.hit_index_work.node_copies > 0);
    assert!(query(&source, binding, [5.0, 5.0]).instances.is_empty());
    assert_eq!(
        query(&source, binding, [25.0, 5.0]).instances,
        [instances[1]]
    );
    assert_eq!(
        query(&source, neighbor_binding, [5.0, 5.0]).instances,
        [instances[64]]
    );
    assert_eq!(query(&predecessor, binding, [5.0, 5.0]).instances, [target]);
    assert_eq!(retained_view.hit_test().len(), 65);
    assert_eq!(source.visual_region_basis().hit_test().len(), 64);
    drop(source); // The uninstalled candidate cannot retire its retained predecessor.
    assert_eq!(query(&predecessor, binding, [5.0, 5.0]).instances, [target]);
}

fn query(
    source: &UiMountedMechanicSource,
    binding: UiSurfaceBindingGeneration,
    point: [f64; 2],
) -> crate::mounting::spatial_index::UiMountedSpatialQuery {
    source
        .allocation_hit_candidates(
            binding,
            UiMountedCoordinateSpace::HostSurface,
            point,
            UiMountedSpatialBudget {
                node_visits: 1024,
                candidates: 128,
            },
        )
        .unwrap()
}

#[test]
fn full_hit_capacity_admits_add_before_remove_and_denies_final_overflow() {
    let (fonts, _) = worth_ui_text::UiGlobalFontCollection::admit_qualified_profile().unwrap();
    let fonts = Arc::new(fonts);
    let surface = UiSemanticSurfaceIdentity::mint_unbound().unwrap();
    let binding = UiSurfaceBindingGeneration::mint_unbound().unwrap();
    let surfaces = [
        (surface, binding),
        (
            UiSemanticSurfaceIdentity::mint_unbound().unwrap(),
            UiSurfaceBindingGeneration::mint_unbound().unwrap(),
        ),
    ];
    let capacity = worth_ui_host_contract::UiMountedHitTestTable::MAX_ROWS;
    let instances = (0..capacity)
        .map(|_| UiMountedInstanceIdentity::mint_unbound().unwrap())
        .collect::<Vec<_>>();
    let semantic = projection(
        instances
            .iter()
            .enumerate()
            .map(|(rank, instance)| node(*instance, surface, rank as u32))
            .collect(),
        surfaces,
    );
    let mut source = UiMountedMechanicSource::default();
    let initial = apply(&mut source, &fonts, &semantic, &instances);
    assert_eq!(initial.hit_tests, capacity);
    let retained = source.clone();
    let added = UiMountedInstanceIdentity::mint_unbound().unwrap();
    let replacement = projection(vec![node(added, surface, capacity as u32)], surfaces);
    let mutation = apply(&mut source, &fonts, &replacement, &[added, instances[0]]);
    assert_eq!(mutation.hit_tests, 1);
    assert_eq!(mutation.hit_index_work.reconstructed_rows, 0);
    assert!(query(&source, binding, [5.0, 5.0]).instances.is_empty());
    assert_eq!(
        query(&retained, binding, [5.0, 5.0]).instances,
        [instances[0]]
    );
    let added_point = [(capacity * 20 + 5) as f64, 5.0];
    assert_eq!(query(&source, binding, added_point).instances, [added]);
    let mut overflowing = source.clone();
    let excess = UiMountedInstanceIdentity::mint_unbound().unwrap();
    let excess_semantic = projection(vec![node(excess, surface, capacity as u32 + 1)], surfaces);
    assert!(matches!(
        attempt(&mut overflowing, &fonts, &excess_semantic, &[excess]),
        Err(crate::mounting::UiMountedProjectionDenial::HitTestCapacityExceeded)
    ));
    drop(overflowing);
    assert_eq!(query(&source, binding, added_point).instances, [added]);
}

fn apply(
    source: &mut UiMountedMechanicSource,
    fonts: &Arc<worth_ui_text::UiGlobalFontCollection>,
    semantic: &UiMountedSemanticProjection,
    changed: &[UiMountedInstanceIdentity],
) -> super::super::mechanic_source::UiMountedMechanicMutation {
    attempt(source, fonts, semantic, changed).unwrap()
}

fn attempt(
    source: &mut UiMountedMechanicSource,
    fonts: &Arc<worth_ui_text::UiGlobalFontCollection>,
    semantic: &UiMountedSemanticProjection,
    changed: &[UiMountedInstanceIdentity],
) -> Result<
    super::super::mechanic_source::UiMountedMechanicMutation,
    crate::mounting::UiMountedProjectionDenial,
> {
    let frame = UiMountedFrameIdentity::mint_unbound().unwrap();
    let mut instances = crate::runtime::persistent_index::UiPersistentOrdSet::default();
    for instance in changed {
        instances.insert(*instance);
    }
    let receipts = crate::mounting::UiMountedNodeReceiptBasis::mint(frame, instances).unwrap();
    source.apply(completion(
        frame,
        UiMountedContentGeneration::mint_unbound().unwrap(),
        &receipts,
        semantic,
        fonts,
        changed,
        1,
    ))
}

fn projection(
    nodes: Vec<UiMountedProjectionNodeRecord>,
    surfaces: [(UiSemanticSurfaceIdentity, UiSurfaceBindingGeneration); 2],
) -> UiMountedSemanticProjection {
    UiMountedSemanticProjection::initial(
        nodes,
        surfaces
            .into_iter()
            .map(|(surface, binding)| UiMountedProjectionSurface {
                surface,
                binding,
                audience: UiMountedProjectionAudience::full(),
            })
            .collect(),
    )
}

fn node(
    instance: UiMountedInstanceIdentity,
    surface: UiSemanticSurfaceIdentity,
    rank: u32,
) -> UiMountedProjectionNodeRecord {
    let bounds = UiMountedCanonicalBox::canonicalize(UiMountedCanonicalBoxInput {
        x: (rank * 20) as f32,
        y: 0.0,
        width: 10.0,
        height: 10.0,
        coordinate_space: UiMountedCoordinateSpace::HostSurface,
    })
    .unwrap();
    UiMountedProjectionNodeRecord {
        receipt: UiMountedNodeReceipt::from_input(UiMountedNodeReceiptInput {
            mounted_instance: instance,
            graph_node: crate::graph::UiGraphNodeIdentity::new(90_000 + u64::from(rank)),
            semantic_surface: surface,
            incarnation: UiMountIncarnation::mint_unbound().unwrap(),
            plan_digest: 7,
            role: UiMountedMechanicalRole::Control,
            participation: admitted_participation(),
            allocation: UiMountedAllocationProjection::Known {
                bounds,
                basis: UiMountedAllocationBasis::new(
                    1,
                    2,
                    3,
                    UiMountedTransformProjection::Identity,
                ),
            },
        }),
        plan_index: Some(rank),
        occurrence_allocation: UiMountedAllocationProjection::Known {
            bounds,
            basis: UiMountedAllocationBasis::new(1, 2, 3, UiMountedTransformProjection::Identity),
        },
        appearance_geometry:
            crate::mounting::projection::frame_storage::UiMountedAppearanceGeometry::from_occurrence(
                UiMountedAllocationProjection::Known {
                    bounds,
                    basis: UiMountedAllocationBasis::new(
                        1,
                        2,
                        3,
                        UiMountedTransformProjection::Identity,
                    ),
                },
                crate::mounting::projection::appearance::UiMountedAppearanceClip::Unclipped,
            ),
        surface_paint_order: Some(0),
        has_appearance_attachment: false,
        appearance_clip:
            crate::mounting::projection::appearance::UiMountedAppearanceClip::Unclipped,
        static_paint: None,
        semantic_text: None,
        hit_test: Some(UiMountedHitTestSeed::for_test(rank)),
        focus_support: crate::capability::ComponentFocusSupport::not_focusable(),
        focus_scope: None,
        focus_container_owner: None,
        component_id: None,
        portal_child_owner: None,
    }
}
