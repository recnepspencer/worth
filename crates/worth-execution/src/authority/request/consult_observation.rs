//! Count real request checks; callbacks may only signal external cancellation.
use std::cell::RefCell;
struct Probe {
    count: usize,
    observe: Box<dyn FnMut(usize)>,
}
thread_local! { static PROBE:RefCell<Option<Probe>>=const{RefCell::new(None)}; }
/// Test observation is confined to the thread whose live checks it counts.
pub struct RequestConsultObservation {
    previous: Option<Probe>,
    _thread: std::marker::PhantomData<std::rc::Rc<()>>,
}
impl RequestConsultObservation {
    pub fn begin(observe: impl FnMut(usize) + 'static) -> Self {
        Self {
            previous: PROBE.with(|p| {
                p.replace(Some(Probe {
                    count: 0,
                    observe: Box::new(observe),
                }))
            }),
            _thread: std::marker::PhantomData,
        }
    }
    pub fn count(&self) -> usize {
        PROBE.with(|p| p.borrow().as_ref().unwrap().count)
    }
}
impl Drop for RequestConsultObservation {
    fn drop(&mut self) {
        PROBE.with(|p| *p.borrow_mut() = self.previous.take());
    }
}
pub(crate) fn reached() {
    PROBE.with(|p| {
        if let Some(probe) = p.borrow_mut().as_mut() {
            probe.count = probe.count.checked_add(1).expect("test consult count");
            (probe.observe)(probe.count);
        }
    });
}
/// Observes this thread's live checks and checkpoints, restoring a nested probe
/// on return or unwind. The callback signals cancellation without calling work.
pub fn with_request_consult_observer<R>(
    observe: impl FnMut(usize) + 'static,
    body: impl FnOnce() -> R,
) -> (R, usize) {
    let restore = RequestConsultObservation::begin(observe);
    let result = body();
    let count = PROBE.with(|p| p.borrow().as_ref().unwrap().count);
    drop(restore);
    (result, count)
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn consult_observation_restores_the_outer_count() {
        let (_, count) = with_request_consult_observer(
            |_| {},
            || {
                reached();
                let (_, inner) = with_request_consult_observer(
                    |_| {},
                    || {
                        reached();
                        reached();
                    },
                );
                assert_eq!(inner, 2);
                reached();
            },
        );
        assert_eq!(count, 2);
    }
}
