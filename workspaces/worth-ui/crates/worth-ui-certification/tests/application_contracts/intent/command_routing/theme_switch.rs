use super::*;
use worth_ui::facade::app::{
    WorthUiNativeManagedIntentPosturePublicationOutcome, WorthUiNativeManagedRebindProgress,
    WorthUiNativeManagedRebindStop,
};
use worth_ui::facade::appearance::*;
use worth_ui::facade::rebind::{UiRebindExecutionPolicy, UiRebindExecutionRequest};

#[path = "theme_switch/world.rs"]
mod world;

#[path = "theme_switch/pending_ingress.rs"]
mod pending_ingress;

#[path = "theme_switch/programmatic.rs"]
mod programmatic;

#[path = "theme_switch/reconstruction.rs"]
mod reconstruction;

#[path = "theme_switch/indeterminate.rs"]
mod indeterminate;

#[path = "theme_switch/posture_recovery.rs"]
mod posture_recovery;

#[test]
fn native_action_published_posture_drives_first_theme_pixels() {
    let host = native_command_host();
    host.set_capabilities(worth_ui_host_native::appearance_capability_report());
    let mut shell = world::application(host.clone())
        .launch_native_surface()
        .unwrap();
    crate::mounted_geometry_fixture::install_native_occurrence_geometry(&mut shell);
    host.push_native_display_presented();
    let first = shell.present_frame(10, 1).unwrap_or_else(|stop| {
        use worth_ui::facade::app::WorthUiMountedFrameExecutionStop as Stop;
        match stop {
            Stop::PublicationLease(denial) => panic!("first frame lease: {denial:?}"),
            Stop::HostMeasurement(denial) => panic!("first frame measurement: {denial:?}"),
            Stop::HostMeasurementTransition(denial) => {
                panic!("first frame measurement transition: {denial:?}")
            }
            Stop::OccurrenceGeometry(denial) => panic!("first frame geometry: {denial:?}"),
            Stop::Preparation(denial) => panic!("first frame preparation: {denial:?}"),
            Stop::FrameworkTransition(_) => panic!("first frame framework transition"),
        }
    });
    assert!(matches!(first, UiMountedFrameOutcome::Published(_)));
    let UiMountedFrameOutcome::Published(receipt) = &first else {
        unreachable!()
    };
    let work = receipt.cost_report();
    assert_eq!(work.appearance().selected_instance_count(), 1);
    assert_eq!(work.appearance().materialized_context_count(), 1);
    assert!(
        work.appearance().order_retained_bytes() > 0,
        "the accepted receipt includes order admission after structural assembly"
    );
    assert!(
        work.hit_index().node_copies() > 0,
        "the first mounted hit target creates index storage: {:?}",
        work.hit_index()
    );
    drop(first);
    assert_eq!(
        host.last_surface_colors(),
        vec![worth_ui_host_contract::UiMountedRgba8::new(
            24, 48, 160, 255
        )]
    );
    let surface = match shell.inspect_mounted_frame(UiMountedInspectionRequest::current()) {
        UiMountedInspectionReceipt::Available(frame) => {
            frame.presentation().surfaces()[0].semantic_surface()
        }
        _ => panic!("native controller requires an accepted mounted surface"),
    };
    let ingress = shell.admit_native_intent_observations(
        UiIntentDefinition::<AdvanceStatus>::runtime_service(
            UiIntentRuntimeServiceDestination::InvokeCommand,
        ),
        shortcut_drain(
            shell.host_session_identity().as_u64(),
            current_presentation(&shell),
            false,
        ),
        crate::intent::execution::execution_deadline(20),
    );
    let mut transitions = ingress.into_transitions().into_vec();
    assert_eq!(transitions.len(), 1);
    let WorthUiNativeIntentTransition::AttemptPrepared(prepared) = transitions.remove(0) else {
        panic!("native shortcut must admit a real action before theme ingress");
    };
    host.push_native_display_settled_without_effects();
    let publication =
        match shell.begin_managed_native_intent_posture_publication(prepared.into_posture(), 21) {
            Ok(WorthUiNativeManagedIntentPosturePublicationOutcome::Published(receipt)) => receipt,
            Err(denial) => panic!("action posture admission failed: {denial:?}"),
            Ok(_) => panic!("action posture must publish through the managed production handoff"),
        };
    let origin = shell
        .issue_theme_switch_origin_from_publication(&publication)
        .unwrap();
    drop(publication);
    let capability = shell
        .admit_appearance_theme(
            surface,
            &UiThemeDefinitionIdentity::new("theme.command.green").unwrap(),
        )
        .unwrap();
    let request = UiThemeSwitchRequest::new(
        origin.clone(),
        surface,
        shell
            .active_theme_binding(surface)
            .unwrap()
            .binding_generation(),
        capability,
    );
    host.push_native_display_presented();
    match shell
        .begin_theme_switch(
            request.clone(),
            UiRebindExecutionPolicy::ordinary(),
            UiRebindExecutionRequest::new(22),
        )
        .unwrap()
    {
        WorthUiNativeManagedRebindProgress::Published(_) => {}
        WorthUiNativeManagedRebindProgress::Stopped(stop) => {
            panic!("theme stopped before publication: {stop:?}")
        }
        WorthUiNativeManagedRebindProgress::AwaitingProgress => {
            panic!("theme unexpectedly awaits host progress")
        }
        _ => panic!("theme returned a non-publication progress posture"),
    }
    // These colors come from the host's consumed appearance work, after the
    // real native action and managed publication, not from a resolver fixture.
    assert_eq!(
        host.last_surface_colors(),
        vec![worth_ui_host_contract::UiMountedRgba8::new(
            16, 144, 48, 255
        )]
    );
    let active = shell.active_theme_binding(surface).unwrap();
    let duplicate = UiThemeSwitchRequest::new(
        origin,
        surface,
        active.binding_generation(),
        active.capability().clone(),
    );
    assert!(matches!(
        shell
            .begin_theme_switch(
                duplicate,
                UiRebindExecutionPolicy::ordinary(),
                UiRebindExecutionRequest::new(23)
            )
            .unwrap(),
        WorthUiNativeManagedRebindProgress::Stopped(WorthUiNativeManagedRebindStop::Duplicate)
    ));
}
