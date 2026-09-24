#[path = "scroll_runtime/geometry.rs"]
mod geometry;
#[path = "scroll_runtime/scenario.rs"]
mod scenario;
#[path = "scroll_runtime/shared_owner_replacement.rs"]
mod shared_owner_replacement;

use scenario::{
    admit_scroll, publish_predecessor, publish_scrolled_frame, publish_with_hit_coordinate,
    sibling_scroll_visual_source, with_scroll_mosaic,
};
use worth_ui::facade::observation_report::UiHostObservationPresentationBasis;
use worth_ui::facade::source::WorthUiFilesystemSourceProvider;
use worth_ui_certification::scenario::filesystem_application_lifecycle::FilesystemApplicationLifecycleScenario;
use worth_ui_host_contract::{
    UiHostScrollDeltaPhase, UiHostScrollDeltaPrecision, UiHostScrollDeltaSource,
    UiHostScrollDeltaTargetAffinity,
};
use worth_ui_host_headless::{UiHeadlessRecorderCapacity, WorthUiHeadlessRecorder};
use worth_ui_test_support::{
    UiScrollObservationCertificationDenial, UiScrollObservationCertificationOutcome,
    WorthUiActiveSessionCertificationExt, WorthUiMountedIdentityCertificationExt,
    WorthUiServiceStateCertificationExt,
};

use crate::filesystem_contract_workspace::FilesystemContractWorkspace;
use crate::filesystem_mounted_world::{component_graph_nodes, establish_allocation};
use crate::mounted_application_lifecycle::known_empty_surface_world::profile;

