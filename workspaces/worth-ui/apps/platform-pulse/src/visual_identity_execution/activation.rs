//! Whether this process runs the visual identity journey at all.
//!
//! The journey is chosen once from the process environment. A disabled Pulse
//! still owns exactly one first frame, which retires it; it never captures,
//! schedules readiness, or retains a rebind receipt.

use std::time::Instant;

use worth_ui::facade::app::WorthUiNativeApplicationShell;

use super::{PlatformPulseVisualExecutionDenial, PlatformPulseVisualIdentityJourney};
use crate::lifecycle_observation_publication::PlatformPulseObservationPublisher;

pub(crate) enum PlatformPulseVisualIdentityExecution {
    Disabled(PlatformPulseDisabledVisualIdentity),
    Enabled(PlatformPulseVisualIdentityJourney),
}

pub(crate) enum PlatformPulseDisabledVisualIdentity {
    AwaitingFirstFrame,
    Retired,
}

impl PlatformPulseVisualIdentityExecution {
    pub(crate) fn new() -> Self {
        if std::env::var_os("WORTH_UI_VISUAL_IDENTITY_JOURNEY").is_some_and(|value| value == "1") {
            Self::Enabled(PlatformPulseVisualIdentityJourney::new())
        } else {
            Self::Disabled(PlatformPulseDisabledVisualIdentity::AwaitingFirstFrame)
        }
    }

    pub(crate) fn install_readiness(
        &mut self,
        signal: worth_ui_platform_pulse::PlatformPulseApplicationReadinessSignal,
    ) {
        if let Self::Enabled(journey) = self {
            journey.install_readiness(signal);
        }
    }

    pub(crate) fn retains_rebind_receipt(&self) -> bool {
        match self {
            Self::Disabled(_) => false,
            Self::Enabled(journey) => journey.retains_rebind_receipt(),
        }
    }

    pub(crate) fn shutdown_quiescent(
        &mut self,
        shell: &mut WorthUiNativeApplicationShell,
    ) -> Result<(), PlatformPulseVisualExecutionDenial> {
        match self {
            Self::Disabled(disabled) => {
                *disabled = PlatformPulseDisabledVisualIdentity::Retired;
                Ok(())
            }
            Self::Enabled(journey) => journey.shutdown_quiescent(shell),
        }
    }

    pub(crate) fn arm_after_first_frame(
        &mut self,
        now: Instant,
    ) -> Result<(), PlatformPulseVisualExecutionDenial> {
        match self {
            Self::Disabled(PlatformPulseDisabledVisualIdentity::Retired) => {
                Err(PlatformPulseVisualExecutionDenial::InitialFrameAlreadyArmed)
            }
            Self::Disabled(disabled) => {
                *disabled = PlatformPulseDisabledVisualIdentity::Retired;
                Ok(())
            }
            Self::Enabled(journey) => journey.arm_after_first_frame(now),
        }
    }

    pub(crate) fn advance(
        &mut self,
        shell: &mut WorthUiNativeApplicationShell,
        publisher: &PlatformPulseObservationPublisher,
        tick: &mut u64,
        now: Instant,
    ) -> Result<(), PlatformPulseVisualExecutionDenial> {
        match self {
            Self::Disabled(_) => Ok(()),
            Self::Enabled(journey) => journey.advance(shell, publisher, tick, now),
        }
    }

    pub(crate) fn compare_after_rebind(
        &mut self,
        shell: &mut WorthUiNativeApplicationShell,
        rebind: worth_ui::facade::rebind::UiRebindReceipt,
        tick: u64,
        now: Instant,
    ) -> Result<(), PlatformPulseVisualExecutionDenial> {
        match self {
            Self::Disabled(_) => Ok(()),
            Self::Enabled(journey) => journey.compare_after_rebind(shell, rebind, tick, now),
        }
    }

    pub(crate) fn prepare_source_rebind(
        &mut self,
        shell: &mut WorthUiNativeApplicationShell,
        tick: u64,
        now: Instant,
    ) -> Result<bool, PlatformPulseVisualExecutionDenial> {
        match self {
            Self::Disabled(_) => Ok(true),
            Self::Enabled(journey) => journey.prepare_source_rebind(shell, tick, now),
        }
    }

    pub(crate) fn refresh_after_presentation_replacement(
        &mut self,
        shell: &mut WorthUiNativeApplicationShell,
        tick: u64,
        now: Instant,
    ) -> Result<(), PlatformPulseVisualExecutionDenial> {
        match self {
            Self::Disabled(_) => Ok(()),
            Self::Enabled(journey) => {
                journey.refresh_after_presentation_replacement(shell, tick, now)
            }
        }
    }
}
