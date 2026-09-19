use std::path::Path;

use worth_ui::facade::query_binding::{
    UiApplicationScalarProjectionRegistration, WorthUiStatusOwnerError, WorthUiStatusSourceOwner,
};

use super::PlatformPulseQueryLifecycle;
use super::{PlatformPulseExternalValueWatch, PlatformPulseExternalValueWatchDenial};

pub(crate) struct InstalledPlatformPulseQuery {
    pub(crate) registration: UiApplicationScalarProjectionRegistration,
    pub(crate) lifecycle: PlatformPulseQueryLifecycle,
    pub(crate) watcher: PlatformPulseExternalValueWatch,
}

impl InstalledPlatformPulseQuery {
    pub(crate) fn shutdown(self) {
        let _ = self.lifecycle.close();
        let _ = self.watcher.shutdown();
    }
}

#[derive(Debug)]
pub(crate) enum PlatformPulseQueryInstallationDenial {
    Application(WorthUiStatusOwnerError),
    Watch(PlatformPulseExternalValueWatchDenial),
}

impl std::fmt::Display for PlatformPulseQueryInstallationDenial {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Application(detail) => write!(formatter, "application: {detail}"),
            Self::Watch(denial) => write!(formatter, "source watch: {denial}"),
        }
    }
}

pub(crate) fn install(
    source_root: &Path,
) -> Result<InstalledPlatformPulseQuery, PlatformPulseQueryInstallationDenial> {
    let owner = WorthUiStatusSourceOwner::install()
        .map_err(PlatformPulseQueryInstallationDenial::Application)?;
    let (registration, initial) = owner
        .initial_projection()
        .map_err(PlatformPulseQueryInstallationDenial::Application)?;
    let watcher = PlatformPulseExternalValueWatch::spawn(source_root)
        .map_err(PlatformPulseQueryInstallationDenial::Watch)?;
    Ok(InstalledPlatformPulseQuery {
        registration,
        lifecycle: PlatformPulseQueryLifecycle::new(owner, initial),
        watcher,
    })
}

pub(crate) fn install_native_presentation_async(
) -> Option<worth_ui::facade::query_binding::WorthUiPresentationAsyncInstallation> {
    install_native_presentation_async_plan(
        worth_ui::facade::query_binding::WorthUiPresentationAsyncHostPlan::prepare().ok()?,
    )
}

#[cfg(feature = "executable-world")]
pub(crate) fn install_native_presentation_async_for_transition_courtroom(
) -> Option<worth_ui::facade::query_binding::WorthUiPresentationAsyncInstallation> {
    install_native_presentation_async()
}

fn install_native_presentation_async_plan(
    plan: worth_ui::facade::query_binding::WorthUiPresentationAsyncHostPlan,
) -> Option<worth_ui::facade::query_binding::WorthUiPresentationAsyncInstallation> {
    let (request, completion) = plan.into_parts();
    let installation =
        worth_query_host::facade::runtime::WorthQueryExecutionRuntimeInstaller::new()
            .install(request.generation(), request.into_packages())
            .ok()?;
    completion.complete(installation).ok()
}
