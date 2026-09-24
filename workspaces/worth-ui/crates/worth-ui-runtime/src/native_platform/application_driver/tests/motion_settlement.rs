//! A Motion tick that settles mounted geometry without host sample work leaves
//! the frame showing it owed to the application runtime. The driver wakes the
//! runtime through `native_motion_settlement_ready` exactly when that debt
//! exists and the runtime is active, and never otherwise.

use std::{cell::Cell, rc::Rc};

use crate::facade::WorthUiNativeApplicationShell;
use crate::native_platform::*;

/// An application owner that lays out and presents through its ordinary frame
/// lifecycle when woken for a settlement, as Pulse does, and counts each wake.
struct SettlementRuntime {
    wakes: Rc<Cell<u32>>,
    work: SettlementWork,
}

#[derive(Clone, Copy)]
enum SettlementWork {
    FrameOnly,
    FrameThenMotion,
}

impl UiNativeApplicationRuntime for SettlementRuntime {
    fn readiness_owner_count(&self) -> UiNativeApplicationReadinessOwnerCount {
        UiNativeApplicationReadinessOwnerCount::new(0).expect("zero owners is admitted")
    }

    fn activate(
        &mut self,
        application: WorthUiNativeApplicationShell,
        ports: Box<[UiNativeApplicationReadinessPort]>,
    ) -> Result<WorthUiNativeApplicationShell, UiNativeApplicationRuntimeActivationStopped> {
        assert!(ports.is_empty());
        Ok(application)
    }

    fn readiness_ready(
        &mut self,
        _: WorthUiNativeApplicationShell,
        _: u8,
        _: u64,
    ) -> Result<
        (
            WorthUiNativeApplicationShell,
            UiNativeApplicationRuntimeDirective,
        ),
        UiNativeApplicationRuntimeProgressStopped,
    > {
        unreachable!("this runtime has no readiness owners")
    }

    fn native_motion_settlement_ready(
        &mut self,
        mut application: WorthUiNativeApplicationShell,
    ) -> Result<
        (
            WorthUiNativeApplicationShell,
            UiNativeApplicationRuntimeDirective,
        ),
        UiNativeApplicationRuntimeProgressStopped,
    > {
        assert!(
            application.native_application_presentation_pending(),
            "the driver wakes the runtime only while a frame is owed"
        );
        self.wakes.set(self.wakes.get() + 1);
        super::super::program_progress::layout::complete_program_layout(&mut application)
            .unwrap_or_else(|()| panic!("a live owner completes its own layout before presenting"));
        let outcome = match application.present_frame(u64::MAX, 10) {
            Ok(outcome) => outcome,
            Err(stop) => panic!(
                "the owner presents ordinary mounted output: {}",
                frame_stop_name(&stop)
            ),
        };
        assert!(matches!(
            outcome,
            crate::mounting::UiMountedFrameOutcome::Published(_)
                | crate::mounting::UiMountedFrameOutcome::Unchanged(_)
        ));
        if matches!(self.work, SettlementWork::FrameThenMotion) {
            crate::facade::entry::native_motion_test_support::install_sampling_track(
                &mut application,
            );
        }
        Ok((application, UiNativeApplicationRuntimeDirective::Continue))
    }

    fn native_pointer_affordance_ready(
        &mut self,
        _: WorthUiNativeApplicationShell,
    ) -> Result<
        (
            WorthUiNativeApplicationShell,
            UiNativeApplicationRuntimeDirective,
        ),
        UiNativeApplicationRuntimeProgressStopped,
    > {
        unreachable!("no pointer affordance is refreshed in these scenarios")
    }

    fn close(
        self: Box<Self>,
        application: WorthUiNativeApplicationShell,
    ) -> Result<UiNativeApplicationRuntimeClosed, UiNativeApplicationRuntimeCloseIncomplete> {
        Ok(UiNativeApplicationRuntimeClosed::from_application_shutdown(
            application.shutdown(),
        ))
    }
}

fn frame_stop_name(stop: &crate::facade::entry::WorthUiMountedFrameExecutionStop<'_>) -> String {
    use crate::facade::entry::WorthUiMountedFrameExecutionStop as Stop;
    match stop {
        Stop::PublicationLease(denial) => format!("PublicationLease({denial:?})"),
        Stop::HostMeasurement(_) => "HostMeasurement".to_owned(),
        Stop::HostMeasurementTransition(_) => "HostMeasurementTransition".to_owned(),
        Stop::OccurrenceGeometry(denial) => format!("OccurrenceGeometry({denial:?})"),
        Stop::FrameworkTransition(_) => "FrameworkTransition".to_owned(),
        Stop::Preparation(denial) => format!("Preparation({denial:?})"),
    }
}

