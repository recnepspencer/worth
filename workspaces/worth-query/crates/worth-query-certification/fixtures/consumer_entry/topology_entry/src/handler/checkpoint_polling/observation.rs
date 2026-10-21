//! A test-owned request action and observations; no runtime truth is supplied.
use std::cell::RefCell;
use worth_query_host::facade::primary_graph::HandlerInterruption;

type Poll = Result<(), HandlerInterruption>;
struct Observed {
    before: Option<Box<dyn FnOnce()>>,
    after: Option<Box<dyn FnOnce()>>,
    polls: Vec<Poll>,
    candidates: usize,
}
thread_local! {
    static OBSERVED: RefCell<Option<Observed>> = const { RefCell::new(None) };
}

pub(crate) struct PollObservation;
impl PollObservation {
    pub(crate) fn arm(before: impl FnOnce() + 'static, after: impl FnOnce() + 'static) -> Self {
        OBSERVED.with_borrow_mut(|slot| {
            assert!(
                slot.is_none(),
                "one native decision observation owns the thread"
            );
            *slot = Some(Observed {
                before: Some(Box::new(before)),
                after: Some(Box::new(after)),
                polls: Vec::new(),
                candidates: 0,
            });
        });
        Self
    }
    pub(crate) fn polls(&self) -> Vec<Poll> {
        OBSERVED.with_borrow(|slot| slot.as_ref().unwrap().polls.clone())
    }
    pub(crate) fn candidates(&self) -> usize {
        OBSERVED.with_borrow(|slot| slot.as_ref().unwrap().candidates)
    }
}
impl Drop for PollObservation {
    fn drop(&mut self) {
        OBSERVED.with_borrow_mut(Option::take);
    }
}

pub(super) fn before_poll() {
    let action =
        OBSERVED.with_borrow_mut(|slot| slot.as_mut().and_then(|state| state.before.take()));
    if let Some(action) = action {
        action();
    }
}
pub(super) fn record(poll: Poll) {
    let action = OBSERVED.with_borrow_mut(|slot| {
        let state = slot.as_mut()?;
        state.polls.push(poll);
        state.after.take()
    });
    if let Some(action) = action {
        action();
    }
}
pub(crate) fn candidate_started() {
    OBSERVED.with_borrow_mut(|slot| {
        if let Some(state) = slot {
            state.candidates += 1;
        }
    });
}
