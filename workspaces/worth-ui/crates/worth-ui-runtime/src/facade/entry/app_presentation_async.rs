use super::WorthUiApp;

#[cfg(any(test, feature = "certification-support"))]
pub(crate) enum WorthUiPresentationAsyncInstallationDenial {
    AlreadyInstalled(Box<worth_ui_query_binding::WorthUiPresentationAsyncInstallation>),
}

#[cfg(any(test, feature = "certification-support"))]
impl std::fmt::Debug for WorthUiPresentationAsyncInstallationDenial {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::AlreadyInstalled(_) => formatter.write_str("AlreadyInstalled(..)"),
        }
    }
}

#[cfg(any(test, feature = "certification-support"))]
impl WorthUiPresentationAsyncInstallationDenial {
    pub(crate) fn into_installation(
        self,
    ) -> worth_ui_query_binding::WorthUiPresentationAsyncInstallation {
        match self {
            Self::AlreadyInstalled(installation) => *installation,
        }
    }
}

impl WorthUiApp {
    #[cfg(any(test, feature = "certification-support"))]
    pub(crate) fn install_presentation_async(
        &mut self,
        installation: worth_ui_query_binding::WorthUiPresentationAsyncInstallation,
    ) -> Result<(), WorthUiPresentationAsyncInstallationDenial> {
        if self.presentation_async.is_some() {
            return Err(
                WorthUiPresentationAsyncInstallationDenial::AlreadyInstalled(Box::new(
                    installation,
                )),
            );
        }
        self.presentation_async = Some(
            crate::native_platform::text_presentation::UiPresentationAsyncRuntime::from_installation(
                installation,
            ),
        );
        Ok(())
    }

    pub(crate) fn install_presentation_async_owner(
        &mut self,
        owner: Option<crate::native_platform::text_presentation::UiPresentationAsyncRuntime>,
    ) {
        self.presentation_async = owner;
    }

    pub(crate) fn take_presentation_async_owner(
        &mut self,
    ) -> Option<crate::native_platform::text_presentation::UiPresentationAsyncRuntime> {
        self.presentation_async.take()
    }
}
