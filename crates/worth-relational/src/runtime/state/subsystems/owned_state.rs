use std::sync::{Arc, RwLock, RwLockReadGuard, RwLockWriteGuard};

/// One runtime subsystem's authoritative mutable state, owned behind its own
/// lock so that ordinary services borrow the runtime shared rather than
/// exclusively.
///
/// Each subsystem holds its own cell. There is deliberately no cell that spans
/// subsystems: a settlement executor writing storage never blocks preparation
/// reading history, and no cell is ever held across branch waiting, durability
/// I/O, immutable-root construction, or derived projection work.
#[derive(Debug)]
pub(crate) struct RuntimeOwnedState<T> {
    state: Arc<RwLock<T>>,
}

impl<T> RuntimeOwnedState<T> {
    pub(crate) fn new(state: T) -> Self {
        Self {
            state: Arc::new(RwLock::new(state)),
        }
    }

    /// A second handle onto the same authoritative state, for cloneable owner
    /// bindings that outlive the borrow that produced them.
    pub(crate) fn share(&self) -> Self {
        Self {
            state: Arc::clone(&self.state),
        }
    }

    pub(crate) fn read(&self) -> RwLockReadGuard<'_, T> {
        self.state.read().expect(/* Proof: Authoritative subsystem guards are private owner operations: user callbacks, I/O and fallible preparation run outside them; guarded native updates operate on admitted phase values and do not unwind. */ "runtime subsystem lock poisoned")
    }

    pub(crate) fn write(&self) -> RwLockWriteGuard<'_, T> {
        self.state.write().expect(/* Proof: Authoritative subsystem guards are private owner operations: user callbacks, I/O and fallible preparation run outside them; guarded native updates operate on admitted phase values and do not unwind. */ "runtime subsystem lock poisoned")
    }
}

impl<T: Default> Default for RuntimeOwnedState<T> {
    fn default() -> Self {
        Self::new(T::default())
    }
}

impl<T: Clone> RuntimeOwnedState<T> {
    /// A detached deep copy for runtime forks, which must not share authority
    /// with the runtime they were taken from.
    pub(crate) fn detached(&self) -> Self {
        Self::new(self.read().clone())
    }
}
