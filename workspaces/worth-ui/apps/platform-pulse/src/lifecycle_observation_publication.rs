use std::cell::{RefCell, RefMut};
use std::io::{self, Write};
use std::rc::Rc;

use worth_ui::facade::app::{
    UiMountedFrameOutcome, UiMountedFramePublicationReceipt, WorthUiApplicationPreparationDenial,
    WorthUiMountedFrameExecutionStop, WorthUiNativeSourceRebindDenial,
};
use worth_ui::facade::source::{
    UiSourceRebindAttemptFailure, WorthUiFilesystemWatcherDenial, WorthUiSourcePackageRevision,
};
use worth_ui_platform_pulse::observation_contract::{
    PlatformPulseLaunchConfigurationDenial, PlatformPulseLifecycleObservationCodecDenial,
    PlatformPulseLifecycleObservationEnvelope, PlatformPulseLifecycleObservationProjectionDenial,
    PlatformPulseLifecycleObservationStream,
};

mod focus;
mod intent;
mod query;

const MAXIMUM_EVENTS: usize = 256;
const MAXIMUM_ENCODED_BYTES: usize = 1_048_576;

#[derive(Clone)]
pub(crate) struct PlatformPulseObservationPublisher {
    inner: Rc<RefCell<PlatformPulseObservationPublication>>,
}

struct PlatformPulseObservationPublication {
    stream: PlatformPulseLifecycleObservationStream,
    budget: PlatformPulseObservationPublicationBudget,
}

#[derive(Default)]
struct PlatformPulseObservationPublicationBudget {
    event_count: usize,
    encoded_bytes: usize,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum PlatformPulseObservationPublicationDenial {
    PublisherBusy,
    Projection(PlatformPulseLifecycleObservationProjectionDenial),
    Encoding(PlatformPulseLifecycleObservationCodecDenial),
    EventLimitExceeded,
    EncodedByteLimitExceeded,
    StdoutUnavailable,
}

impl PlatformPulseObservationPublisher {
    pub(crate) fn appearance_preparation_failure(
        &self,
    ) -> Result<(), PlatformPulseObservationPublicationDenial> {
        self.with_publication(|publication| {
            publication.project(|stream| stream.project_appearance_preparation_failure())
        })
    }

    pub(crate) fn start() -> Result<Self, PlatformPulseObservationPublicationDenial> {
        let (stream, started) = PlatformPulseLifecycleObservationStream::start();
        let mut publication = PlatformPulseObservationPublication {
            stream,
            budget: PlatformPulseObservationPublicationBudget::default(),
        };
        publication.publish(started)?;
        Ok(Self {
            inner: Rc::new(RefCell::new(publication)),
        })
    }

    pub(crate) fn launch_configuration_failure(
        &self,
        denial: &PlatformPulseLaunchConfigurationDenial,
    ) -> Result<(), PlatformPulseObservationPublicationDenial> {
        self.with_publication(|publication| {
            publication.project(|stream| stream.project_launch_configuration_failure(denial))
        })
    }

    pub(crate) fn filesystem_watcher_failure(
        &self,
        denial: &WorthUiFilesystemWatcherDenial,
    ) -> Result<(), PlatformPulseObservationPublicationDenial> {
        self.with_publication(|publication| {
            publication.project(|stream| stream.project_filesystem_watcher_failure(denial))
        })
    }

    pub(crate) fn application_preparation_failure(
        &self,
        denial: &WorthUiApplicationPreparationDenial,
    ) -> Result<(), PlatformPulseObservationPublicationDenial> {
        self.with_publication(|publication| {
            publication.project(|stream| stream.project_application_preparation_failure(denial))
        })
    }

    pub(crate) fn candidate_submission_failure(
        &self,
        denial: &UiSourceRebindAttemptFailure,
    ) -> Result<(), PlatformPulseObservationPublicationDenial> {
        self.with_publication(|publication| {
            publication.project(|stream| stream.project_candidate_submission_failure(denial))
        })
    }

