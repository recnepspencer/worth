use std::rc::Rc;

use crate::runtime::tests::appearance_component_session_test_support as support;

#[path = "appearance_projection_locality_test_support.rs"]
mod fixture;
#[path = "appearance_projection_retirement_tests.rs"]
mod retirement_tests;
use fixture::{
    admit_owner_snapshot, admit_theme, background_role, graph_node_for, locality_fixture,
    theme_bundle,
};

const CANDIDATE_TOKEN: &str = "theme.appearance_locality_candidate";
const UNSTYLED_COMPONENT: &str = "workspace.component.appearance_locality_unstyled";

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct LocalityMetrics {
    selected: usize,
    materialized: usize,
    canonical_consumers: usize,
    index_entries: usize,
    lifecycle_retired: usize,
    key_probes: usize,
    copied_nodes: usize,
    traversed: usize,
    motion_commands: u64,
    host_completed_without_effects: bool,
}

#[test]
fn appearance_preparation_locality_slope_and_projection_owner_identity() {
    let small = run_case(32);
    let large = run_case(512);
    for (retained_count, case_data) in [(32, &small), (512, &large)] {
        for metrics in &case_data.repeated {
            assert_eq!(metrics.selected, 1);
            assert_eq!(metrics.materialized, 1);
            assert_eq!(metrics.canonical_consumers, 1);
            assert_eq!(metrics.traversed, 0);
            assert!(metrics.host_completed_without_effects);
            let bound = avl_work_bound(retained_count);
            assert!(metrics.key_probes <= bound);
            assert!(metrics.copied_nodes <= bound);
        }
    }
    assert_eq!(small.initial.selected, 32);
    assert_eq!(small.initial.materialized, 32);
    assert_eq!(large.initial.selected, 512);
    assert_eq!(large.initial.materialized, 512);
    assert_eq!(
        small.repeated[0].index_entries,
        large.repeated[0].index_entries
    );
    assert!(small
        .repeated
        .iter()
        .all(|metrics| { metrics.index_entries == small.repeated[0].index_entries }));
    assert!(large
        .repeated
        .iter()
        .all(|metrics| { metrics.index_entries == large.repeated[0].index_entries }));
    assert_eq!(
        small
            .repeated
            .iter()
            .map(|metrics| metrics.motion_commands)
            .collect::<Vec<_>>(),
        large
            .repeated
            .iter()
            .map(|metrics| metrics.motion_commands)
            .collect::<Vec<_>>()
    );
    assert!(large
        .repeated
        .iter()
        .all(|metrics| metrics.motion_commands <= 3));
    assert_eq!(small.retirement.selected, 1);
    assert_eq!(large.retirement.selected, 1);
    assert_eq!(small.retirement.lifecycle_retired, 1);
    assert_eq!(large.retirement.lifecycle_retired, 1);
    assert_eq!(small.retirement.traversed, 0);
    assert_eq!(large.retirement.traversed, 0);
    assert_eq!(small.surviving, 30);
    assert_eq!(large.surviving, 510);
    assert!(small.baseline_static_paint);
    assert!(large.baseline_static_paint);
    assert!(small.appearance_only_host_static_only);
    assert!(large.appearance_only_host_static_only);
}

fn avl_work_bound(retained_count: usize) -> usize {
    let mut levels = 0;
    let mut width = 1;
    while width < retained_count {
        width *= 2;
        levels += 1;
    }
    64 + levels * 16
}

struct LocalityCase {
    initial: LocalityMetrics,
    repeated: Vec<LocalityMetrics>,
    retirement: LocalityMetrics,
    surviving: usize,
    baseline_static_paint: bool,
    appearance_only_host_static_only: bool,
}

