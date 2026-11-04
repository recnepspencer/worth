//! Sealed test doors over this module's private members.
use super::*;

impl<'a, Schema: ApplicationSchema + 'static> RefusingMember<'a, Schema> {
    #[doc(hidden)]
    pub fn end_for_test(
        self,
        stop: WorthQueryOutputDemandDenial,
        recording: StopRecording,
        queue: &mut QueuedRequiredRoots,
    ) -> Result<RefusalEnd, WorthQueryOutputDemandDenial> {
        Self::end(self, stop, recording, queue)
    }
}

impl RefusalEnd {
    #[doc(hidden)]
    pub fn finish_for_test<T>(self, finish_cursor: impl FnOnce() -> T) -> Option<T> {
        Self::finish(self, finish_cursor)
    }
}