    pub(crate) fn frame_execution_failure(
        &self,
        denial: &WorthUiMountedFrameExecutionStop<'_>,
    ) -> Result<(), PlatformPulseObservationPublicationDenial> {
        self.with_publication(|publication| {
            publication.project(|stream| stream.project_frame_execution_failure(denial))
        })
    }

    pub(crate) fn frame_outcome_failure(
        &self,
        outcome: &UiMountedFrameOutcome,
    ) -> Result<(), PlatformPulseObservationPublicationDenial> {
        self.with_publication(|publication| {
            publication.project(|stream| stream.project_frame_outcome_failure(outcome))
        })
    }

    pub(crate) fn native_rebind_failure(
        &self,
        denial: &WorthUiNativeSourceRebindDenial,
    ) -> Result<(), PlatformPulseObservationPublicationDenial> {
        self.with_publication(|publication| {
            publication.project(|stream| stream.project_native_rebind_failure(denial))
        })
    }

    pub(crate) fn native_rebind_outcome_failure(
        &self,
    ) -> Result<(), PlatformPulseObservationPublicationDenial> {
        self.with_publication(|publication| {
            publication.project(
                PlatformPulseLifecycleObservationStream::project_native_rebind_outcome_failure,
            )
        })
    }

    pub(crate) fn first_frame(
        &self,
        source: &WorthUiSourcePackageRevision,
        publication: &UiMountedFramePublicationReceipt,
    ) -> Result<(), PlatformPulseObservationPublicationDenial> {
        self.with_publication(|publisher| {
            publisher.project(|stream| stream.project_first_frame(source, publication))
        })
    }

    pub(crate) fn native_input_reached(
        &self,
        reached: worth_ui_platform_pulse::observation_contract::PlatformPulseNativeInputReached,
    ) -> Result<(), PlatformPulseObservationPublicationDenial> {
        self.with_publication(|publisher| {
            publisher.project(|stream| stream.project_native_input_reached(reached))
        })
    }

    pub(crate) fn replacement(
        &self,
        source: &WorthUiSourcePackageRevision,
        receipt: &worth_ui::facade::rebind::UiRebindReceipt,
        latest_focus_transition: Option<
            worth_ui::facade::inspection::UiFocusMovedInspectionSummary,
        >,
    ) -> Result<(), PlatformPulseObservationPublicationDenial> {
        self.with_publication(|publisher| {
            publisher.project(|stream| {
                stream.project_replacement(source, receipt, latest_focus_transition)
            })
        })
    }

    pub(crate) fn preserved_predecessor(
        &self,
        source: &WorthUiSourcePackageRevision,
        denial: &UiSourceRebindAttemptFailure,
    ) -> Result<(), PlatformPulseObservationPublicationDenial> {
        self.with_publication(|publisher| {
            publisher.project(|stream| stream.project_preserved_predecessor(source, denial))
        })
    }

    pub(crate) fn visual_comparison(
        &self,
        comparison: worth_ui::facade::inspection::UiVisualSnapshotComparison,
    ) -> Result<(), PlatformPulseObservationPublicationDenial> {
        self.with_publication(|publisher| {
            publisher.project(|stream| stream.project_visual_comparison(comparison))
        })
    }

    pub(crate) fn source_worker_panicked(
        &self,
    ) -> Result<(), PlatformPulseObservationPublicationDenial> {
        self.with_publication(|publisher| {
            publisher.project(PlatformPulseLifecycleObservationStream::project_source_worker_panic)
        })
    }

    pub(crate) fn native_event_loop_failure(
        &self,
    ) -> Result<(), PlatformPulseObservationPublicationDenial> {
        self.with_publication(|publisher| {
            publisher
                .project(PlatformPulseLifecycleObservationStream::project_native_event_loop_failure)
        })
    }

    fn with_publication(
        &self,
        publish: impl FnOnce(
            &mut PlatformPulseObservationPublication,
        ) -> Result<(), PlatformPulseObservationPublicationDenial>,
    ) -> Result<(), PlatformPulseObservationPublicationDenial> {
        let mut publication = self.lock()?;
        publish(&mut publication)
    }

