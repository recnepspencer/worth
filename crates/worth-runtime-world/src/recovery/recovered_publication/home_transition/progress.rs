//! Report the retained owner's actual progress without granting advancement.
use super::*;
use crate::publication::WorldHomeTransitionEffectProgress as Progress;

impl OriginalHomeTransitionContinuation {
    pub(crate) fn progress(&self) -> Progress {
        match self
            .progress
            .as_ref()
            .expect("a consuming advance loans its phase")
        {
            OriginalContinuationProgress::Prepared(_) => Progress::ReadyToSubmit,
            OriginalContinuationProgress::Submitted(_) => Progress::AwaitingAcknowledgement,
            OriginalContinuationProgress::Acknowledged(_)
            | OriginalContinuationProgress::Materialized(_)
            | OriginalContinuationProgress::HistoryInstalled(_)
            | OriginalContinuationProgress::Transferred(_)
            | OriginalContinuationProgress::Installed(_)
            | OriginalContinuationProgress::Completed(_) => Progress::Acknowledged,
            OriginalContinuationProgress::ReopenRequired(_) => Progress::ReopenRequired,
            OriginalContinuationProgress::Delivered => {
                unreachable!("delivery removes the original row")
            }
        }
    }
}