fn driver_with_owed_frame(
    active: bool,
    work: SettlementWork,
) -> (
    super::super::UiNativeApplicationDriver,
    Rc<Cell<u32>>,
    crate::certification_support::ScriptedPresentationHost,
) {
    use crate::runtime::tests::active_application_session_test_support::source_backed_component_app_with_host_and_viewport_allocation;

    let host = crate::certification_support::ScriptedPresentationHost::native_display();
    let mut shell = source_backed_component_app_with_host_and_viewport_allocation(host.clone())
        .launch_native_surface()
        .expect("native test shell should launch");
    shell.observe_native_viewport_readiness([800, 600], 1_000, false);
    assert!(
        shell.native_application_presentation_pending(),
        "a launched shell owes its first projection until a frame presents it"
    );
    let wakes = Rc::new(Cell::new(0));
    let mut runtime = Box::new(SettlementRuntime {
        wakes: Rc::clone(&wakes),
        work,
    });
    let shell = runtime
        .activate(shell, Box::new([]))
        .unwrap_or_else(|_| panic!("activate settlement owner"));
    let mut driver = super::super::UiNativeApplicationDriver::from_launched_shell_for_test(shell);
    driver.application_runtime = Some(runtime);
    driver.application_runtime_active = active;
    (driver, wakes, host)
}

/// An owed frame wakes the active runtime once; the runtime presents it, and
/// the next settlement finds nothing owed and wakes nobody.
#[test]
fn an_owed_frame_wakes_the_active_runtime_until_it_presents() {
    let (mut driver, wakes, host) = driver_with_owed_frame(true, SettlementWork::FrameOnly);
    host.push_native_display_presented();

    let directive = driver
        .progress_application_runtime_motion_settlement(false)
        .expect("an active runtime settles the owed frame");
    assert_eq!(
        directive,
        worth_ui_host_native::UiNativeEventLoopDirective::Continue
    );
    assert_eq!(wakes.get(), 1, "one owed frame is one wake");
    assert!(
        !driver
            .shell
            .as_ref()
            .expect("the driver holds its shell again")
            .native_application_presentation_pending(),
        "the presented frame clears the debt"
    );

    let directive = driver
        .progress_application_runtime_motion_settlement(false)
        .expect("nothing owed continues without waking the runtime");
    assert_eq!(
        directive,
        worth_ui_host_native::UiNativeEventLoopDirective::Continue
    );
    assert_eq!(wakes.get(), 1, "no debt, no wake");
    assert!(driver.close_application_runtime().unwrap().is_some());
}

/// A runtime that is installed but not yet active is never woken, even with a
/// frame owed: activation, not installation, admits it to the settlement lane.
#[test]
fn an_inactive_runtime_is_not_woken_for_an_owed_frame() {
    let (mut driver, wakes, _host) = driver_with_owed_frame(false, SettlementWork::FrameOnly);

    let directive = driver
        .progress_application_runtime_motion_settlement(false)
        .expect("an inactive runtime is skipped, not failed");
    assert_eq!(
        directive,
        worth_ui_host_native::UiNativeEventLoopDirective::Continue
    );
    assert_eq!(wakes.get(), 0);
    assert!(
        driver
            .shell
            .as_ref()
            .expect("the driver keeps its shell")
            .native_application_presentation_pending(),
        "the debt stays owed until an active runtime presents it"
    );
}

#[test]
fn motion_started_by_settlement_callback_gets_a_readiness_wake() {
    use std::{sync::mpsc, time::Duration};
    let (mut driver, wakes, host) = driver_with_owed_frame(true, SettlementWork::FrameThenMotion);
    let (send, receive) = mpsc::channel();
    driver.motion_support_installed = true;
    driver.motion_readiness = Some(super::super::UiNativeMotionReadinessLane::start_for_test(
        move || send.send(()).map_err(|_| ()),
    ));
    assert!(!driver.motion_can_request_readiness());
    host.push_native_display_presented();
    driver
        .progress_application_runtime_motion_settlement(false)
        .unwrap();
    assert_eq!(wakes.get(), 1);
    assert!(
        driver.motion_can_request_readiness(),
        "the callback really installed runnable work"
    );
    let woke = receive.recv_timeout(Duration::from_secs(1)).is_ok();
    if !woke {
        // A missing wake is distinct from a broken worker or notification sink.
        driver.arm_motion_readiness_now();
        receive
            .recv_timeout(Duration::from_secs(1))
            .expect("the same readiness worker responds when armed");
    }
    assert!(driver.close_application_runtime().unwrap().is_some());
    assert!(
        woke,
        "settlement created runnable Motion work but the driver returned without waking it"
    );
}
