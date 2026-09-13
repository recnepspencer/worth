use super::{
    PlatformPulseLifecycleObservation, PlatformPulseLifecycleObservationEnvelope,
    PlatformPulseLifecycleObservationProjectionDenial, PlatformPulseLifecycleObservationStream,
    PlatformPulseSemanticFocusPublished,
};

impl PlatformPulseLifecycleObservationStream {
    pub fn project_semantic_focus_published(
        &mut self,
        receipt: worth_ui::facade::app::UiSemanticFocusPublicationReceipt,
        mounted: &worth_ui::facade::app::UiMountedFramePublicationReceipt,
    ) -> Result<
        PlatformPulseLifecycleObservationEnvelope,
        PlatformPulseLifecycleObservationProjectionDenial,
    > {
        let publication = self.validate_content_publication(mounted)?;
        let observation = PlatformPulseSemanticFocusPublished::from_runtime(receipt)?;
        if observation.frame() != publication.frame().diagnostic_value() {
            return Err(
                PlatformPulseLifecycleObservationProjectionDenial::SemanticFocusPublicationMismatch,
            );
        }
        self.commit_content_publication(publication);
        self.next_envelope(PlatformPulseLifecycleObservation::SemanticFocusPublished(
            observation,
        ))
    }
}
