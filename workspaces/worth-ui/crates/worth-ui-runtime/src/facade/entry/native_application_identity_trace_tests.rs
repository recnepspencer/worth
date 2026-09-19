use crate::runtime::tests::active_application_session_test_support::{
    component_candidate_submission, component_candidate_submission_with_removal_token,
    source_backed_component_app_with_host,
};

use super::native_application_identity_trace_test_support::{
    assert_indexed_trace_cost, assert_same_authored_affinity, assert_trace_authored_affinity,
    assert_trace_runtime_identity, authored_presented_identity, completed, frame_receipt,
    install_bound_surface_geometry, mounted_instance, mounted_source_oracle,
    only_presented_identity, presented_graph_node, remount_presented_instance,
    replace_application_with_provenance, retained_trace,
};
use super::native_identity_trace_host::NativeIdentityTraceHost;
use crate::mounting::UiMountedFrameOutcome;

#[test]
fn retained_declaration_keeps_its_authored_trace_across_application_replacement() {
    let app = source_backed_component_app_with_host(NativeIdentityTraceHost::default());
    let declaration_artifacts = app.declaration_artifacts().to_vec();
    let mut shell = app
        .launch_native_surface()
        .expect("source-backed application should launch through the native lifecycle");
    install_bound_surface_geometry(&mut shell);
    let predecessor_oracle = mounted_source_oracle(&declaration_artifacts, shell.session.graph());
    let predecessor = frame_receipt(completed(shell.present_frame(100, 1)));
    let predecessor_identity = authored_presented_identity(
        &shell,
        &predecessor,
        predecessor_oracle.authored_provenance_digest(),
        "predecessor",
    );
    let predecessor_generation = predecessor.generation().clone();

    let candidate = component_candidate_submission_with_removal_token(
        &shell.session,
        "identity-trace-replacement",
        "workspace.component.active_session_current",
    );
    let (successor, successor_provenance) =
        replace_application_with_provenance(&mut shell, candidate);
    let successor_identity = only_presented_identity(&shell, &successor);
    let successor_graph_node = presented_graph_node(&shell, successor_identity);
    let successor_oracle = successor_provenance
        .iter()
        .find(|oracle| oracle.graph_node() == successor_graph_node)
        .expect("mounted successor declaration should retain candidate provenance");

    let predecessor_trace = retained_trace(&shell, &predecessor, predecessor_identity);
    let successor_trace = retained_trace(&shell, &successor, successor_identity);

    assert_eq!(predecessor_trace.generation(), &predecessor_generation);
    assert_eq!(successor_trace.generation(), successor.generation());
    assert_ne!(predecessor_trace.generation(), successor_trace.generation());
    assert_trace_runtime_identity(&predecessor_trace, predecessor_identity);
    assert_trace_runtime_identity(&successor_trace, successor_identity);
    assert_eq!(
        predecessor_trace
            .authored_provenance()
            .source_artifact()
            .path(),
        "app/main.wui"
    );
    assert_trace_authored_affinity(&predecessor_trace, &predecessor_oracle);
    assert_trace_authored_affinity(&successor_trace, successor_oracle);
    assert_eq!(
        predecessor_trace.authored_provenance().source_generation(),
        successor_trace.authored_provenance().source_generation()
    );
    assert_indexed_trace_cost(predecessor_trace.cost());
    assert_indexed_trace_cost(successor_trace.cost());
    let shutdown = shell.shutdown();
    assert!(shutdown.host_session_released());
    assert_eq!(shutdown.released_surface_count(), 1);
}