fn run_case(retained_count: usize) -> LocalityCase {
    let role_a = background_role("test.appearance-locality-a", support::APPEARANCE_TOKEN);
    let role_b = background_role("test.appearance-locality-b", CANDIDATE_TOKEN);
    let token_a = crate::capability::ThemeTokenId::new(support::APPEARANCE_TOKEN).unwrap();
    let token_b = crate::capability::ThemeTokenId::new(CANDIDATE_TOKEN).unwrap();
    let static_token =
        crate::capability::ThemeTokenId::new(support::LEGACY_STATIC_PAINT_TOKEN).unwrap();
    let host = crate::certification_support::ScriptedPresentationHost::native_display();
    host.set_capabilities(worth_ui_host_native::appearance_capability_report());
    let host_observer = host.clone();
    let builder = support::legacy_static_paint_appearance_component_builder(&role_a)
        .register_component(support::static_paint_component(
            UNSTYLED_COMPONENT,
            static_token,
        ))
        .register_appearance_role(role_b.clone())
        .unwrap()
        .register_theme_token(support::appearance_theme_token(token_b.clone()))
        .register_appearance_theme_bundle(theme_bundle())
        .unwrap()
        .with_rust_authored_declaration_fixture(locality_fixture(&role_a, &role_b));
    let mut session = builder
        .freeze()
        .map(|application| {
            crate::facade::entry::WorthUiCertificationApplicationTransition::activate_test_host(
                application,
                host,
            )
        })
        .expect("locality fixture should prepare")
        .launch()
        .expect("locality fixture should launch");
    let surface = session.create_semantic_surface().unwrap();
    session
        .register_host_surface(
            surface,
            crate::facade::mounted::UiHostSurfacePresentationMode::NativeDisplay,
            crate::facade::mounted::UiSurfaceBindingProfile::new(
                1_000,
                crate::facade::mounted::UiSurfaceBindingCoordinatePosture::LogicalPoints,
                1,
            )
            .unwrap(),
        )
        .unwrap();
    let _active_node = graph_node_for(&session, support::APPEARANCE_NODE_A);
    let candidate_node = graph_node_for(&session, support::APPEARANCE_NODE_B);
    let _unstyled_node = graph_node_for(&session, UNSTYLED_COMPONENT);
    let graph_nodes = session
        .graph()
        .node_identities()
        .filter_map(|identity| {
            let lookup = session.graph().lookup().graph_node(identity)?;
            let name = lookup
                .value()
                .declaration_identity()
                .authored_semantic_name()
                .to_owned();
            (name != "worth_ui.runtime.bootstrap.product_root")
                .then_some((identity, Box::<str>::from(name)))
        })
        .collect::<Vec<_>>();
    let mut candidate_instances = Vec::with_capacity(retained_count.saturating_sub(1));
    for (graph_node, name) in graph_nodes {
        session
            .register_application_semantic_text(name, graph_node)
            .unwrap();
        let mounted_node = session.mounted_graph_node(graph_node).unwrap();
        let instance = session.mount_instance(mounted_node, surface).unwrap();
        if graph_node == candidate_node {
            candidate_instances.push(instance);
        }
    }
    let candidate_handle = session.mounted_graph_node(candidate_node).unwrap();
    for _ in 1..retained_count.saturating_sub(1) {
        candidate_instances.push(session.mount_instance(candidate_handle, surface).unwrap());
    }
    let capability = session.host_measurement_capability();
    let assumptions = crate::host::UiHostMeasurementAssumptionProfile::from_capability_report(
        capability.capability_report(),
        1,
        2,
        3,
        4,
    );
    session
        .establish_mounted_allocation_catalog(
            1,
            [
                crate::facade::entry::UiMountedAllocationMeasurementRequest::new(
                    worth_ui_host_contract::UiMeasurementEvidenceFamily::ViewportExtent,
                    crate::host::UiHostMeasurementNeed::ViewportExtent(
                        worth_ui_host_contract::UiViewportExtentRequest,
                    ),
                    crate::host::UiHostMeasurementNormalizationContext::viewport_logical_exact(
                        assumptions,
                    ),
                ),
            ],
        )
        .expect("locality fixture allocation should commit");
    crate::facade::entry::mounted_occurrence_geometry_test_support::install_nonoverlapping_surface_geometry(
        &mut session,
        surface,
        1,
        &[],
    );
    admit_owner_snapshot(
        &mut session,
        &role_a,
        &role_b,
        "appearance-locality-initial",
    );
    session.advance_mounted_identity_frame().unwrap();

    admit_theme(&mut session, token_a.clone(), 0, "#405060");
    admit_theme(&mut session, token_b.clone(), 0, "#607080");
    let initial = prepare_and_publish(&mut session, &host_observer, 1, true);
    assert_eq!(initial.selected, retained_count);
    assert_eq!(initial.materialized, retained_count);
    let baseline_static_paint = {
        let colors = host_observer.last_filled_rect_colors();
        !colors.is_empty()
            && colors
                .iter()
                .all(|color| color.channels() == [17, 34, 51, 255])
    };
    assert!(baseline_static_paint);

    let mut repeated = Vec::new();
    for (offset, color) in ["#506070", "#708090", "#8090a0"].into_iter().enumerate() {
        let revision = u64::try_from(offset + 1).unwrap();
        admit_theme(&mut session, token_a.clone(), revision, color);
        let metrics = prepare_and_publish(
            &mut session,
            &host_observer,
            u64::try_from(offset + 2).unwrap(),
            false,
        );
        assert_eq!(metrics.selected, 1);
        assert_eq!(metrics.materialized, 1);
        repeated.push(metrics);
    }
    let appearance_only_host_static_only = host_observer
        .last_filled_rect_colors()
        .iter()
        .all(|color| color.channels() == [17, 34, 51, 255]);
    assert!(appearance_only_host_static_only);
    admit_theme(&mut session, token_b.clone(), 1, "#90a0b0");
    let surviving = prepare_and_publish(&mut session, &host_observer, 5, false);
    assert_eq!(surviving.selected, retained_count - 1);
    assert_eq!(surviving.materialized, retained_count - 1);
    assert_eq!(surviving.canonical_consumers, 1);
    assert!(surviving.host_completed_without_effects);

    let retired_predecessor = retirement_tests::capture(&session, candidate_instances[0]);
    session.unmount_instance(candidate_instances[0]).unwrap();
    admit_theme(&mut session, token_a, 4, "#a0b0c0");
    let retirement = prepare_and_publish(&mut session, &host_observer, 6, false);
    assert_eq!(retirement.selected, 1);
    assert_eq!(retirement.materialized, 1);
    assert_eq!(retirement.lifecycle_retired, 1);
    assert_eq!(retirement.traversed, 0);
    assert!(retirement.host_completed_without_effects);
    retirement_tests::assert_removed(&session, &retired_predecessor);
    admit_theme(&mut session, token_b, 2, "#b0c0d0");
    let after_retirement = prepare_and_publish(&mut session, &host_observer, 7, false);
    assert_eq!(after_retirement.selected, retained_count - 2);
    assert_eq!(after_retirement.materialized, retained_count - 2);
    assert_eq!(after_retirement.canonical_consumers, 1);
    assert!(after_retirement.host_completed_without_effects);
    let _ = session.shutdown();
    LocalityCase {
        initial,
        repeated,
        retirement,
        surviving: after_retirement.selected,
        baseline_static_paint,
        appearance_only_host_static_only,
    }
}

