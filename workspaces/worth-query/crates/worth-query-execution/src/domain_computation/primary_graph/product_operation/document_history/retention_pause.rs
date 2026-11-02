//! Rendezvous on the actual retention receiver, outside every authority guard.
use super::{receiver_pause::ReceiverPauseGuard, WorthQueryDocumentHistory};
use std::time::Duration;
use worth_query_declaration::facade::{
    application_history::ApplicationHistoryBinding, application_schema::ApplicationSchema,
};

#[derive(Clone, Copy, Eq, PartialEq)]
pub(in crate::domain_computation::primary_graph) enum RetentionChangePoint {
    Before,
    After,
}

#[doc(hidden)]
pub struct WorthQueryHistoryRetentionChangePauseForTest {
    guard: ReceiverPauseGuard,
}
impl<S: ApplicationSchema, H: ApplicationHistoryBinding<S>> WorthQueryDocumentHistory<'_, S, H> {
    #[doc(hidden)]
    pub fn pause_before_history_retention_change_for_test(
        &mut self,
    ) -> WorthQueryHistoryRetentionChangePauseForTest {
        self.arm_retention_pause(RetentionChangePoint::Before)
    }
    #[doc(hidden)]
    pub fn pause_after_history_retention_change_for_test(
        &mut self,
    ) -> WorthQueryHistoryRetentionChangePauseForTest {
        self.arm_retention_pause(RetentionChangePoint::After)
    }
    fn arm_retention_pause(
        &mut self,
        point: RetentionChangePoint,
    ) -> WorthQueryHistoryRetentionChangePauseForTest {
        let mut receiver = self.retention_pause.take().map(|(_, receiver)| receiver);
        let guard = ReceiverPauseGuard::arm(&mut receiver);
        self.retention_pause = Some((point, receiver.expect("the actual receiver was armed")));
        WorthQueryHistoryRetentionChangePauseForTest { guard }
    }
    pub(super) fn pause_retention(&self, point: RetentionChangePoint) {
        if let Some((actual, receiver)) = &self.retention_pause {
            if *actual == point {
                receiver.pause();
            }
        }
    }
}
impl WorthQueryHistoryRetentionChangePauseForTest {
    pub fn wait_until_reached(&self, timeout: Duration) -> bool {
        self.guard.wait_until_reached(timeout)
    }
    pub fn release(&self) {
        self.guard.release();
    }
}
