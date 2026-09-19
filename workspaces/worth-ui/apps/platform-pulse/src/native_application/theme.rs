use super::{
    PlatformPulseApplicationRuntime, PlatformPulsePendingManagedRebind, PlatformPulseTerminalError,
};
use crate::theme_preference::PlatformPulseThemePreference;
use worth_ui::facade::app::{
    WorthUiNativeApplicationShell, WorthUiNativeManagedRebindProgress,
    WorthUiNativeManagedRebindStop,
};
use worth_ui::facade::rebind::{
    UiRebindExecutionPolicy, UiRebindExecutionRequest, UiRebindReceipt,
};

#[derive(Debug)]
pub(super) enum PlatformPulseThemeSwitchDenial {
    LayoutUnavailable,
    BindingUnavailable,
    Capability(worth_ui::facade::appearance::UiThemeCapabilityReceiptDenial),
    Origin(worth_ui::facade::appearance::UiProgrammaticThemeSwitchPreparationDenial),
    Publication(worth_ui::facade::appearance::UiNativeThemeSwitchDenial),
    Stopped(WorthUiNativeManagedRebindStop),
    PublicationMismatch,
    UnexpectedProgress,
    TickExhausted,
}

impl PlatformPulseApplicationRuntime {
    pub(super) fn poll_theme_preference(&mut self) {
        let Some(watch) = &mut self.theme_watch else {
            return;
        };
        let preference = match watch.take_changed() {
            Ok(Some(preference)) => preference,
            Ok(None) => return,
            Err(denial @ (crate::theme_preference::PlatformPulseThemePreferenceDenial::WatchUnavailable
                | crate::theme_preference::PlatformPulseThemePreferenceDenial::ReadUnavailable
                | crate::theme_preference::PlatformPulseThemePreferenceDenial::WorkerPanicked)) => {
                self.fail(PlatformPulseTerminalError::ThemePreference(denial), self.publisher.appearance_preparation_failure());
                return;
            }
            Err(denial) => {
                // Invalid preference edits leave the accepted theme and its pixels intact.
                eprintln!("Pulse theme preference rejected: {denial:?}");
                return;
            }
        };
        let mut shell = self.take_runtime_shell();
        self.begin_theme_preference(&mut shell, preference);
        self.shell = Some(shell);
    }

    pub(super) fn begin_theme_preference(
        &mut self,
        shell: &mut WorthUiNativeApplicationShell,
        preference: PlatformPulseThemePreference,
    ) {
        let result = (|| {
            self.presentation_tick = self
                .presentation_tick
                .checked_add(1)
                .ok_or(PlatformPulseThemeSwitchDenial::TickExhausted)?;
            let surface = shell
                .native_layout_basis()
                .map_err(|_| PlatformPulseThemeSwitchDenial::LayoutUnavailable)?
                .surface();
            let predecessor = shell
                .active_theme_binding(surface)
                .ok_or(PlatformPulseThemeSwitchDenial::BindingUnavailable)?
                .binding_generation();
            let capability = shell
                .admit_appearance_theme(surface, &preference.theme.definition())
                .map_err(PlatformPulseThemeSwitchDenial::Capability)?;
            let request = shell
                .prepare_programmatic_theme_switch(surface, predecessor, capability)
                .map_err(PlatformPulseThemeSwitchDenial::Origin)?;
            shell
                .begin_managed_theme_switch(
                    request,
                    UiRebindExecutionPolicy::ordinary(),
                    UiRebindExecutionRequest::new(self.presentation_tick),
                )
                .map_err(PlatformPulseThemeSwitchDenial::Publication)
        })();
        match result {
            Ok(progress) => self.settle_theme_progress(shell, preference, progress),
            Err(denial) => self.fail_theme_switch(denial),
        }
    }

    pub(super) fn settle_theme_progress(
        &mut self,
        shell: &mut WorthUiNativeApplicationShell,
        preference: PlatformPulseThemePreference,
        progress: WorthUiNativeManagedRebindProgress,
    ) {
        match progress {
            WorthUiNativeManagedRebindProgress::Published(receipt) => {
                self.settle_theme_publication(shell, preference, Some(receipt))
            }
            WorthUiNativeManagedRebindProgress::Stopped(
                WorthUiNativeManagedRebindStop::ObservedNoChange,
            ) => self.settle_theme_publication(shell, preference, None),
            WorthUiNativeManagedRebindProgress::AwaitingProgress
            | WorthUiNativeManagedRebindProgress::RecoveryBlocked(_) => {
                self.pending_managed_rebind =
                    Some(PlatformPulsePendingManagedRebind::ThemeSwitch(preference));
            }
            WorthUiNativeManagedRebindProgress::RebindRecovered(_) => {
                self.begin_theme_preference(shell, preference)
            }
            WorthUiNativeManagedRebindProgress::Stopped(stop) => {
                self.fail_theme_switch(PlatformPulseThemeSwitchDenial::Stopped(stop))
            }
            _ => self.fail_theme_switch(PlatformPulseThemeSwitchDenial::UnexpectedProgress),
        }
    }

    fn settle_theme_publication(
        &mut self,
        shell: &mut WorthUiNativeApplicationShell,
        preference: PlatformPulseThemePreference,
        receipt: Option<UiRebindReceipt>,
    ) {
        let result = (|| {
            let surface = shell
                .native_layout_basis()
                .map_err(|_| PlatformPulseThemeSwitchDenial::LayoutUnavailable)?
                .surface();
            let capability = shell
                .admit_appearance_theme(surface, &preference.theme.definition())
                .map_err(PlatformPulseThemeSwitchDenial::Capability)?;
            let binding = shell
                .active_theme_binding(surface)
                .ok_or(PlatformPulseThemeSwitchDenial::BindingUnavailable)?;
            if binding.capability() != &capability
                || receipt
                    .as_ref()
                    .is_some_and(|receipt| receipt.mounted_publication().is_none())
            {
                return Err(PlatformPulseThemeSwitchDenial::PublicationMismatch);
            }
            Ok(binding.binding_generation())
        })();
        let generation = match result {
            Ok(generation) => generation,
            Err(denial) => {
                self.fail_theme_switch(denial);
                return;
            }
        };
        if let Err(error) = self.publisher.theme_switch_settled(
            preference.revision,
            &preference.theme.definition(),
            generation,
            receipt
                .as_ref()
                .and_then(UiRebindReceipt::mounted_publication),
        ) {
            self.fail(
                PlatformPulseTerminalError::ObservationPublication,
                Err(error),
            );
            return;
        }
        if receipt.is_some() {
            if let Err(denial) = self.visual_identity.refresh_after_presentation_replacement(
                shell,
                self.presentation_tick,
                std::time::Instant::now(),
            ) {
                self.fail_visual_identity(denial);
            }
        }
    }

    fn fail_theme_switch(&mut self, denial: PlatformPulseThemeSwitchDenial) {
        self.fail(
            PlatformPulseTerminalError::ThemeSwitch(denial),
            self.publisher.appearance_preparation_failure(),
        );
    }
}

impl std::fmt::Display for PlatformPulseThemeSwitchDenial {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Capability(denial) => write!(formatter, "capability: {denial:?}"),
            Self::Origin(denial) => write!(formatter, "origin: {denial:?}"),
            Self::Publication(denial) => write!(formatter, "publication: {denial:?}"),
            Self::Stopped(stop) => write!(formatter, "managed stop: {stop:?}"),
            other => write!(formatter, "{other:?}"),
        }
    }
}
