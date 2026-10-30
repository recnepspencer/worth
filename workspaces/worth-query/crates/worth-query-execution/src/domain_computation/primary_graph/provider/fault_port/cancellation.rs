//! One pending actual cancellation per named certification boundary.
use std::sync::Mutex;
type Source = worth_query_admission::facade::authenticated_principal::WorthQueryCancellationSource;
#[derive(Default)]
pub(super) struct BoundaryCancellation {
    pending: Mutex<Option<Source>>,
}
impl BoundaryCancellation {
    pub(super) fn arm(&self, source: Source) -> bool {
        let mut pending = self.pending.lock().unwrap_or_else(|p| p.into_inner());
        if pending.is_some() {
            return false;
        }
        *pending = Some(source);
        true
    }
    pub(super) fn fire(&self) {
        let source = self
            .pending
            .lock()
            .unwrap_or_else(|p| p.into_inner())
            .take();
        if let Some(source) = source {
            source.cancel();
        }
    }
}
