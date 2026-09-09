use super::{
    UiNativeApplicationDefinition, UiNativeApplicationPreparation,
    UiNativeApplicationPreparationOutcome, UiNativePlatformOutcome, UiNativePlatformProfile,
};
mod preparation;

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
        let preparation_identity = self.preparation_identity;
        let binding = super::native_platform_binding::UiNativePlatformBindingGrant::issue(
            preparation_identity,
        );
        let prepared = match application.prepare(UiNativeApplicationPreparation::new(
            preparation_identity,
            binding,
        )) {
            UiNativeApplicationPreparationOutcome::Prepared(prepared) => prepared,
            UiNativeApplicationPreparationOutcome::Denied(denial) => {
                return UiNativePlatformOutcome::ApplicationPreparationDenied(denial);
            }
        };
        let host = self.profile.prepare_native_host();
        let window = worth_ui_host_native::UiNativeWindowConfiguration::qualified(
            self.profile.window().title(),
            self.profile.window().initial_logical_size(),
        );
        let (adapter, event_loop) = host.into_parts(window);
        let (bound_application, program, application_runtime, native_surface_declaration) =
            prepared.bind_qualified_native(adapter);
        let runtime_qualification = self.profile.driver_runtime_qualification();
        let driver = super::application_driver::UiNativeApplicationDriver::new(
            bound_application,
            program,
            runtime_qualification,
            application_runtime,
            native_surface_declaration,
        );
        match driver.run(event_loop) {
            Ok(report) => UiNativePlatformOutcome::Closed(
                super::UiNativePlatformCloseReceipt::from_native_report(report),
            ),
            Err(report) => UiNativePlatformOutcome::Stopped(
                super::UiNativePlatformStopReport::from_native_report(report),
            ),
        }
    }
}
