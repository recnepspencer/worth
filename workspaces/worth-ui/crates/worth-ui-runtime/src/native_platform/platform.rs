use super::{
    UiNativeApplicationDefinition, UiNativeApplicationPreparation,
    UiNativeApplicationPreparationOutcome, UiNativePlatformOutcome, UiNativePlatformProfile,
};
mod offscreen;
mod preparation;

pub use offscreen::{UiNativeOffscreenPlatformSession, UiNativeOffscreenStart};

pub struct WorthUiNativePlatform {
    _sealed: (),
}

#[must_use]
pub struct UiPreparedNativePlatform {
    profile: UiNativePlatformProfile,
    preparation_identity: u64,
}

impl UiPreparedNativePlatform {
    pub fn profile(&self) -> &UiNativePlatformProfile {
        &self.profile
    }

    pub fn run<Application>(self, application: Application) -> UiNativePlatformOutcome
    where
        Application: UiNativeApplicationDefinition,
    {
        match self.prepare_driver(application) {
            Ok((driver, event_loop)) => {
                UiNativePlatformOutcome::from_native(driver.run(event_loop))
            }
            Err(denial) => UiNativePlatformOutcome::ApplicationPreparationDenied(denial),
        }
    }

    pub(super) fn prepare_driver<Application>(
        self,
        application: Application,
    ) -> Result<
        (
            super::application_driver::UiNativeApplicationDriver,
            worth_ui_host_native::WorthUiNativeEventLoop,
        ),
        super::UiNativeApplicationPreparationDenial,
    >
    where
        Application: UiNativeApplicationDefinition,
    {
        let preparation_identity = self.preparation_identity;
        let binding = super::native_platform_binding::UiNativePlatformBindingGrant::issue(
            preparation_identity,
        );
        let prepared = match application.prepare(UiNativeApplicationPreparation::new(
            preparation_identity,
            binding,
        )) {
            UiNativeApplicationPreparationOutcome::Prepared(prepared) => prepared,
            UiNativeApplicationPreparationOutcome::Denied(denial) => return Err(denial),
        };
        let host = self.profile.prepare_native_host();
        let window = worth_ui_host_native::UiNativeWindowConfiguration::qualified(
            self.profile.window().title(),
            self.profile.window().initial_logical_size(),
        )
        .with_minimum_logical_size(self.profile.window().minimum_logical_size());
        let (adapter, event_loop) = host.into_parts(window);
        let (bound_application, program, application_runtime, native_surface_declaration) =
            prepared.bind_qualified_native(adapter);
        let driver = super::application_driver::UiNativeApplicationDriver::new(
            bound_application,
            program,
            self.profile.driver_qualification(),
            application_runtime,
            native_surface_declaration,
        );
        Ok((driver, event_loop))
    }
}

impl UiNativePlatformOutcome {
    pub(super) fn from_native(
        run: Result<
            worth_ui_host_native::UiNativeEventLoopRunReport,
            worth_ui_host_native::UiNativeEventLoopStopReport,
        >,
    ) -> Self {
        match run {
            Ok(report) => Self::Closed(super::UiNativePlatformCloseReceipt::from_native_report(
                report,
            )),
            Err(report) => Self::Stopped(super::UiNativePlatformStopReport::from_native_report(
                report,
            )),
        }
    }
}
