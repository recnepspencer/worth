use super::{appearance, authoring, geometry};
use crate::projection_lifecycle::support::ScalarLifecycleWorld;
use std::collections::{BTreeMap, BTreeSet};
use worth_runtime_bridge::facade::BridgeMixedCauseOrderingInput;
use worth_signal::facade::NodeId;
use worth_ui::facade::app::WorthUiActiveApplicationSession;
use worth_ui::facade::appearance::UiThemeDefinitionIdentity;
use worth_ui::facade::observation::UiChangeClassificationOutcome;
use worth_ui::facade::rebind::{
    UiRebindExecutionPolicy, UiRebindExecutionRequest, UiRebindOutcome,
};
use worth_ui_host_contract::{UiMountedInstanceIdentity, UiSemanticSurfaceIdentity};
use worth_ui_host_headless::{
    UiHeadlessMountedFrameTranscript, UiHeadlessRecorderCapacity, WorthUiHeadlessRecorder,
};
use worth_ui_test_support::WorthUiMountedIdentityCertificationExt;

pub(super) fn verify() {
    assert_eq!(appearance::axis_count(), 6);
    assert_runtime_scale_axes();
    let recorder = WorthUiHeadlessRecorder::with_viewport_extent(
        UiHeadlessRecorderCapacity::new(4, 8, 16_384),
        worth_ui::facade::measurement_exchange::UiViewportExtentObservation {
            width: 1_280.0,
            height: 720.0,
        },
    );
    let (mut query, completion) = ScalarLifecycleWorld::standard(NodeId::new(31_610, 0), "Ready");
    let registration =
        crate::projection_presentation::scalar_query_only::scalar_registration(&query);
    let mut session = authoring::build(recorder.clone(), registration)
        .launch()
        .unwrap();
    let (surfaces, nodes) = mount(&mut session);
    super::super::world::establish_allocations(&mut session, super::NODE_COUNT);
    geometry::install(&mut session, &surfaces, &nodes);
    let pending = query.initial().into_fact_and_predecessor().0;
    let current = query.advance(
        BridgeMixedCauseOrderingInput::AsyncCompletion(completion),
        Some(pending),
    );
    let mut turn = session.begin_observation_turn().unwrap();
    turn.admit_projection_query(worth_ui_query_binding::UiProjectionObservation::Scalar(
        current.into_fact_and_predecessor().0.into_observation(),
    ))
    .unwrap();
    let observations = turn.seal().unwrap();
    let UiChangeClassificationOutcome::Changed(change) =
        session.classify_observations(observations).unwrap()
    else {
        panic!("AP10 initial Query text and appearance owners must change");
    };
    execute_change(&mut session, change, 1);
    let initial = recorder.drain_transcripts().into_vec();
    assert_initial(&initial, &nodes);
    let neighbor_bindings = surfaces[1..]
        .iter()
        .map(|surface| session.active_theme_binding(*surface).unwrap().clone())
        .collect::<Vec<_>>();
    let surface = surfaces[0];
    let generation = session
        .active_theme_binding(surface)
        .unwrap()
        .binding_generation();
    let capability = session
        .admit_appearance_theme(
            surface,
            &UiThemeDefinitionIdentity::new(appearance::CHANGED_THEME).unwrap(),
        )
        .unwrap();
    let request = session
        .prepare_programmatic_theme_switch(surface, generation, capability)
        .unwrap();
    let UiChangeClassificationOutcome::Changed(change) =
        session.prepare_theme_switch(request).unwrap()
    else {
        panic!("AP10 different theme must prepare a binding settlement");
    };
    let scope = session.resolve_affected_scope(change).unwrap();
    let scope_cost = scope.cost();
    assert_eq!(scope_cost.lookup_receipts(), 0);
    assert_eq!(scope_cost.index_probes(), 207);
    assert_eq!(scope_cost.graph_and_mounted_entries(), 360);
    let compared = scope_cost.theme_slots_compared();
    let plan = session
        .compile_rebind_plan(
            scope.resolve_identity_lifecycle().unwrap(),
            UiRebindExecutionPolicy::ordinary(),
        )
        .unwrap();
    let prepared = session
        .prepare_rebind(plan, UiRebindExecutionRequest::new(2))
        .unwrap();
    let prepared_cost = prepared.prepared_frame().unwrap().cost_report();
    assert_eq!(prepared_cost.appearance().selected_instance_count(), 60);
    assert_eq!(prepared_cost.appearance().materialized_context_count(), 60);
    assert_eq!(prepared_cost.appearance().membership_traversed_entries(), 0);
    assert_eq!(prepared_cost.surface_instance_pairs(), 0);
    assert_eq!(prepared_cost.named().minted(), 0);
    assert!(
        recorder.observed_transcripts().is_empty(),
        "preparation cannot publish host work"
    );
    let receipt = match prepared.execute(2) {
        UiRebindOutcome::Published(receipt) => receipt,
        UiRebindOutcome::RejectedBeforeEffects(denial) => panic!(
            "AP10 theme rejected: {:?}; {:?}",
            denial.cause(),
            denial.host_rejections()
        ),
        _ => panic!("AP10 theme must reach accepted host publication"),
    };
    let mounted_cost = receipt.realized_mount_cost().unwrap();
    assert_eq!(mounted_cost.appearance().selected_instance_count(), 60);
    let host = mounted_cost.adapter();
    assert_eq!(receipt.mounted_publication().unwrap().bindings().len(), 1);
    assert_eq!(host.retained_command_scans(), 0);
    assert_eq!(host.retained_command_clones(), 0);
    assert_eq!(host.translated_rows(), 240);
    assert_eq!(host.delta_rows_carried(), 120);
    assert_eq!(host.draw_list_mutations(), 60);
    assert_eq!(host.logical_damage_regions(), 60);
    assert_eq!(host.order_mutations(), 0);
    drop(receipt);
    let changed = recorder.drain_transcripts().into_vec();
    assert_theme_delta(&changed, &nodes);
    assert_text_layout_reuse(&initial, &changed);
    for (surface, predecessor) in surfaces[1..].iter().zip(neighbor_bindings) {
        assert_eq!(session.active_theme_binding(*surface), Some(&predecessor));
    }
    assert_eq!(
        session
            .active_theme_binding(surface)
            .unwrap()
            .binding_generation(),
        generation + 1
    );
    let shutdown = session.shutdown();
    assert!(shutdown.mounted_presentation().is_empty());
    assert!(shutdown.rebind().is_empty());
    assert_eq!(
        compared, 18,
        "AP10 compares the twelve changed/equal candidates with exact alias work, never all 512 slots"
    );
}

