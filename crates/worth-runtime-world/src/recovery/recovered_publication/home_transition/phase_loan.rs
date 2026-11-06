//! Restore the current owned phase when a provider or cutover unwinds.
use super::*;

pub(super) struct OriginalContinuationPhaseLoan<'owner> {
    slot: &'owner mut Option<OriginalContinuationProgress>,
    current: Option<OriginalContinuationProgress>,
}
impl<'owner> OriginalContinuationPhaseLoan<'owner> {
    pub(super) fn take(slot: &'owner mut Option<OriginalContinuationProgress>) -> Self {
        let current = slot.take().expect("one original continuation advance");
        Self {
            slot,
            current: Some(current),
        }
    }
    pub(super) fn phase(&mut self) -> &mut OriginalContinuationProgress {
        self.current
            .as_mut()
            .expect("the original phase remains owned")
    }
    /// Only infallible owner field transfers run here, after fallible work.
    pub(super) fn transition(
        &mut self,
        next: impl FnOnce(OriginalContinuationProgress) -> OriginalContinuationProgress,
    ) {
        let current = self
            .current
            .take()
            .expect("the original phase remains owned");
        self.current = Some(next(current));
    }
}
impl Drop for OriginalContinuationPhaseLoan<'_> {
    fn drop(&mut self) {
        *self.slot = Some(
            self.current
                .take()
                .expect("infallible phase transfer completed"),
        );
    }
}