#[test]
fn remount_mints_new_runtime_identity_without_losing_authored_affinity() {
    let app = source_backed_component_app_with_host(NativeIdentityTraceHost::default());
    let declaration_artifacts = app.declaration_artifacts().to_vec();
    let mut shell = app
        .launch_native_surface()
        .expect("source-backed application should launch through the native lifecycle");
    install_bound_surface_geometry(&mut shell);
    let authored_oracle = mounted_source_oracle(&declaration_artifacts, shell.session.graph());
    let predecessor = frame_receipt(completed(shell.present_frame(100, 1)));
    let predecessor_identity = authored_presented_identity(
        &shell,
        &predecessor,
        authored_oracle.authored_provenance_digest(),
        "predecessor",
    );
    let (successor, successor_instance, predecessor_incarnation) =
        remount_presented_instance(&mut shell, predecessor_identity);
    assert_ne!(successor.frame(), predecessor.frame());
    assert_eq!(successor.predecessor(), Some(predecessor.frame()));
    let successor_identity = authored_presented_identity(
        &shell,
        &successor,
        authored_oracle.authored_provenance_digest(),
        "successor",
    );
    let successor_view = mounted_instance(&shell.session.mounted.view(), successor_instance);

    let predecessor_trace = retained_trace(&shell, &predecessor, predecessor_identity);
    let successor_trace = retained_trace(&shell, &successor, successor_identity);

    assert_ne!(
        predecessor_identity.mounted_instance_identity(),
        successor_identity.mounted_instance_identity()
    );
    assert_ne!(
        predecessor_identity.node_receipt_identity(),
        successor_identity.node_receipt_identity()
    );
    assert_ne!(predecessor_incarnation, successor_view.mount_incarnation());
    assert_eq!(predecessor_trace.incarnation(), predecessor_incarnation);
    assert_eq!(
        successor_trace.incarnation(),
        successor_view.mount_incarnation()
    );
    assert_same_authored_affinity(&predecessor_trace, &successor_trace);
    assert_trace_authored_affinity(&predecessor_trace, &authored_oracle);
    assert_indexed_trace_cost(predecessor_trace.cost());
    assert_indexed_trace_cost(successor_trace.cost());
    let shutdown = shell.shutdown();
    assert!(shutdown.host_session_released());
    assert_eq!(shutdown.released_surface_count(), 1);
}

#[test]
fn replacement_with_a_different_declaration_identity_retires_the_old_mount() {
    let app = source_backed_component_app_with_host(NativeIdentityTraceHost::default());
    let mut shell = app
        .launch_native_surface()
        .expect("source-backed application should launch through the native lifecycle");
    install_bound_surface_geometry(&mut shell);
    let predecessor = frame_receipt(completed(shell.present_frame(100, 1)));
    assert!(!shell.session.mounted.view().mounted_instances().is_empty());

    let candidate = component_candidate_submission(
        &shell.session,
        "identity-trace-declaration-replacement",
        "workspace.component.active_session_candidate",
    );
    let (successor, _) = replace_application_with_provenance(&mut shell, candidate);
    let mounted = shell.session.mounted.view();

    assert_eq!(successor.predecessor(), Some(predecessor.frame()));
    assert!(mounted.mounted_instances().is_empty());
    assert!(mounted.frame_receipts().is_empty());
    let shutdown = shell.shutdown();
    assert!(shutdown.host_session_released());
    assert_eq!(shutdown.released_surface_count(), 1);
}

#[test]
fn new_surface_receives_owner_issued_reconstruction_and_next_state_becomes_current() {
    let host = NativeIdentityTraceHost::default();
    let app = source_backed_component_app_with_host(host.clone());
    let mut shell = app
        .launch_native_surface()
        .expect("source-backed application should launch");
    install_bound_surface_geometry(&mut shell);
    frame_receipt(completed(shell.present_frame(100, 1)));
    let graph_node = shell
        .session
        .graph()
        .node_identities()
        .next()
        .expect("fixture should contain a graph node");
    let surface = shell
        .session
        .create_semantic_surface()
        .expect("second semantic surface should be available");
    let profile = crate::mounting::UiSurfaceBindingProfile::new(
        1_000,
        crate::mounting::UiSurfaceBindingCoordinatePosture::LogicalPoints,
        1,
    )
    .expect("surface profile should be lawful");
    shell
        .session
        .register_host_surface(
            surface,
            worth_ui_host_contract::UiHostSurfacePresentationMode::NativeDisplay,
            profile,
        )
        .expect("second host surface should register known empty");
    let handle = shell
        .session
        .mounted_graph_node(graph_node)
        .expect("graph node should be mountable on the second surface");
    shell
        .session
        .mount_instance(handle, surface)
        .expect("second surface should carry mounted content");
    crate::facade::entry::mounted_occurrence_geometry_test_support::
        refresh_nonoverlapping_surface_geometry(&mut shell.session, surface);
    let calls_before_reconstruction = host.presentation_calls();
    let successor = frame_receipt(completed(shell.present_frame(200, 2)));
    assert_eq!(host.presentation_calls(), calls_before_reconstruction + 2);
    assert_eq!(
        shell
            .session
            .current_mounted_publication()
            .expect("reconstructed successor becomes current")
            .frame(),
        successor.frame()
    );
    let shutdown = shell.shutdown();
    assert!(shutdown.host_session_released());
    assert_eq!(shutdown.released_surface_count(), 2);
}