fn assert_runtime_scale_axes() {
    let evidence = worth_ui_test_support::runtime_service_scale_evidence();
    assert_eq!(evidence.active_motion_tracks(), 64);
    assert_eq!(evidence.motion_tracks_sampled(), 64);
    assert_eq!(evidence.retained_inactive_motion_tracks(), 64);
    assert_eq!(evidence.inactive_motion_tracks_sampled(), 0);
    assert_eq!(evidence.overlay_portal_rows(), 32);
    assert_eq!(evidence.overlay_portal_depth(), 32);
    assert_eq!(evidence.overlay_backdrop_rows(), 48);
    assert_eq!(evidence.overlay_backdrop_categories(), [16, 16, 16]);
    assert_eq!(evidence.overlay_initial_portal_rows_read(), 32);
    assert_eq!(evidence.overlay_initial_bindings_read(), 32);
    assert_eq!(evidence.overlay_initial_backdrops_selected(), 48);
    assert_eq!(evidence.overlay_initial_backdrops_changed(), 48);
    assert_eq!(evidence.overlay_initial_relation_edges(), 80);
    assert_eq!(evidence.overlay_successor_portal_rows_read(), 32);
    assert_eq!(evidence.overlay_successor_bindings_read(), 32);
    assert_eq!(evidence.overlay_successor_backdrops_selected(), 2);
    assert_eq!(evidence.overlay_successor_relation_edges(), 0);
    assert_eq!(evidence.overlay_successor_backdrops_changed(), 2);
    assert_eq!(evidence.overlay_successor_replayed(), 0);
    assert_eq!(evidence.overlay_successor_unrelated_neighborhoods(), 0);
    assert_eq!(evidence.overlay_released_rows(), 80);
    assert_eq!(evidence.unrelated_neighborhoods_touched(), 0);
    assert!(evidence.terminal_resources_zero());
}

