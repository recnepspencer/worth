//! A member compute cannot open a second installed host advancement.
use std::{cell::Cell, marker::PhantomData, rc::Rc};
thread_local! { static ACTIVE: Cell<bool> = const { Cell::new(false) }; }

pub(in crate::domain_computation::primary_graph) struct MemberComputeCustody {
    previous: bool,
    _thread: PhantomData<Rc<()>>,
}
impl MemberComputeCustody {
    pub(super) fn enter() -> Self {
        Self {
            previous: ACTIVE.with(|active| active.replace(true)),
            _thread: PhantomData,
        }
    }
}
impl Drop for MemberComputeCustody {
    fn drop(&mut self) {
        ACTIVE.with(|active| active.set(self.previous));
    }
}
pub(in crate::domain_computation::primary_graph) fn active() -> bool {
    ACTIVE.with(Cell::get)
}

#[cfg(test)]
mod tests;