fn prepare_and_publish(
    session: &mut crate::facade::WorthUiActiveApplicationSession,
    host: &crate::certification_support::ScriptedPresentationHost,
    now: u64,
    native_effects: bool,
) -> LocalityMetrics {
    for _ in session.inspect_mounted_identity().surface_bindings() {
        if native_effects {
            host.push_native_display_presented();
        } else {
            host.push_native_display_settled_without_effects();
        }
    }
    let frame = session
        .prepare_mounted_frame_with_application_presentation(
            crate::mounting::UiMountedFrameRequest::all_bound_surfaces(),
            |_| {},
        )
        .unwrap_or_else(|stop| match stop {
            crate::facade::entry::WorthUiMountedFrameExecutionStop::Preparation(denial) => {
                panic!("locality frame should prepare: {denial:?}")
            }
            _ => panic!("locality frame stopped before preparation"),
        });
    let candidate_projection = frame.projection_rc_for_test();
    // Tick 5 is the first multi-consumer successor with a published owner basis.
    if now == 2
        && frame
            .appearance_selection_cost_report()
            .selected_instance_count()
            == 1
    {
        frame.verify_mixed_appearance_reconstruction_denial_and_retry();
    }
    if now == 5 {
        frame.verify_unpublished_appearance_member_denial();
    }
    if now == 6 {
        retirement_tests::verify_retry(session, &frame);
    }
    let canonical_consumers = frame
        .appearance_invalidation_batch()
        .map_or(0, |batch| batch.graph_consumers().len());
    let surface_projection = frame
        .surfaces()
        .first()
        .expect("locality frame has bound surfaces")
        .projection_owner();
    assert!(Rc::ptr_eq(&candidate_projection, &surface_projection));
    let report = frame.appearance_selection_cost_report();
    let outcome = session.present_prepared_mounted_frame_internal(
        frame,
        worth_ui_host_contract::UiPresentationDeadline::at_tick(100),
        now,
    );
    let host_completed_without_effects =
        !native_effects && presentation_completed_without_effects(&outcome);
    let motion_commands = match &outcome {
        crate::mounting::UiMountedFrameOutcome::Published(receipt)
        | crate::mounting::UiMountedFrameOutcome::Reconciled(receipt) => {
            receipt.cost_report().appearance_motion_commands_visited()
        }
        _ => 0,
    };
    assert!(matches!(
        outcome,
        crate::mounting::UiMountedFrameOutcome::Published(_)
    ));
    let published_projection = session
        .current_mounted_projection_rc_for_test()
        .expect("published locality frame retains its owner");
    assert!(Rc::ptr_eq(&candidate_projection, &published_projection));
    LocalityMetrics {
        selected: report.selected_instance_count(),
        materialized: report.materialized_context_count(),
        canonical_consumers,
        index_entries: report.index_entries_touched(),
        lifecycle_retired: report.lifecycle_memberships_retired(),
        key_probes: report.membership_key_probes(),
        copied_nodes: report.membership_copied_avl_nodes(),
        traversed: report.membership_traversed_entries(),
        motion_commands,
        host_completed_without_effects,
    }
}

fn presentation_completed_without_effects(
    outcome: &crate::mounting::UiMountedFrameOutcome,
) -> bool {
    let mut completed_without_effects = false;
    if let crate::mounting::UiMountedFrameOutcome::Published(receipt)
    | crate::mounting::UiMountedFrameOutcome::Reconciled(receipt) = outcome
    {
        receipt.with_surface_presentations(|surfaces| {
            completed_without_effects = !surfaces.is_empty()
                && surfaces.iter().all(|surface| {
                    surface.effects().families().is_empty()
                        && surface.adapter_cost()
                            == worth_ui_host_contract::UiHostPresentationCostReport::default()
                });
        });
    }
    completed_without_effects
}
