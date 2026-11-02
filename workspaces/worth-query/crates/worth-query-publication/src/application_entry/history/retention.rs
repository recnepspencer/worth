//! Finite availability changes keep their unfinished frontier in the document.
use super::*;
use worth_query_declaration::facade::{
    application_capability::{ApplicationCapabilityRef, ApplicationCapabilityRequest},
    application_history::{
        ApplicationHistoryBookmarkBinding, ApplicationHistoryModel, ApplicationHistoryRecentSteps,
        ApplicationHistoryWindowBinding,
    },
    application_schema::{ApplicationOperationMarkerIdentity, ApplicationOperationRef},
};
mod bookmark;
mod control;
mod maintenance;
mod read;
mod window;

#[must_use = "advance the change or resume its registered target through the same document"]
pub struct WorthQueryHistoryWindowChange<'doc, 'runtime, S, H, Payload, Outputs>
where
    S: ApplicationSchema,
    H: ApplicationHistoryBinding<S>,
    Payload: WorthQueryHistoryPayloadMode<S, H>,
{
    document: &'doc mut WorthQueryApplicationHistoryDocument<'runtime, S, H, Payload, Outputs>,
    target: ApplicationHistoryRecentSteps,
}

impl<'runtime, S, H, Payload, Outputs>
    WorthQueryApplicationHistoryDocument<'runtime, S, H, Payload, Outputs>
where
    S: ApplicationSchema,
    H: ApplicationHistoryBinding<S>,
    Payload: WorthQueryHistoryPayloadMode<S, H>,
{
    pub fn set_history_window(
        &mut self,
        target: ApplicationHistoryRecentSteps,
    ) -> WorthQueryHistoryWindowChange<'_, 'runtime, S, H, Payload, Outputs> {
        WorthQueryHistoryWindowChange {
            document: self,
            target,
        }
    }
}
