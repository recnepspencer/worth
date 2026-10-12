//! Dispatch inherits a runtime owner's refusal context, never its capabilities.
use std::{cell::Cell, marker::PhantomData, rc::Rc};
thread_local! { static OWNED_WORK: Cell<bool> = const { Cell::new(false) }; }

/// Marks work lent by an owning runtime. The marker grants no graph, phase,
/// reader, writer or execution authority; an owner uses it to refuse reentry.
/// The guard stays on its thread and restores the prior context on unwind.
#[must_use = "owner context lasts only while its guard is held"]
pub struct ExecutionOwnerContext {
    previous: bool,
    _thread: PhantomData<Rc<()>>,
}
impl ExecutionOwnerContext {
    pub fn enter() -> Self {
        CapturedOwnerContext(true).enter()
    }
    pub fn is_active() -> bool {
        OWNED_WORK.with(Cell::get)
    }
}
impl Drop for ExecutionOwnerContext {
    fn drop(&mut self) {
        OWNED_WORK.with(|owned| owned.set(self.previous));
    }
}

#[derive(Clone, Copy)]
pub(super) struct CapturedOwnerContext(bool);
impl CapturedOwnerContext {
    pub(super) fn capture() -> Self {
        Self(ExecutionOwnerContext::is_active())
    }
    pub(super) fn enter(self) -> ExecutionOwnerContext {
        ExecutionOwnerContext {
            previous: OWNED_WORK.with(|owned| owned.replace(self.0)),
            _thread: PhantomData,
        }
    }
}

#[cfg(test)]
mod tests;
