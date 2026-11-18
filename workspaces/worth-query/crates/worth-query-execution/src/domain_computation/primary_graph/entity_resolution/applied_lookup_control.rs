//! Receiver-armed time progress at one named field's real applied entity lookup.
use super::*;
use std::{
    cell::{Cell, RefCell},
    rc::Rc,
};
struct Pending {
    runtime: crate::domain_computation::execution_runtime::WorthQueryRuntimeAuthorityIdentity,
    field: &'static str,
    action: interruption_control::Action,
    triggered: Rc<Cell<bool>>,
}
thread_local! { static PENDING: RefCell<Option<Pending>> = const { RefCell::new(None) }; }
pub struct AppliedLookupTimeControlForTest {
    triggered: Rc<Cell<bool>>,
}
impl WorthQueryInstalledEntityResolutionContext {
    #[doc(hidden)]
    pub fn advance_before_applied_lookup_for_test(
        &self,
        field: &'static str,
        time: worth_execution::MonotonicTimeController,
        advance: std::time::Duration,
    ) -> AppliedLookupTimeControlForTest {
        let deadline = time
            .clock()
            .now()
            .checked_add(advance)
            .expect("fixture time fits");
        let triggered = Rc::new(Cell::new(false));
        PENDING.with(|pending| {
            let mut pending = pending.borrow_mut();
            assert!(pending.is_none(), "one applied time control per thread");
            *pending = Some(Pending {
                runtime: self.runtime_authority,
                field,
                action: interruption_control::Action::Deadline { time, deadline },
                triggered: Rc::clone(&triggered),
            });
        });
        AppliedLookupTimeControlForTest { triggered }
    }
}
impl AppliedLookupTimeControlForTest {
    pub fn was_triggered_for_test(&self) -> bool {
        self.triggered.get()
    }
}
impl Drop for AppliedLookupTimeControlForTest {
    fn drop(&mut self) {
        PENDING.with(|pending| {
            let mut pending = pending.borrow_mut();
            if pending
                .as_ref()
                .is_some_and(|p| Rc::ptr_eq(&p.triggered, &self.triggered))
            {
                *pending = None;
            }
        });
    }
}
/// Applied lookups of other fields pass without spending the armed control.
pub(super) fn trigger(installed: &WorthQueryInstalledEntityResolutionContext, field: &str) {
    let pending = PENDING.with(|pending| {
        let mut pending = pending.borrow_mut();
        if pending
            .as_ref()
            .is_some_and(|p| p.runtime == installed.runtime_authority && p.field == field)
        {
            pending.take()
        } else {
            None
        }
    });
    if let Some(pending) = pending {
        pending.triggered.set(true);
        pending.action.apply();
    }
}
