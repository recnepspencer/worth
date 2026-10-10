//! Law 2, scope: each owned guard clears exactly the pre-receiver arms it set.
use std::{
    cell::{Cell, RefCell},
    rc::{Rc, Weak},
};
type Cleanup = Box<dyn FnOnce()>;
struct Scope {
    id: u64,
    cleanups: RefCell<Vec<Cleanup>>,
}
thread_local! {
 static NEXT: Cell<u64> = const { Cell::new(0) };
 static SCOPES: RefCell<Vec<Weak<Scope>>> = const { RefCell::new(Vec::new()) };
}
pub(super) struct OwnedControlScope(Rc<Scope>);
impl OwnedControlScope {
    pub(super) fn install() -> Self {
        let id = NEXT.with(|next| {
            let id = next
                .get()
                .checked_add(1)
                .expect("control scope identity overflow");
            next.set(id);
            id
        });
        let scope = Rc::new(Scope {
            id,
            cleanups: RefCell::new(Vec::new()),
        });
        SCOPES.with(|scopes| scopes.borrow_mut().push(Rc::downgrade(&scope)));
        Self(scope)
    }
}
impl Drop for OwnedControlScope {
    fn drop(&mut self) {
        SCOPES.with(|scopes| {
            scopes
                .borrow_mut()
                .retain(|scope| scope.upgrade().is_some_and(|scope| scope.id != self.0.id))
        });
        let cleanups = std::mem::take(&mut *self.0.cleanups.borrow_mut());
        for cleanup in cleanups.into_iter().rev() {
            cleanup();
        }
    }
}
fn current() -> Rc<Scope> {
    SCOPES
        .with(|scopes| scopes.borrow().iter().rev().find_map(Weak::upgrade))
        .expect("a test control requires an owned Query observation guard")
}
pub(in crate::domain_computation) fn current_id() -> u64 {
    current().id
}
pub(in crate::domain_computation) fn is_live(id: u64) -> bool {
    SCOPES.with(|scopes| {
        scopes
            .borrow()
            .iter()
            .any(|scope| scope.upgrade().is_some_and(|scope| scope.id == id))
    })
}
pub(in crate::domain_computation) fn register(cleanup: impl FnOnce() + 'static) {
    current().cleanups.borrow_mut().push(Box::new(cleanup));
}
