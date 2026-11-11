//! Passive anchors on this actual fixture receiver; no modeled work counter.
use std::sync::{Arc, Mutex};
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum Stage {
    UpdateBegin,
    UpdateEnd,
    MergeBegin,
    MergeEnd,
    BoundsBegin,
    BoundsEnd,
    LoanBegin,
    LoanEnd,
}
pub(crate) type Observer = Arc<dyn Fn(Stage) + Send + Sync>;
#[derive(Default)]
pub(crate) struct CostStages(Mutex<Option<Observer>>);
impl CostStages {
    pub(crate) fn replace(&self, observer: Option<Observer>) -> Option<Observer> {
        std::mem::replace(&mut *self.0.lock().unwrap(), observer)
    }
    pub(crate) fn record(&self, stage: Stage) {
        let observer = self.0.lock().unwrap().clone();
        if let Some(observer) = observer {
            observer(stage);
        }
    }
}
