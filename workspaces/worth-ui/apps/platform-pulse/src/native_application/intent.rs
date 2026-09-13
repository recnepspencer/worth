use super::{PlatformPulseApplicationRuntime, PlatformPulseTerminalError};

mod clock;
mod command_inspection;
mod evidence_index;
mod execution;
mod native_ingress;
mod portal_dismissal;
mod product_action;
mod product_cycle;
mod product_input;

pub(super) use clock::{PlatformPulseIntentClock, PlatformPulseIntentClockDenial};
pub(super) use command_inspection::latest_command_transition;
pub(super) use evidence_index::PlatformPulseIntentEvidenceIndex;
pub(super) use execution::PlatformPulsePendingIntentConsequence;
pub(super) use native_ingress::{
    PlatformPulseIntentPosturePublicationDisposition, PlatformPulseIntentPostureSettlement,
    PlatformPulsePendingIntentPosture, PlatformPulsePendingNativePublication,
    PlatformPulsePreparedIntentPosture,
};
pub(super) use product_cycle::{
    PlatformPulseIntentExecutionProgress, PlatformPulseIntentProductCycleOutcome,
};

pub(super) enum PlatformPulseIntentPosturePublicationDenial {
    Managed(worth_ui::facade::app::WorthUiNativeManagedIntentPosturePublicationDenial),
    Stopped(worth_ui::facade::app::WorthUiNativeManagedRebindStop),
}

impl std::fmt::Display for PlatformPulseIntentPosturePublicationDenial {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Managed(denial) => write!(formatter, "managed admission: {denial:?}"),
            Self::Stopped(stop) => write!(formatter, "stopped: {stop:?}"),
        }
    }
}

impl PlatformPulseApplicationRuntime {
    fn fail_intent_clock(&mut self, denial: PlatformPulseIntentClockDenial) {
        let observation = self.publisher.intent_preparation_failure();
        self.fail(PlatformPulseTerminalError::IntentClock(denial), observation);
    }

    fn fail_intent_settlement(&mut self, detail: impl Into<String>) {
        let observation = self.publisher.intent_preparation_failure();
        self.fail(
            PlatformPulseTerminalError::IntentExecution(detail.into()),
            observation,
        );
    }
}