fn assert_text_layout_reuse(
    initial: &[UiHeadlessMountedFrameTranscript],
    changed: &[UiHeadlessMountedFrameTranscript],
) {
    let layouts = |transcripts: &[UiHeadlessMountedFrameTranscript]| {
        transcripts
            .iter()
            .flat_map(|transcript| transcript.semantic_text())
            .map(|text| (text.command_identity(), text.layout_identity()))
            .collect::<std::collections::HashMap<_, _>>()
    };
    assert_eq!(layouts(initial).len(), 32);
    assert_eq!(
        layouts(changed),
        layouts(initial),
        "paint-only work preserves every qualified text layout"
    );
}

fn mount(
    session: &mut WorthUiActiveApplicationSession,
) -> ([UiSemanticSurfaceIdentity; 4], Vec<geometry::MountedNode>) {
    let surfaces = std::array::from_fn(|_| {
        let surface = session.create_semantic_surface().unwrap();
        session
            .register_host_surface(
                surface,
                worth_ui_host_contract::UiHostSurfacePresentationMode::RecordOnly,
                crate::mounted_application_lifecycle::known_empty_surface_world::profile(1),
            )
            .unwrap();
        surface
    });
    let graph = session.graph();
    let mut declarations = graph
        .node_identities()
        .filter_map(|identity| {
            let node = graph.lookup().graph_node(identity)?;
            let value = node.value();
            let name = value.declaration_identity().authored_semantic_name();
            let index = name
                .strip_prefix("component:appearance.scale.node_")?
                .parse::<usize>()
                .unwrap();
            Some((index, session.mounted_graph_node(identity).unwrap()))
        })
        .collect::<Vec<_>>();
    declarations.sort_by_key(|(index, _)| *index);
    assert_eq!(declarations.len(), super::NODE_COUNT);
    assert_eq!(
        declarations
            .iter()
            .map(|(_, node)| node.graph_node_identity())
            .collect::<BTreeSet<_>>()
            .len(),
        super::NODE_COUNT
    );
    declarations
        .iter()
        .enumerate()
        .for_each(|(expected, (actual, _))| assert_eq!(expected, *actual));
    let nodes = declarations
        .into_iter()
        .map(|(authored_index, node)| {
            let surface = surfaces[geometry::surface_index(authored_index)];
            geometry::MountedNode {
                authored_index,
                instance: session.mount_instance(node, surface).unwrap(),
                surface,
            }
        })
        .collect::<Vec<_>>();
    assert_eq!(
        nodes
            .iter()
            .map(|node| node.authored_index / 64)
            .collect::<BTreeSet<_>>()
            .len(),
        super::NEIGHBORHOOD_COUNT
    );
    (surfaces, nodes)
}

fn execute_change(
    session: &mut WorthUiActiveApplicationSession,
    change: worth_ui::facade::observation::UiClassifiedChange,
    tick: u64,
) {
    let lifecycle = session
        .resolve_affected_scope(change)
        .unwrap()
        .resolve_identity_lifecycle()
        .unwrap();
    let plan = session
        .compile_rebind_plan(lifecycle, UiRebindExecutionPolicy::ordinary())
        .unwrap();
    let prepared = session
        .prepare_rebind(plan, UiRebindExecutionRequest::new(tick))
        .unwrap();
    match prepared.execute(tick) {
        UiRebindOutcome::Published(_) => {}
        UiRebindOutcome::RejectedBeforeEffects(denial) => panic!(
            "AP10 baseline rejected: {:?}; {:?}",
            denial.cause(),
            denial.host_rejections()
        ),
        _ => panic!("AP10 baseline must publish"),
    }
}

