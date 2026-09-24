use worth_ui_platform_pulse::observation_contract::PlatformPulseNativeInputIngressPosture;

use crate::lifecycle_observation_publication::{
    PlatformPulseObservationPublicationDenial, PlatformPulseObservationPublisher,
};

/// Native input is observed only once the first frame is presented; which
/// input kinds have been published exists only from then on.
pub(super) enum PlatformPulseNativeInputIngress {
    AwaitingFirstFrame,
    Armed(PlatformPulsePublishedInputKinds),
}

pub(super) struct PlatformPulsePublishedInputKinds {
    pointer: bool,
    keyboard: bool,
}

impl PlatformPulseNativeInputIngress {
    pub(super) fn arm_after_first_frame(&mut self) {
        if matches!(self, Self::AwaitingFirstFrame) {
            *self = Self::Armed(PlatformPulsePublishedInputKinds {
                pointer: false,
                keyboard: false,
            });
        }
    }

    pub(super) fn observe_native(
        &mut self,
        progress: &worth_ui_native_platform::UiNativeApplicationObservationProgress,
        publisher: &PlatformPulseObservationPublisher,
    ) -> Result<(), PlatformPulseObservationPublicationDenial> {
        let Self::Armed(published) = self else {
            return Ok(());
        };
        for (focus, mounted) in progress
            .focus_publications()
            .filter_map(|result| result.as_ref().ok())
        {
            publisher.semantic_focus_published(*focus, mounted)?;
        }
        if progress.event_count() == 0 {
            return Ok(());
        }
        let posture = if progress.retained_batch_count() == 0 {
            PlatformPulseNativeInputIngressPosture::Stopped
        } else {
            PlatformPulseNativeInputIngressPosture::Retained
        };
        let reached = worth_ui_platform_pulse::observation_contract::
            PlatformPulseNativeInputReached::from_counts(
                progress.event_count(),
                progress.pointer_button_events(),
                progress.keyboard_events(),
                progress.text_events(),
                progress.ime_preedit_events(),
                progress.ime_commit_events(),
                progress.ime_cancel_events(),
                posture,
            );
        published.publish_discovered(reached, publisher)
    }
}

impl PlatformPulsePublishedInputKinds {
    fn publish_discovered(
        &mut self,
        reached: worth_ui_platform_pulse::observation_contract::PlatformPulseNativeInputReached,
        publisher: &PlatformPulseObservationPublisher,
    ) -> Result<(), PlatformPulseObservationPublicationDenial> {
        let pointer_discovered = reached.pointer_button_events() > 0 && !self.pointer;
        let keyboard_discovered = reached.keyboard_events() > 0 && !self.keyboard;
        if pointer_discovered || keyboard_discovered {
            publisher.native_input_reached(reached)?;
            self.pointer |= pointer_discovered;
            self.keyboard |= keyboard_discovered;
        }
        Ok(())
    }
}
