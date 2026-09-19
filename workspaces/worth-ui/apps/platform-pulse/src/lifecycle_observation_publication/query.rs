use worth_ui::facade::app::{
    UiMountedFramePublicationReceipt, WorthUiNativeApplicationShutdownReceipt,
};
use worth_ui::facade::source::WorthUiFilesystemWatcherShutdownReceipt;
use worth_ui_platform_pulse::observation_contract::{
    PlatformPulseIntentWatcherShutdownEvidence, PlatformPulseLifecycleObservationStream,
    PlatformPulseQueryProjectionEvidence, PlatformPulseQueryShutdownEvidence,
    PlatformPulseQueryWatcherShutdownEvidence,
};

use super::{PlatformPulseObservationPublicationDenial, PlatformPulseObservationPublisher};

impl PlatformPulseObservationPublisher {
    pub(crate) fn query_projection_issued(
        &self,
        observation: &worth_ui::facade::query_binding::UiProjectionObservation,
    ) -> Result<PlatformPulseQueryProjectionEvidence, PlatformPulseObservationPublicationDenial>
    {
        let evidence = PlatformPulseQueryProjectionEvidence::from_observation(observation)
            .map_err(PlatformPulseObservationPublicationDenial::Projection)?;
        self.with_publication(|publication| {
            publication.project(|stream| stream.project_query_projection_issued(&evidence))
        })?;
        Ok(evidence)
    }

    pub(crate) fn query_projection_published(
        &self,
        evidence: &PlatformPulseQueryProjectionEvidence,
        publication: &UiMountedFramePublicationReceipt,
    ) -> Result<(), PlatformPulseObservationPublicationDenial> {
        self.with_publication(|publisher| {
            publisher
                .project(|stream| stream.project_query_projection_published(evidence, publication))
        })
    }

    pub(crate) fn query_preparation_failure(
        &self,
    ) -> Result<(), PlatformPulseObservationPublicationDenial> {
        self.with_publication(|publication| {
            publication
                .project(PlatformPulseLifecycleObservationStream::project_query_preparation_failure)
        })
    }

    pub(crate) fn query_shutdown_failure(
        &self,
    ) -> Result<(), PlatformPulseObservationPublicationDenial> {
        self.with_publication(|publisher| {
            publisher
                .project(PlatformPulseLifecycleObservationStream::project_query_shutdown_failure)
        })
    }

    pub(crate) fn shutdown(
        &self,
        watcher: &WorthUiFilesystemWatcherShutdownReceipt,
        query: crate::query_source::PlatformPulseQueryShutdownReceipt,
        query_watcher: crate::query_source::PlatformPulseExternalValueWatchShutdownReceipt,
        intent_watcher: worth_ui_platform_pulse::intent::PlatformPulseIntentInputWatchShutdownReceipt,
        theme_watch_released: bool,
        application: &WorthUiNativeApplicationShutdownReceipt,
    ) -> Result<(), PlatformPulseObservationPublicationDenial> {
        let query = PlatformPulseQueryShutdownEvidence::new(
            PlatformPulseQueryWatcherShutdownEvidence::new(
                query_watcher.worker_joined(),
                query_watcher.pending_event_count() as u64,
            ),
            query.owner_terminal(),
        );
        let intent = PlatformPulseIntentWatcherShutdownEvidence::new(
            intent_watcher.worker_joined(),
            intent_watcher.pending_event_count() as u64,
        );
        self.with_publication(|publisher| {
            publisher.project(|stream| {
                stream.project_shutdown(watcher, query, intent, theme_watch_released, application)
            })
        })
    }
}
