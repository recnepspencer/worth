use worth_ui::facade::source::WorthUiFilesystemSourceProvider;
use worth_ui_certification::scenario::filesystem_application_lifecycle::FilesystemApplicationLifecycleScenario;
use worth_ui_runtime::certification_support::WorthUiPresentationAsyncInstallationCertificationExt;
use worth_ui_runtime::facade::mounted::{UiHostSurfaceCancellationOutcome, UiMountedFrameOutcome};

use super::super::filesystem_contract_workspace::FilesystemContractWorkspace;
use super::super::mounted_host_protocol::scripted_host::{
    ScriptedPresentationHost, ScriptedSurfaceCompletion,
};

#[test]
fn native_partial_effect_cancellation_advances_query_to_recovery_required() {
    let host = ScriptedPresentationHost::native_display();
    let mut shell = native_semantic_shell(host.clone(), "partial-effect-query-cancellation");
    host.push_in_flight(
        vec![ScriptedSurfaceCompletion::Pending],
        UiHostSurfaceCancellationOutcome::EffectsMayHaveBegun,
    );
    crate::mounted_geometry_fixture::install_native_occurrence_geometry(&mut shell);
    let presented = shell
        .present_frame(2, 0)
        .unwrap_or_else(|_| panic!("native semantic frame must enter presentation"));
    let in_flight = super::expect_in_flight(presented);
    assert!(matches!(
        shell.complete_frame_presentation(in_flight, 2),
        UiMountedFrameOutcome::PresentationIndeterminate(_)
    ));
    assert_query_recovery_and_close(shell.shutdown());
}

#[test]
fn direct_host_indeterminacy_advances_query_to_recovery_required() {
    let host = ScriptedPresentationHost::native_display();
    let mut shell = native_semantic_shell(host.clone(), "direct-query-indeterminate");
    host.push_presentation(
        worth_ui_host_contract::UiHostSurfacePresentationOutcome::PresentationIndeterminate,
    );
    crate::mounted_geometry_fixture::install_native_occurrence_geometry(&mut shell);
    assert!(matches!(
        shell
            .present_frame(2, 0)
            .unwrap_or_else(|_| panic!("native semantic frame must enter presentation")),
        UiMountedFrameOutcome::PresentationIndeterminate(_)
    ));
    assert_query_recovery_and_close(shell.shutdown());
}

fn native_semantic_shell(
    host: ScriptedPresentationHost,
    label: &str,
) -> worth_ui_runtime::facade::entry::WorthUiNativeApplicationShell {
    host.set_capabilities(
        worth_ui_host_contract::WorthUiHostCapabilityReport::available(vec![
            worth_ui_host_contract::WorthUiHostCapability::NativePaint,
            worth_ui_host_contract::WorthUiHostCapability::ViewportObservation,
            worth_ui_host_contract::WorthUiHostCapability::DpiObservation,
            worth_ui_host_contract::WorthUiHostCapability::TextIntrinsicMeasurement,
            worth_ui_host_contract::WorthUiHostCapability::TextBaselineMeasurement,
            worth_ui_host_contract::WorthUiHostCapability::FontMetrics,
        ]),
    );
    let scenario = FilesystemApplicationLifecycleScenario::new(label);
    let workspace = FilesystemContractWorkspace::new(label);
    workspace.write(
        "app/main.wui",
        &FilesystemApplicationLifecycleScenario::phase5_cancellation_source_text(),
    );
    let snapshot = WorthUiFilesystemSourceProvider::new(workspace.root())
        .read()
        .unwrap();
    let capabilities = scenario.phase5_cancellation_application(host.clone());
    let submission = FilesystemApplicationLifecycleScenario::lower_snapshot(
        snapshot,
        capabilities.capabilities(),
    );
    workspace.close();
    let mut application =
        scenario.prepare_phase5_cancellation_application_with_host(submission, host);
    install_presentation_async(&mut application);
    let mut shell = application.launch_native_surface().unwrap();
    shell
        .apply_component_semantic_text(&[
            worth_ui_runtime::facade::entry::UiNativeComponentSemanticTextChange::new(
                "component:phase5.cancel.component",
                "partial effects",
            )
            .unwrap(),
        ])
        .unwrap();
    shell
}

fn install_presentation_async(application: &mut worth_ui::facade::app::WorthUiApp) {
    let plan =
        worth_ui::facade::query_binding::WorthUiPresentationAsyncHostPlan::prepare().unwrap();
    let (request, completion) = plan.into_parts();
    let query = worth_query_host::facade::runtime::WorthQueryExecutionRuntimeInstaller::new()
        .install(request.generation(), request.into_packages())
        .unwrap();
    application
        .install_presentation_async_for_certification(completion.complete(query).unwrap())
        .unwrap();
}

fn assert_query_recovery_and_close(
    shutdown: worth_ui_runtime::facade::entry::WorthUiNativeApplicationShutdownReceipt,
) {
    let transitions = shutdown.query_transitions();
    for kind in [
        worth_ui_query_binding::WorthUiPresentationTransitionKind::Unresolved,
        worth_ui_query_binding::WorthUiPresentationTransitionKind::RecoveryRequired,
    ] {
        assert!(
            transitions
                .iter()
                .any(|transition| transition.kind() == kind),
            "production uncertainty must retain {kind:?}, observed {transitions:?}"
        );
    }
    assert!(shutdown.query_transition_trace_complete());
    assert!(shutdown.query_close_complete());
    assert!(shutdown.closed_query_resources() > 0);
}
