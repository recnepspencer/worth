//! Checked admission and bounded overlap for the existing genuine aggregate reader.
use super::*;
use std::sync::Condvar;
use std::thread::{self, ThreadId};
use std::time::Duration;

#[derive(Default)]
struct Overlap {
    first: Option<ThreadId>,
    second: Option<ThreadId>,
    second_finished: bool,
}

pub(in super::super) struct ParallelReads {
    enabled: bool,
    state: Mutex<Overlap>,
    signal: Condvar,
}

impl ParallelReads {
    pub(in super::super) fn new(workers: usize) -> Arc<Self> {
        Arc::new(Self {
            enabled: workers > 1,
            state: Mutex::new(Overlap::default()),
            signal: Condvar::new(),
        })
    }

    pub(in super::super) fn reset(&self) {
        *self.state.lock().unwrap() = Overlap::default();
    }

    pub(in super::super) fn assert_overlap(&self) {
        if self.enabled {
            let state = self.state.lock().unwrap();
            assert!(state.first.is_some() && state.second.is_some());
            assert_ne!(state.first, state.second, "real separate rule workers");
            assert!(state.second_finished);
        }
    }

    pub(in super::super) fn enter(&self, id: usize) {
        if !self.enabled {
            return;
        }
        let mut state = self.state.lock().unwrap();
        if id == 10 {
            state.first = Some(thread::current().id());
            self.signal.notify_all();
        } else {
            let (mut state, timeout) = self
                .signal
                .wait_timeout_while(state, Duration::from_secs(30), |state| {
                    state.first.is_none()
                })
                .unwrap();
            assert!(!timeout.timed_out(), "first actual reader did not enter");
            state.second = Some(thread::current().id());
        }
    }

    pub(in super::super) fn finish(&self, id: usize) {
        if !self.enabled {
            return;
        }
        let mut state = self.state.lock().unwrap();
        if id == 11 {
            state.second_finished = true;
            self.signal.notify_all();
        } else {
            let (_state, timeout) = self
                .signal
                .wait_timeout_while(state, Duration::from_secs(30), |state| {
                    !state.second_finished
                })
                .unwrap();
            assert!(
                !timeout.timed_out(),
                "second actual component read did not finish"
            );
        }
    }
}

struct CheckedReader {
    reader: ReaderRule,
    overlap: Arc<ParallelReads>,
}

pub(in super::super) fn registration(
    id: usize,
    control: &Arc<Control>,
    overlap: &Arc<ParallelReads>,
) -> CustomInvariantRegistration {
    CustomInvariantRegistration::new(CheckedReader {
        reader: ReaderRule {
            id,
            mode: ReadMode::Both,
            maximum: None,
            control: Arc::clone(control),
            maintenance: false,
        },
        overlap: Arc::clone(overlap),
    })
    .unwrap()
}

impl CustomInvariantRule for CheckedReader {
    type Scope = Vec<AggregateRead>;

    fn descriptor(&self) -> CustomInvariantDescriptor {
        self.reader.descriptor()
    }

    fn prepare_scope(
        &self,
        planner: &mut CustomInvariantScopePlanner<'_>,
    ) -> Result<Self::Scope, CustomInvariantPreparationError> {
        self.reader.prepare_scope(planner)
    }

    fn supports_checked_preparation(&self) -> bool {
        true
    }

    fn prepare_scope_checked(
        &self,
        planner: &mut CustomInvariantScopePlanner<'_>,
        budget: &dyn CustomInvariantLeaseBudget,
    ) -> Option<Result<Self::Scope, CustomInvariantPreparationError>> {
        let count = self.reader.control.roots.lock().unwrap().len() as u64;
        // Native funds each aggregate body. The wrapper additionally admits its
        // retained vector of handles before the existing reader grows it.
        if !budget.checkpoint(count)
            || !budget.claim_result(count * size_of::<AggregateRead>() as u64)
        {
            return Some(Err(CustomInvariantPreparationError::new(
                "reader scope stopped",
            )));
        }
        Some(self.reader.prepare_scope(planner))
    }

    fn evaluate(
        &self,
        context: &CustomInvariantExecutionContext<'_>,
        scope: &Self::Scope,
    ) -> Result<CustomInvariantVerdict, CustomInvariantExecutionError> {
        if !self.reader.control.enabled.load(Ordering::SeqCst) {
            return self.reader.evaluate(context, scope);
        }
        self.overlap.enter(self.reader.id);
        let result = self.reader.evaluate(context, scope);
        self.overlap.finish(self.reader.id);
        result
    }

    fn evaluate_checked(
        &self,
        context: &CustomInvariantExecutionContext<'_>,
        scope: &Self::Scope,
        budget: &dyn CustomInvariantLeaseBudget,
    ) -> Option<Result<CustomInvariantVerdict, CustomInvariantExecutionError>> {
        // The original reader builds root coordinates and a temporary handle
        // vector; its Native reads retain their own original request custody.
        let scratch =
            scope.len() as u64 * (size_of::<EntityId>() + size_of::<AggregateRead>()) as u64;
        if !budget.checkpoint(scope.len() as u64) || !budget.claim_scratch(scratch) {
            return Some(Err(CustomInvariantExecutionError::new(
                "reader evaluation stopped",
            )));
        }
        Some(self.evaluate(context, scope))
    }
}