#[test]
fn second_frame_prepared_from_one_predecessor_is_denied_before_host_effects() {
    let host = NativeIdentityTraceHost::default();
    let app = source_backed_component_app_with_host(host.clone());
    let mut shell = app
        .launch_native_surface()
        .expect("source-backed application should launch");
    install_bound_surface_geometry(&mut shell);
    let predecessor = frame_receipt(completed(shell.present_frame(100, 1)));
    let identity = first_presented_identity(&shell, &predecessor);
    remount_without_presentation(&mut shell, identity);

    let completion = shell
        .session
        .execute_framework_turn(|_| {})
        .expect("framework turn should be available");
    let mut execution = completion
        .into_execution()
        .unwrap_or_else(|_| panic!("framework turn should execute"));
    let request = crate::mounting::UiMountedFrameRequest::all_bound_surfaces();
    let first = execution
        .prepare_mounted_frame_internal(request.clone())
        .expect("first candidate should prepare");
    let second = execution
        .prepare_mounted_frame_internal(request)
        .expect("second candidate should prepare against the same predecessor");
    drop(execution);

    let successor = frame_receipt(shell.session.present_prepared_mounted_frame_internal(
        first,
        worth_ui_host_contract::UiPresentationDeadline::at_tick(200),
        2,
    ));
    let calls_after_successor = host.presentation_calls();
    let stale = shell.session.present_prepared_mounted_frame_internal(
        second,
        worth_ui_host_contract::UiPresentationDeadline::at_tick(300),
        3,
    );
    let rejection = match stale {
        UiMountedFrameOutcome::AdmissionDenied(rejection) => rejection,
        _ => panic!("second candidate must be stale before presentation admission"),
    };
    assert_eq!(
        rejection.denial(),
        crate::mounting::UiMountedPresentationAdmissionDenial::CandidatePreparation(
            worth_ui_host_contract::UiHostSurfacePresentationDenial::StalePredecessor,
        )
    );
    assert_eq!(host.presentation_calls(), calls_after_successor);
    assert_eq!(
        shell
            .session
            .current_mounted_publication()
            .expect("successor remains current")
            .frame(),
        successor.frame()
    );
    assert!(shell.shutdown().host_session_released());
}

fn first_presented_identity(
    shell: &super::WorthUiNativeApplicationShell,
    receipt: &crate::mounting::UiMountedFramePublicationReceipt,
) -> crate::mounting::UiMountedFrameIdentityView {
    shell
        .session
        .mounted
        .view()
        .frame_receipts()
        .iter()
        .copied()
        .find(|identity| identity.frame_identity() == receipt.frame())
        .expect("published frame should carry a mounted receipt")
}

fn remount_without_presentation(
    shell: &mut super::WorthUiNativeApplicationShell,
    predecessor: crate::mounting::UiMountedFrameIdentityView,
) {
    let mounted = shell.session.mounted.view();
    let instance = mounted_instance(&mounted, predecessor.mounted_instance_identity());
    let graph_node = instance.graph_node_identity();
    let surface = mounted.surface_bindings()[0].semantic_surface_identity();
    drop(mounted);
    shell
        .session
        .unmount_instance(predecessor.mounted_instance_identity())
        .expect("presented instance should unmount");
    let handle = shell
        .session
        .mounted_graph_node(graph_node)
        .expect("graph node remains mountable");
    shell
        .session
        .mount_instance(handle, surface)
        .expect("graph node should remount");
    crate::facade::entry::mounted_occurrence_geometry_test_support::
        refresh_nonoverlapping_surface_geometry(&mut shell.session, surface);
}