#[test]
fn exact_coordinate_scroll_retains_sign_cancellation_and_ambiguous_denial() {
    let scenario = FilesystemApplicationLifecycleScenario::new("phase-315-scroll-ingress");
    let recorder = WorthUiHeadlessRecorder::with_viewport_extent(
        UiHeadlessRecorderCapacity::production_default(),
        worth_ui::facade::measurement_exchange::UiViewportExtentObservation {
            width: 160.0,
            height: 96.0,
        },
    );
    let capabilities =
        with_scroll_mosaic(scenario.visual_identity_application_builder(recorder.clone()))
            .freeze()
            .expect("scroll visual capabilities freeze");
    let workspace = FilesystemContractWorkspace::new("phase-315-scroll-ingress");
    workspace.write("app/main.wui", &sibling_scroll_visual_source());
    let snapshot = WorthUiFilesystemSourceProvider::new(workspace.root())
        .read()
        .expect("production source provider reads the scroll visual source");
    let submission = FilesystemApplicationLifecycleScenario::lower_snapshot(
        snapshot,
        capabilities.capabilities(),
    );
    let application =
        with_scroll_mosaic(scenario.visual_identity_application_builder(recorder.clone()))
            .with_candidate_submission(submission)
            .freeze()
            .expect("source-authored scroll visual application freezes");
    workspace.close();
    let component_nodes = component_graph_nodes(&application);
    let scroll_target_node = component_nodes[1];
    let mut session = application
        .launch()
        .expect("mosaic-authored application launches");
    let surface = session
        .create_declared_semantic_surface("visual.identity.surface.main")
        .unwrap();
    let binding = session
        .register_host_surface(
            surface,
            worth_ui_runtime::facade::mounted::UiHostSurfacePresentationMode::RecordOnly,
            profile(1),
        )
        .unwrap()
        .binding_generation();
    let mut scroll_target = None;
    for graph_node in component_nodes {
        let handle = session.mounted_graph_node(graph_node).unwrap();
        let mounted = session.mount_instance(handle, surface).unwrap();
        if graph_node == scroll_target_node {
            scroll_target = Some(mounted);
        }
    }
    establish_allocation(&mut session, 3);
    assert_eq!(
        session.declared_region_layout_inputs(surface).len(),
        4,
        "both nested region declarations on both mounted owners must reach layout"
    );
    publish_predecessor(&mut session);
    let (mut current, coordinate) = publish_with_hit_coordinate(
        &mut session,
        binding,
        scroll_target.expect("hit-only component is mounted"),
    );
    let mut presentation = UiHostObservationPresentationBasis::new(
        current.host_surface,
        current.frame,
        binding,
        current.epoch,
    );
    let ownership_before_deltas = session.inspect_scroll_runtime_for_certification();
    assert!(ownership_before_deltas.ownership_resolutions() > 0);
    assert!(ownership_before_deltas.ownership_graph_nodes_visited() > 0);
    assert!(ownership_before_deltas.ownership_plan_nodes_visited() > 0);
    let updated = admit_scroll(
        &mut session,
        binding,
        &current,
        1,
        UiHostScrollDeltaPhase::Updated,
        UiHostScrollDeltaTargetAffinity::exact_coordinate(presentation, coordinate),
        125,
        -250,
    );
    assert_eq!(
        updated,
        UiScrollObservationCertificationOutcome::Applied {
            source: UiHostScrollDeltaSource::PointerWheel,
            phase: UiHostScrollDeltaPhase::Updated,
            precision: UiHostScrollDeltaPrecision::Pixel,
            requested_inline_subpixels: -125,
            requested_block_subpixels: 250,
            owners_visited: 1,
        }
    );

    assert_eq!(
        target_offset(&session, scroll_target_node),
        0,
        "admitted wheel input must not move the committed offset before its frame is accepted"
    );
    let routed = session.inspect_scroll_runtime_for_certification();
    assert_eq!(
        routed.ownership_resolutions(),
        ownership_before_deltas.ownership_resolutions()
    );
    assert_eq!(
        routed.ownership_graph_nodes_visited(),
        ownership_before_deltas.ownership_graph_nodes_visited()
    );
    assert_eq!(
        routed.ownership_plan_nodes_visited(),
        ownership_before_deltas.ownership_plan_nodes_visited()
    );
    (current, presentation) = republish(&mut session, binding, &current);
    assert_eq!(
        target_offset(&session, scroll_target_node),
        250,
        "publication must commit the same regional offset used to prepare the accepted frame"
    );

    // Completing a new mounted layout legitimately refreshes ownership. The
    // following ingress interval must again do no ownership discovery.
    let ownership_before_deltas = session.inspect_scroll_runtime_for_certification();
    let cancelled = admit_scroll(
        &mut session,
        binding,
        &current,
        2,
        UiHostScrollDeltaPhase::Cancelled,
        UiHostScrollDeltaTargetAffinity::exact_coordinate(presentation, coordinate),
        500,
        -750,
    );
    assert_eq!(
        cancelled,
        UiScrollObservationCertificationOutcome::Applied {
            source: UiHostScrollDeltaSource::PointerWheel,
            phase: UiHostScrollDeltaPhase::Cancelled,
            precision: UiHostScrollDeltaPrecision::Pixel,
            requested_inline_subpixels: 0,
            requested_block_subpixels: 0,
            owners_visited: 1,
        }
    );

    let saturated = admit_scroll(
        &mut session,
        binding,
        &current,
        3,
        UiHostScrollDeltaPhase::Updated,
        UiHostScrollDeltaTargetAffinity::exact_coordinate(presentation, coordinate),
        0,
        -1_000_000_000,
    );
    assert_eq!(
        saturated,
        UiScrollObservationCertificationOutcome::Applied {
            source: UiHostScrollDeltaSource::PointerWheel,
            phase: UiHostScrollDeltaPhase::Updated,
            precision: UiHostScrollDeltaPrecision::Pixel,
            requested_inline_subpixels: 0,
            requested_block_subpixels: 1_000_000_000,
            owners_visited: 1,
        }
    );
    (current, presentation) = republish(&mut session, binding, &current);
    let saturated_geometry = session.inspect_scroll_runtime_for_certification();
    assert_eq!(saturated_geometry.owner_geometry().len(), 2);
    assert_eq!(
        saturated_geometry
            .owner_geometry()
            .iter()
            .filter(|owner| owner.graph_node_digest() == Some(scroll_target_node.digest()))
            .count(),
        1,
        "the target resolves to its nearest declared owner"
    );
    let saturated_target = saturated_geometry
        .owner_geometry()
        .iter()
        .find(|owner| owner.graph_node_digest() == Some(scroll_target_node.digest()))
        .expect("exact-coordinate target has one nearest declared owner");
    assert!(saturated_target.plan_region_index().is_some());
    assert!(saturated_target.max_block_subpixels() > 0);
    assert_eq!(
        saturated_target.block_offset_subpixels(),
        saturated_target.max_block_subpixels()
    );

    let returned = admit_scroll(
        &mut session,
        binding,
        &current,
        4,
        UiHostScrollDeltaPhase::Updated,
        UiHostScrollDeltaTargetAffinity::exact_coordinate(presentation, coordinate),
        0,
        1_000_000_000,
    );
    assert_eq!(
        returned,
        UiScrollObservationCertificationOutcome::Applied {
            source: UiHostScrollDeltaSource::PointerWheel,
            phase: UiHostScrollDeltaPhase::Updated,
            precision: UiHostScrollDeltaPrecision::Pixel,
            requested_inline_subpixels: 0,
            requested_block_subpixels: -1_000_000_000,
            owners_visited: 1,
        }
    );
    (current, presentation) = republish(&mut session, binding, &current);
    let returned_geometry = session.inspect_scroll_runtime_for_certification();
    assert!(returned_geometry
        .owner_geometry()
        .iter()
        .filter(|owner| owner.graph_node_digest() == Some(scroll_target_node.digest()))
        .all(|owner| owner.block_offset_subpixels() == 0));
    let mut distinct_owner_nodes = returned_geometry
        .owner_geometry()
        .iter()
        .map(|owner| {
            owner
                .graph_node_digest()
                .expect("each sibling region owner retains its graph identity")
        })
        .collect::<Vec<_>>();
    distinct_owner_nodes.sort_unstable();
    distinct_owner_nodes.dedup();
    assert_eq!(
        distinct_owner_nodes.len(),
        2,
        "surface fallback is ambiguous between two distinct sibling owners"
    );

    let fallback = admit_scroll(
        &mut session,
        binding,
        &current,
        5,
        UiHostScrollDeltaPhase::Updated,
        UiHostScrollDeltaTargetAffinity::presented_surface_fallback(presentation),
        1,
        1,
    );
    assert_eq!(
        fallback,
        UiScrollObservationCertificationOutcome::Denied(
            UiScrollObservationCertificationDenial::PresentedSurfaceFallbackIsAmbiguous,
        )
    );
    let ownership_after_deltas = session.inspect_scroll_runtime_for_certification();
    assert_eq!(
        ownership_after_deltas.ownership_resolutions(),
        ownership_before_deltas.ownership_resolutions(),
        "host delta routing must use the mounted ownership index"
    );
    assert_eq!(
        ownership_after_deltas.ownership_graph_nodes_visited(),
        ownership_before_deltas.ownership_graph_nodes_visited(),
        "host delta routing must not rediscover ownership through the graph"
    );
    assert_eq!(
        ownership_after_deltas.ownership_plan_nodes_visited(),
        ownership_before_deltas.ownership_plan_nodes_visited(),
        "host delta routing must not rediscover ownership through mosaic plan ranges"
    );

    for sequence in 6..=101 {
        let wheel = if sequence % 2 == 0 { -1_000 } else { 1_000 };
        assert!(
            matches!(
                admit_scroll(
                    &mut session,
                    binding,
                    &current,
                    sequence,
                    UiHostScrollDeltaPhase::Updated,
                    UiHostScrollDeltaTargetAffinity::exact_coordinate(presentation, coordinate),
                    0,
                    wheel,
                ),
                UiScrollObservationCertificationOutcome::Applied {
                    owners_visited: 1,
                    ..
                }
            ),
            "lossless wheel input must continue after consumed reports exceed retention capacity"
        );
    }
    let _ = republish(&mut session, binding, &current);
    assert_eq!(target_offset(&session, scroll_target_node), 0);

    let _ = session.shutdown();
    drop(capabilities);
}

/// Presents the pending frame, which is what commits a direct wheel offset.
fn republish(
    session: &mut worth_ui::facade::app::WorthUiActiveApplicationSession,
    binding: worth_ui_runtime::facade::mounted::UiSurfaceBindingGeneration,
    current: &crate::mounted_application_lifecycle::published_mounted_world::PresentedObservationBasis,
) -> (
    crate::mounted_application_lifecycle::published_mounted_world::PresentedObservationBasis,
    UiHostObservationPresentationBasis,
) {
    let next = publish_scrolled_frame(session, binding, current.instance);
    let presentation =
        UiHostObservationPresentationBasis::new(next.host_surface, next.frame, binding, next.epoch);
    (next, presentation)
}

fn target_offset(
    session: &worth_ui::facade::app::WorthUiActiveApplicationSession,
    node: worth_ui::facade::graph::UiGraphNodeIdentity,
) -> i64 {
    session
        .inspect_scroll_runtime_for_certification()
        .owner_geometry()
        .iter()
        .find(|owner| owner.graph_node_digest() == Some(node.digest()))
        .unwrap()
        .block_offset_subpixels()
}
