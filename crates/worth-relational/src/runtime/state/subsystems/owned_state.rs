use std::fmt;
use std::sync::{Arc, RwLock, RwLockReadGuard, RwLockWriteGuard};

/// One runtime subsystem's authoritative mutable state, owned behind its own
/// lock so that ordinary services borrow the runtime shared rather than
/// exclusively.
///
/// Each subsystem holds its own cell. There is deliberately no cell that spans
/// subsystems: a settlement executor writing storage never blocks preparation
/// reading history, and no cell is ever held across branch waiting, durability
/// I/O, immutable-root construction, or derived projection work.
pub(crate) struct RuntimeOwnedState<T> {
    state: Arc<dyn SubsystemCell<T>>,
}

// The subsystem owner proves that its exact cell may be shared across threads.
// A receiver of that cell need not re-prove the storage layout of every index,
// history entry or retained root reachable from it.
trait SubsystemCell<T>: Send + Sync {
    fn lock(&self) -> &RwLock<T>;
}

impl<T: Send + Sync> SubsystemCell<T> for RwLock<T> {
    fn lock(&self) -> &RwLock<T> {
        self
    }
}

impl<T: fmt::Debug> fmt::Debug for RuntimeOwnedState<T> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("RuntimeOwnedState")
            .field("state", self.state.lock())
            .finish()
    }
}

impl<T> RuntimeOwnedState<T> {
    pub(crate) fn try_read(&self) -> Option<RwLockReadGuard<'_, T>> {
        match self.state.lock().try_read() {
            Ok(value) => Some(value),
            Err(std::sync::TryLockError::Poisoned(error)) => Some(error.into_inner()),
            Err(std::sync::TryLockError::WouldBlock) => None,
        }
    }
    pub(crate) fn try_write(&self) -> Option<RwLockWriteGuard<'_, T>> {
        match self.state.lock().try_write() {
            Ok(value) => Some(value),
            Err(std::sync::TryLockError::Poisoned(error)) => Some(error.into_inner()),
            Err(std::sync::TryLockError::WouldBlock) => None,
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
        self.state.lock().read().expect(/* Proof: Authoritative subsystem guards are private owner operations: user callbacks, I/O and fallible preparation run outside them; guarded native updates operate on admitted phase values and do not unwind. */ "runtime subsystem lock poisoned")
    }

    pub(crate) fn write(&self) -> RwLockWriteGuard<'_, T> {
        self.state.lock().write().expect(/* Proof: Authoritative subsystem guards are private owner operations: user callbacks, I/O and fallible preparation run outside them; guarded native updates operate on admitted phase values and do not unwind. */ "runtime subsystem lock poisoned")
    }
}

impl<T: Send + Sync + 'static> RuntimeOwnedState<T> {
    pub(crate) fn new(state: T) -> Self {
        Self {
            state: Arc::new(RwLock::new(state)),
        }
    }
}

impl<T: Default + Send + Sync + 'static> Default for RuntimeOwnedState<T> {
    fn default() -> Self {
        Self::new(T::default())
    }
}

impl<T: Clone + Send + Sync + 'static> RuntimeOwnedState<T> {
    /// A detached deep copy for runtime forks, which must not share authority
    /// with the runtime they were taken from.
    pub(crate) fn detached(&self) -> Self {
        Self::new(self.read().clone())
    }
}
