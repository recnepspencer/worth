use std::time::Duration;

use crate::adjudication::ExecutableLifecycleCleanupEvidence;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) struct PlatformPulseJourneyCost {
    first_publication: Duration,
    full_journey: Duration,
    lifecycle_events: usize,
    lifecycle_bytes: usize,
    source_actions: u32,
    native_captures: u32,
    window_lookups: u32,
    process_launches: u32,
    native_windows: u32,
    close_requests: u32,
    successful_exit: bool,
    installation_removed: bool,
}

pub(super) struct JourneyCostInputs {
    pub(super) first_publication: Duration,
    pub(super) full_journey: Duration,
    pub(super) source_actions: u32,
    pub(super) native_captures: u32,
    pub(super) window_lookups: u32,
}

impl PlatformPulseJourneyCost {
    pub(super) fn from_completed(
        inputs: JourneyCostInputs,
        cleanup: &ExecutableLifecycleCleanupEvidence,
    ) -> Self {
        let lifecycle = cleanup.lifecycle_measurement();
        Self {
            first_publication: inputs.first_publication,
            full_journey: inputs.full_journey,
            lifecycle_events: lifecycle.accepted_events(),
            lifecycle_bytes: lifecycle.accepted_bytes(),
            source_actions: inputs.source_actions,
            native_captures: inputs.native_captures,
            window_lookups: inputs.window_lookups,
            process_launches: 1,
            native_windows: 1,
            close_requests: cleanup.close_request_count(),
            successful_exit: cleanup.successful_exit().status().success(),
            installation_removed: cleanup.installation_removed(),
        }
    }

    pub(super) fn report(self) {
        eprintln!(
            "WORTH_UI_EXECUTABLE_WORLD_COST first_publication_ms={} journey_ms={} \
             lifecycle_events={} lifecycle_bytes={} source_actions={} native_captures={} \
             window_lookups={} process_launches={} native_windows={} close_requests={} \
             successful_exit={} installation_removed={}",
            self.first_publication.as_millis(),
            self.full_journey.as_millis(),
            self.lifecycle_events,
            self.lifecycle_bytes,
            self.source_actions,
            self.native_captures,
            self.window_lookups,
            self.process_launches,
            self.native_windows,
            self.close_requests,
            self.successful_exit,
            self.installation_removed,
        );
    }

    pub(super) const fn full_journey(self) -> Duration {
        self.full_journey
    }
}
