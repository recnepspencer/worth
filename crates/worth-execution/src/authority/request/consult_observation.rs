//! Count real request checks; callbacks may only signal external cancellation.
use std::cell::RefCell;
struct Probe {
    scope: Option<crate::backend::ObservationScope>,
    count: usize,
    charges: Vec<(usize, u64)>,
    checkpoint_stops: std::collections::HashMap<usize, usize>,
    stop_origins: Vec<(usize, usize)>,
    observe: Box<dyn FnMut(usize)>,
}
thread_local! { static PROBE:RefCell<Option<Probe>>=const{RefCell::new(None)}; }
/// Test observation is confined to the thread whose live checks it counts.
pub struct RequestConsultObservationForTest {
    previous: Option<Probe>,
    _thread: std::marker::PhantomData<std::rc::Rc<()>>,
}
impl RequestConsultObservationForTest {
    pub fn install_on_this_thread_for_test(observe: impl FnMut(usize) + 'static) -> Self {
        Self::install(None, observe)
    }
    /// Observe only the actual current root and its genuine active descendants.
    /// Nested or noncurrent receivers refuse; prefixes keep their root meaning.
    /// Detached diagnostics are excluded. Weak metadata retains no live meter.
    pub fn install_in_scope_on_this_thread_for_test(
        context: &crate::MapKernelContext<'_, '_>,
        observe: impl FnMut(usize) + 'static,
    ) -> Result<Self, crate::WorkCeilingDenial> {
        let scope = crate::backend::ObservationScope::current(context).ok_or(
            crate::WorkCeilingDenial::Admission(crate::LeaseDenial::NoActiveExecutionScope),
        )?;
        Ok(Self::install(Some(scope), observe))
    }
    fn install(
        scope: Option<crate::backend::ObservationScope>,
        observe: impl FnMut(usize) + 'static,
    ) -> Self {
        Self {
            previous: PROBE.with(|p| {
                p.replace(Some(Probe {
                    scope,
                    count: 0,
                    charges: Vec::new(),
                    checkpoint_stops: Default::default(),
                    stop_origins: Vec::new(),
                    observe: Box::new(observe),
                }))
            }),
            _thread: std::marker::PhantomData,
        }
    }
    /// The cumulative root-scope Work required at each actual charge.
    pub fn charged_prefixes_for_test(&self) -> Vec<(usize, u64)> {
        PROBE.with(|p| p.borrow().as_ref().unwrap().charges.clone())
    }
    /// Emission and origin consults; a latched checkpoint retains its first cause.
    /// Origin zero predates this observation.
    pub fn stop_origins_for_test(&self) -> Vec<(usize, usize)> {
        PROBE.with(|p| p.borrow().as_ref().unwrap().stop_origins.clone())
    }
    /// Passive marker admission; reading saved records needs no active scope.
    pub fn is_current_scope_observed_for_test(&self) -> bool {
        is_installed()
    }
    pub fn count_for_test(&self) -> usize {
        PROBE.with(|p| p.borrow().as_ref().unwrap().count)
    }
}
impl Drop for RequestConsultObservationForTest {
    fn drop(&mut self) {
        PROBE.with(|p| *p.borrow_mut() = self.previous.take());
    }
}
fn is_installed() -> bool {
    PROBE.with(|p| {
        p.borrow().as_ref().is_some_and(|probe| {
            probe
                .scope
                .as_ref()
                .is_none_or(|scope| scope.admits_current())
        })
    })
}
pub(crate) fn reached() {
    if !is_installed() {
        return;
    }
    PROBE.with(|p| {
        if let Some(probe) = p.borrow_mut().as_mut() {
            probe.count = probe.count.checked_add(1).expect("test consult count");
            (probe.observe)(probe.count);
        }
    });
}
/// Observes this thread's live checks and checkpoints, restoring a nested probe
/// on return or unwind. The callback signals cancellation without calling work.
pub fn with_request_consult_observer_for_test<R>(
    observe: impl FnMut(usize) + 'static,
    body: impl FnOnce() -> R,
) -> (R, usize) {
    let restore = RequestConsultObservationForTest::install_on_this_thread_for_test(observe);
    let result = body();
    let count = PROBE.with(|p| p.borrow().as_ref().unwrap().count);
    drop(restore);
    (result, count)
}

pub(crate) fn charged_prefix(required: impl FnOnce() -> u64) {
    if !is_installed() {
        return;
    }
    PROBE.with(|p| {
        if let Some(probe) = p.borrow_mut().as_mut() {
            probe.charges.push((probe.count, required()));
        }
    });
}

pub(crate) fn stopped(checkpoint: Option<(usize, bool)>) {
    if !is_installed() {
        return;
    }
    PROBE.with(|p| {
        if let Some(probe) = p.borrow_mut().as_mut() {
            let origin = match checkpoint {
                Some((identity, true)) => {
                    probe.checkpoint_stops.get(&identity).copied().unwrap_or(0)
                }
                Some((identity, false)) => {
                    probe.checkpoint_stops.insert(identity, probe.count);
                    probe.count
                }
                None => probe.count,
            };
            probe.stop_origins.push((probe.count, origin));
        }
    });
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn latched_stop_keeps_its_origin_and_a_late_observer_reports_unknown() {
        let serial = crate::SerialRequest::from_memory(
            crate::SerialMemoryBudget::new(None),
            crate::CancellationToken::new(),
            None,
        );
        crate::backend::run_scope_within::<(), (), _>(
            None,
            0,
            0,
            true,
            Some(1),
            Some(&serial),
            |work| {
                let observer =
                    RequestConsultObservationForTest::install_on_this_thread_for_test(|_| {});
                assert_eq!(work.checkpoint(2), Err(crate::MapKernelStop::WorkCeiling));
                assert_eq!(work.checkpoint(0), Err(crate::MapKernelStop::WorkCeiling));
                assert_eq!(observer.stop_origins_for_test(), vec![(1, 1), (2, 1)]);
                assert_eq!(observer.charged_prefixes_for_test(), vec![(1, 2)]);
                drop(observer);
                let late =
                    RequestConsultObservationForTest::install_on_this_thread_for_test(|_| {});
                assert_eq!(work.checkpoint(0), Err(crate::MapKernelStop::WorkCeiling));
                assert_eq!(late.stop_origins_for_test(), vec![(1, 0)]);
                Ok(())
            },
        );
    }
    #[test]
    fn consult_observation_restores_the_outer_count() {
        let (_, count) = with_request_consult_observer_for_test(
            |_| {},
            || {
                reached();
                let (_, inner) = with_request_consult_observer_for_test(
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
