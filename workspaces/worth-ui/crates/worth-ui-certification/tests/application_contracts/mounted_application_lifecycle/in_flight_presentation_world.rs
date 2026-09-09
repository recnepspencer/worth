use worth_ui_runtime::facade::mounted::{UiHostSurfacePresentationMode, UiMountedFrameRequest};
use worth_ui_test_support::WorthUiFrameworkTurnCertificationExt;
use worth_ui_test_support::WorthUiMountedIdentityCertificationExt;
use worth_ui_test_support::{
    WorthUiMountedFrameExecutionCertificationExt, WorthUiMountedPublicationCertificationExt,
};

use super::known_empty_surface_world::{
    first_node, mounted_application_with_host, mounted_application_with_host_and_visual_policy,
    profile,
};
use crate::mounted_host_protocol::scripted_host::ScriptedPresentationHost;

pub(crate) struct InFlightPresentationWorld {
    pub session: worth_ui::facade::app::WorthUiActiveApplicationSession,
    pub handle: worth_ui_runtime::facade::mounted::UiMountedPresentationInFlight,
}

impl InFlightPresentationWorld {
    pub(crate) fn accepted(label: &str) -> Self {
        let host = ScriptedPresentationHost::default();
        let (mut session, _) = mounted_session(host.clone(), label, 1);
        let frame = prepared(&mut session);
        host.push_in_flight(
            vec![crate::mounted_host_protocol::scripted_host::presented_completion()],
            worth_ui_runtime::facade::mounted::UiHostSurfaceCancellationOutcome::EffectsMayHaveBegun,
        );
        let outcome = session.present_prepared_mounted_frame(
            frame,
            worth_ui_runtime::facade::mounted::UiPresentationDeadline::at_tick(20),
            0,
        );
        let handle = match outcome {
            worth_ui_runtime::facade::mounted::UiMountedFrameOutcome::InFlight(handle) => handle,
            _ => panic!("canonical in-flight world requires host acceptance"),
        };
        Self { session, handle }
    }
}

pub(crate) fn mounted_session(
    host: ScriptedPresentationHost,
    label: &str,
    surface_count: usize,
) -> (
    worth_ui::facade::app::WorthUiActiveApplicationSession,
    Vec<worth_ui_runtime::facade::mounted::UiSurfaceBindingGeneration>,
) {
    mount_surfaces(
        mounted_application_with_host(label, host).launch().unwrap(),
        surface_count,
    )
}

pub(crate) fn mounted_session_with_visual_policy(
    host: ScriptedPresentationHost,
    label: &str,
    surface_count: usize,
    policy: worth_ui::facade::inspection::UiVisualInspectionPolicy,
) -> (
    worth_ui::facade::app::WorthUiActiveApplicationSession,
    Vec<worth_ui_runtime::facade::mounted::UiSurfaceBindingGeneration>,
) {
    mount_surfaces(
        mounted_application_with_host_and_visual_policy(label, host, policy)
            .launch()
            .unwrap(),
        surface_count,
    )
}

pub(crate) fn mounted_session_with_retention_budget(
    host: ScriptedPresentationHost,
    label: &str,
    surface_count: usize,
    budget: worth_ui_runtime::facade::mounted::UiMountedFrameRetentionBudget,
) -> (
    worth_ui::facade::app::WorthUiActiveApplicationSession,
    Vec<worth_ui_runtime::facade::mounted::UiSurfaceBindingGeneration>,
) {
    mount_surfaces(
        super::known_empty_surface_world::mounted_application_with_host_and_retention_budget(
            label, host, budget,
        )
        .launch()
        .unwrap(),
        surface_count,
    )
}

fn mount_surfaces(
    session: worth_ui::facade::app::WorthUiActiveApplicationSession,
    surface_count: usize,
) -> (
    worth_ui::facade::app::WorthUiActiveApplicationSession,
    Vec<worth_ui_runtime::facade::mounted::UiSurfaceBindingGeneration>,
) {
    mount_surfaces_with_mode(
        session,
        surface_count,
        UiHostSurfacePresentationMode::RecordOnly,
    )
}

fn mount_surfaces_with_mode(
    mut session: worth_ui::facade::app::WorthUiActiveApplicationSession,
    surface_count: usize,
    mode: UiHostSurfacePresentationMode,
) -> (
    worth_ui::facade::app::WorthUiActiveApplicationSession,
    Vec<worth_ui_runtime::facade::mounted::UiSurfaceBindingGeneration>,
) {
    let node = first_node(&session);
    let mut bindings = Vec::new();
    for epoch in 0..surface_count {
        let surface = session.create_semantic_surface().unwrap();
        let binding = session
            .register_host_surface(surface, mode, profile(u64::try_from(epoch + 1).unwrap()))
            .unwrap()
            .binding_generation();
        session.mount_instance(node, surface).unwrap();
        bindings.push(binding);
    }
    bindings.sort();
    crate::mounted_geometry_fixture::install_current_occurrence_geometry(&mut session);
    (session, bindings)
}

pub(crate) fn prepared(
    session: &mut worth_ui::facade::app::WorthUiActiveApplicationSession,
) -> worth_ui_runtime::facade::mounted::UiPreparedMountedFrame {
    crate::mounted_geometry_fixture::install_current_occurrence_geometry(session);
    session
        .execute_framework_turn(|_| {})
        .expect("no mounted presentation lease is active")
        .into_execution()
        .unwrap_or_else(|_| panic!("empty source turn permits presentation preparation"))
        .prepare_mounted_frame(UiMountedFrameRequest::all_bound_surfaces())
        .unwrap()
}
