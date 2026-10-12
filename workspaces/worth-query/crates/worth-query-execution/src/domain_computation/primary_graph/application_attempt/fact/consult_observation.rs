//! Observe only the read being proved, inside its real outer door.
use std::{
    cell::{Cell, RefCell},
    rc::Rc,
};
use worth_query_admission::facade::authenticated_principal::WorthQueryCancellationSource;
#[derive(Clone, Copy, Eq, PartialEq)]
pub(in crate::domain_computation) enum Door {
    Equality,
    Approval,
    Activation,
}
struct State {
    door: Door,
    depth: Cell<usize>,
    count: Cell<usize>,
}
thread_local! { static ACTIVE: RefCell<Option<Rc<State>>> = const { RefCell::new(None) }; }
pub(in crate::domain_computation) struct ReadConsultObservationForTest {
    state: Rc<State>,
    previous: Option<Rc<State>>,
    _listener: worth_execution::ExecutionCheckpointObservationForTest,
}
impl ReadConsultObservationForTest {
    pub(in crate::domain_computation) fn install_on_this_thread_for_test(
        door: Door,
        cancel: Option<(usize, WorthQueryCancellationSource)>,
    ) -> Self {
        let at = cancel.as_ref().map(|(at, _)| *at);
        Self::install_callback_on_this_thread_for_test(door, at, move || {
            if let Some((_, source)) = &cancel {
                source.cancel();
            }
        })
    }
    pub(in crate::domain_computation) fn install_callback_on_this_thread_for_test(
        door: Door,
        at: Option<usize>,
        mut cancel: impl FnMut() + 'static,
    ) -> Self {
        assert!(crate::domain_computation::execution_observation::is_installed());
        let state = Rc::new(State {
            door,
            depth: Cell::new(0),
            count: Cell::new(0),
        });
        let previous = ACTIVE.replace(Some(Rc::clone(&state)));
        let observed = Rc::clone(&state);
        let listener =
            worth_execution::ExecutionCheckpointObservationForTest::install_on_this_thread_for_test(
                move || {
                    if observed.depth.get() > 0 {
                        let next = observed.count.get() + 1;
                        observed.count.set(next);
                        if at == Some(next) {
                            cancel();
                        }
                    }
                },
            );
        Self {
            state,
            previous,
            _listener: listener,
        }
    }
    pub(in crate::domain_computation) fn consults(&self) -> usize {
        self.state.count.get()
    }
}
impl Drop for ReadConsultObservationForTest {
    fn drop(&mut self) {
        ACTIVE.replace(self.previous.take());
    }
}
pub(in crate::domain_computation) struct ReadConsultScopeForTest(Option<Rc<State>>);
impl ReadConsultScopeForTest {
    pub(in crate::domain_computation) fn enter_for_test(door: Door) -> Self {
        let state = ACTIVE.with(|slot| {
            slot.borrow()
                .as_ref()
                .filter(|state| state.door == door)
                .cloned()
        });
        if let Some(state) = &state {
            state.depth.set(state.depth.get() + 1);
        }
        Self(state)
    }
}
impl Drop for ReadConsultScopeForTest {
    fn drop(&mut self) {
        if let Some(state) = &self.0 {
            state.depth.set(state.depth.get() - 1);
        }
    }
}
