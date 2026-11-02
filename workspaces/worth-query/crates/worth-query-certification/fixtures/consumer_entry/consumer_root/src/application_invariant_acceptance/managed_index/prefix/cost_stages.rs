//! Passive anchors on this actual fixture receiver; no modeled work counter.
use std::sync::{Arc, Mutex};
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(in super::super) enum Stage {
    UpdateBegin,
    UpdateEnd,
    MergeBegin,
    MergeEnd,
    BoundsBegin,
    BoundsEnd,
    LoanBegin,
    LoanEnd,
}
pub(in super::super) type Observer = Arc<dyn Fn(Stage) + Send + Sync>;
#[derive(Default)]
pub(in super::super) struct CostStages(Mutex<Option<Observer>>);
impl CostStages {
    pub(in super::super) fn replace(&self, observer: Option<Observer>) -> Option<Observer> {
        std::mem::replace(&mut *self.0.lock().unwrap(), observer)
    }
    pub(super) fn record(&self, stage: Stage) {
        let observer = self.0.lock().unwrap().clone();
        if let Some(observer) = observer {
            observer(stage);
        }
    }
}
