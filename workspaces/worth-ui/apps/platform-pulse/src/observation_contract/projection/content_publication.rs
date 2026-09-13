use worth_ui::facade::app::UiMountedFramePublicationReceipt;

use super::{
    PlatformPulseLifecycleObservationProjectionDenial, PlatformPulseLifecycleObservationStream,
    PlatformPulseObservationState,
};
use crate::observation_contract::PlatformPulseMountedFrameObservation;

pub(in crate::observation_contract) struct PlatformPulseValidatedContentPublication {
    frame: PlatformPulseMountedFrameObservation,
}

impl PlatformPulseLifecycleObservationStream {
    /// Advances visual currentness from an accepted ordinary content frame.
    /// The frame is not a new source or Query publication event.
    pub fn observe_mounted_content_publication(
        &mut self,
        publication: &UiMountedFramePublicationReceipt,
    ) -> Result<(), PlatformPulseLifecycleObservationProjectionDenial> {
        let publication = self.validate_content_publication(publication)?;
        let visual = self
            .visual_state
            .after_content_publication(publication.frame().diagnostic_value())?;
        self.commit_content_publication(publication);
        self.visual_state = visual;
        Ok(())
    }

    pub(in crate::observation_contract) fn validate_content_publication(
        &self,
        publication: &UiMountedFramePublicationReceipt,
    ) -> Result<
        PlatformPulseValidatedContentPublication,
        PlatformPulseLifecycleObservationProjectionDenial,
    > {
        match &self.state {
            PlatformPulseObservationState::Published { generation, .. }
                if publication.generation() == generation =>
            {
                Ok(PlatformPulseValidatedContentPublication {
                    frame: PlatformPulseMountedFrameObservation::from_publication(publication),
                })
            }
            PlatformPulseObservationState::Published { .. } => {
                Err(PlatformPulseLifecycleObservationProjectionDenial::ActiveGenerationMismatch)
            }
            PlatformPulseObservationState::Started => Err(
                PlatformPulseLifecycleObservationProjectionDenial::PublishedPredecessorUnavailable,
            ),
            PlatformPulseObservationState::Terminal => {
                Err(PlatformPulseLifecycleObservationProjectionDenial::StreamTerminated)
            }
        }
    }

    pub(in crate::observation_contract) fn commit_content_publication(
        &mut self,
        publication: PlatformPulseValidatedContentPublication,
    ) {
        let PlatformPulseObservationState::Published { frame, .. } = &mut self.state else {
            unreachable!("validated content publication preserves the published stream state")
        };
        *frame = publication.frame;
    }
}

impl PlatformPulseValidatedContentPublication {
    pub(in crate::observation_contract) const fn frame(
        &self,
    ) -> PlatformPulseMountedFrameObservation {
        self.frame
    }
}
