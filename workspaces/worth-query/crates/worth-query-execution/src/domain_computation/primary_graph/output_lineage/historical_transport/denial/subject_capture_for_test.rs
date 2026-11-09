//! Owned read-only capture observation, cleared on ordinary return or unwinding.
use std::{
    cell::RefCell,
    sync::{Arc, Mutex, Weak},
};
type Probe = Box<dyn Fn() -> bool + Send + Sync>;
type State = Mutex<Option<Probe>>;
thread_local! { static ACTIVE: RefCell<Option<Weak<State>>> = const { RefCell::new(None) }; }
#[doc(hidden)]
pub struct Observation {
    state: Arc<State>,
    _thread: std::marker::PhantomData<std::rc::Rc<()>>,
}
impl Observation {
    pub(super) fn install() -> Self {
        let state = Arc::new(Mutex::new(None));
        ACTIVE.with(|active| {
            let mut active = active.borrow_mut();
            assert!(
                active.as_ref().and_then(Weak::upgrade).is_none(),
                "one capture observation per thread"
            );
            *active = Some(Arc::downgrade(&state));
        });
        Self {
            state,
            _thread: std::marker::PhantomData,
        }
    }
    pub fn captured_for_test(&self) -> bool {
        self.state
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .is_some()
    }
    pub fn basis_alive_for_test(&self) -> bool {
        self.state
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .as_ref()
            .is_some_and(|probe| probe())
    }
}
impl Drop for Observation {
    fn drop(&mut self) {
        ACTIVE.with(|active| {
            let mut active = active.borrow_mut();
            if active
                .as_ref()
                .and_then(Weak::upgrade)
                .is_some_and(|state| Arc::ptr_eq(&state, &self.state))
            {
                *active = None;
            }
        });
    }
}
pub(super) fn record(
    publication: &worth_relational::facade::mvcc::RelationalRequiredOutputPublication,
) {
    ACTIVE.with(|active| {
        if let Some(state) = active.borrow().as_ref().and_then(Weak::upgrade) {
            *state
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner) = publication
                .observe_readmitted_basis_for_test()
                .map(|probe| Box::new(probe) as Probe);
        }
    });
}