fn assert_initial(
    transcripts: &[UiHeadlessMountedFrameTranscript],
    nodes: &[geometry::MountedNode],
) {
    assert_eq!(transcripts.len(), super::SURFACE_COUNT);
    assert_eq!(
        transcripts
            .iter()
            .map(|transcript| transcript.nodes().len())
            .sum::<usize>(),
        super::NODE_COUNT
    );
    assert_eq!(
        transcripts
            .iter()
            .map(|transcript| transcript.semantic_text().len())
            .sum::<usize>(),
        32
    );
    let surfaces = transcripts
        .iter()
        .flat_map(super::super::world::ordered_surfaces)
        .collect::<Vec<_>>();
    assert_eq!(surfaces.len(), super::STYLED_COUNT);
    assert_eq!(
        surfaces
            .iter()
            .map(|surface| surface.node_receipt().mounted_instance())
            .collect::<BTreeSet<_>>()
            .len(),
        super::STYLED_COUNT
    );
    let mounted = nodes
        .iter()
        .map(|node| (node.instance, node.authored_index))
        .collect::<BTreeMap<_, _>>();
    for surface in surfaces {
        let index = mounted[&surface.node_receipt().mounted_instance()];
        assert_eq!(
            surface.paint(),
            &worth_ui_host_contract::UiMountedSurfacePaint::Fill(
                worth_ui_host_contract::UiMountedAppearanceColor::from_straight_srgba(
                    appearance::rgba(authoring::component_role_index(index), false)
                )
                .into()
            )
        );
    }
    let texts = transcripts
        .iter()
        .flat_map(|transcript| transcript.semantic_text())
        .collect::<Vec<_>>();
    assert_eq!(
        texts.iter().filter(|text| text.text() == "Ready").count(),
        authoring::TEXT_NODE_COUNT
    );
    assert_eq!(
        texts.iter().filter(|text| text.text() == "CURRENT").count(),
        authoring::TEXT_NODE_COUNT
    );
}

fn assert_theme_delta(
    transcripts: &[UiHeadlessMountedFrameTranscript],
    nodes: &[geometry::MountedNode],
) {
    assert_eq!(transcripts.len(), 1, "unrelated surfaces are not presented");
    let surfaces = super::super::world::ordered_surfaces(&transcripts[0]);
    let expected = nodes
        .iter()
        .filter(|node| node.authored_index < 60)
        .map(|node| node.instance)
        .collect::<BTreeSet<UiMountedInstanceIdentity>>();
    let actual = surfaces
        .iter()
        .map(|surface| surface.node_receipt().mounted_instance())
        .collect::<BTreeSet<_>>();
    assert_eq!(
        actual, expected,
        "only the five changed used roles' twelve consumers appear in host work"
    );
    let mounted = nodes
        .iter()
        .map(|node| (node.instance, node.authored_index))
        .collect::<BTreeMap<_, _>>();
    for surface in surfaces {
        let index = mounted[&surface.node_receipt().mounted_instance()];
        assert_eq!(
            surface.paint(),
            &worth_ui_host_contract::UiMountedSurfacePaint::Fill(
                worth_ui_host_contract::UiMountedAppearanceColor::from_straight_srgba(
                    appearance::rgba(authoring::component_role_index(index), true)
                )
                .into()
            )
        );
        let [x, y, width, height] = geometry::bounds(index);
        let bounds = surface.bounds();
        assert_eq!(
            [
                i64::from(bounds.x()),
                i64::from(bounds.y()),
                i64::from(bounds.width()),
                i64::from(bounds.height())
            ],
            [
                (x * 1_000.0) as i64,
                (y * 1_000.0) as i64,
                (width * 1_000.0) as i64,
                (height * 1_000.0) as i64
            ]
        );
    }
}
