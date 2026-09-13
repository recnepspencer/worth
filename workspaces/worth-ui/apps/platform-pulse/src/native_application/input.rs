use worth_ui_platform_pulse::observation_contract::PlatformPulseNativeInputIngressPosture;

use crate::lifecycle_observation_publication::{
    PlatformPulseObservationPublicationDenial, PlatformPulseObservationPublisher,
};

#[derive(Default)]
pub(super) struct PlatformPulseNativeInputIngress {
    armed: bool,
    pointer_published: bool,
    keyboard_published: bool,
}

impl PlatformPulseNativeInputIngress {
    pub(super) fn arm_after_first_frame(&mut self) {
        self.armed = true;
    }

    fn publish_discovered(
        &mut self,
        reached: worth_ui_platform_pulse::observation_contract::PlatformPulseNativeInputReached,
        publisher: &PlatformPulseObservationPublisher,
    ) -> Result<(), PlatformPulseObservationPublicationDenial> {
        let pointer_discovered = reached.pointer_button_events() > 0 && !self.pointer_published;
        let keyboard_discovered = reached.keyboard_events() > 0 && !self.keyboard_published;
        if pointer_discovered || keyboard_discovered {
            publisher.native_input_reached(reached)?;
            self.pointer_published |= pointer_discovered;
            self.keyboard_published |= keyboard_discovered;
        }
        Ok(())
    }

    pub(super) fn observe_native(
        &mut self,
        progress: &worth_ui_native_platform::UiNativeApplicationObservationProgress,
        publisher: &PlatformPulseObservationPublisher,
    ) -> Result<(), PlatformPulseObservationPublicationDenial> {
        if !self.armed {
            return Ok(());
        }
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
        self.publish_discovered(reached, publisher)
    }
}