    pub(super) fn project_observation(
        &self,
        projection: impl FnOnce(
            &mut PlatformPulseLifecycleObservationStream,
        ) -> Result<
            PlatformPulseLifecycleObservationEnvelope,
            PlatformPulseLifecycleObservationProjectionDenial,
        >,
    ) -> Result<(), PlatformPulseObservationPublicationDenial> {
        self.with_publication(|publication| publication.project(projection))
    }

    fn lock(
        &self,
    ) -> Result<
        RefMut<'_, PlatformPulseObservationPublication>,
        PlatformPulseObservationPublicationDenial,
    > {
        self.inner
            .try_borrow_mut()
            .map_err(|_| PlatformPulseObservationPublicationDenial::PublisherBusy)
    }
}

impl PlatformPulseObservationPublication {
    fn project(
        &mut self,
        projection: impl FnOnce(
            &mut PlatformPulseLifecycleObservationStream,
        ) -> Result<
            PlatformPulseLifecycleObservationEnvelope,
            PlatformPulseLifecycleObservationProjectionDenial,
        >,
    ) -> Result<(), PlatformPulseObservationPublicationDenial> {
        let envelope = projection(&mut self.stream)
            .map_err(PlatformPulseObservationPublicationDenial::Projection)?;
        self.publish(envelope)
    }

    fn publish(
        &mut self,
        envelope: PlatformPulseLifecycleObservationEnvelope,
    ) -> Result<(), PlatformPulseObservationPublicationDenial> {
        let line = envelope
            .encode_prefixed_line()
            .map_err(PlatformPulseObservationPublicationDenial::Encoding)?;
        self.budget.admit(line.len() + 1)?;
        let stdout = io::stdout();
        let mut stdout = stdout.lock();
        writeln!(stdout, "{line}")
            .map_err(|_| PlatformPulseObservationPublicationDenial::StdoutUnavailable)?;
        stdout
            .flush()
            .map_err(|_| PlatformPulseObservationPublicationDenial::StdoutUnavailable)
    }
}

impl PlatformPulseObservationPublicationBudget {
    fn admit(
        &mut self,
        encoded_bytes: usize,
    ) -> Result<(), PlatformPulseObservationPublicationDenial> {
        let event_count = self
            .event_count
            .checked_add(1)
            .ok_or(PlatformPulseObservationPublicationDenial::EventLimitExceeded)?;
        let total_bytes = self
            .encoded_bytes
            .checked_add(encoded_bytes)
            .ok_or(PlatformPulseObservationPublicationDenial::EncodedByteLimitExceeded)?;
        if event_count > MAXIMUM_EVENTS {
            return Err(PlatformPulseObservationPublicationDenial::EventLimitExceeded);
        }
        if total_bytes > MAXIMUM_ENCODED_BYTES {
            return Err(PlatformPulseObservationPublicationDenial::EncodedByteLimitExceeded);
        }
        self.event_count = event_count;
        self.encoded_bytes = total_bytes;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::{
        PlatformPulseObservationPublicationBudget, PlatformPulseObservationPublicationDenial,
        MAXIMUM_ENCODED_BYTES, MAXIMUM_EVENTS,
    };

    #[test]
    fn publication_budget_rejects_the_257th_event_and_over_one_mibibyte() {
        let mut events = PlatformPulseObservationPublicationBudget::default();
        for _ in 0..MAXIMUM_EVENTS {
            events.admit(1).expect("within event budget");
        }
        assert_eq!(
            events.admit(1),
            Err(PlatformPulseObservationPublicationDenial::EventLimitExceeded)
        );

        let mut bytes = PlatformPulseObservationPublicationBudget::default();
        assert_eq!(
            bytes.admit(MAXIMUM_ENCODED_BYTES + 1),
            Err(PlatformPulseObservationPublicationDenial::EncodedByteLimitExceeded)
        );
    }
}
