use crate::facade::WorthUiNativeApplicationShell;
use crate::native_platform::*;

/// A custom application owner at the same trait boundary consumed by Pulse.
/// Physical completion is synchronous in this scripted-host case.
pub(super) struct PointerApplicationRuntime;

impl UiNativeApplicationRuntime for PointerApplicationRuntime {
    fn readiness_owner_count(&self) -> UiNativeApplicationReadinessOwnerCount {
        UiNativeApplicationReadinessOwnerCount::new(0).unwrap()
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

    fn native_pointer_affordance_ready(
        &mut self,
        mut application: WorthUiNativeApplicationShell,
    ) -> Result<
        (
            WorthUiNativeApplicationShell,
            UiNativeApplicationRuntimeDirective,
        ),
        UiNativeApplicationRuntimeProgressStopped,
    > {
        assert!(application.native_pointer_presentation_pending());
        let outcome = application
            .present_frame(u64::MAX, 10)
            .unwrap_or_else(|_| panic!("custom owner prepares ordinary mounted output"));
        assert!(matches!(
            outcome,
            crate::mounting::UiMountedFrameOutcome::Published(_)
                | crate::mounting::UiMountedFrameOutcome::Unchanged(_)
        ));
        Ok((application, UiNativeApplicationRuntimeDirective::Close))
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
