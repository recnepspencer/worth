use std::sync::{Arc, Mutex};

use crate::runtime::RelationalRuntime;

/// A handle to one live Relational runtime, however its owner shares it.
///
/// Callers hold a runtime either as an immutable `Arc<RelationalRuntime>` or
/// as an `Arc<Mutex<RelationalRuntime>>` shared with writers. The handle
/// covers both, so a change-source consumer is written once.
///
/// # Locking
///
/// - An immutable handle takes no lock.
/// - A shared handle holds the runtime's mutex for the whole
///   [`with_runtime`](Self::with_runtime) closure. The closure must not reach
///   the same mutex again, through this handle or any other holder; that
///   re-entry deadlocks. Code that already holds the runtime passes
///   `&RelationalRuntime` directly instead.
/// - A poisoned mutex is read through. A panic in another holder does not make
///   the runtime unreadable.
#[derive(Clone)]
pub struct RelationalRuntimeHandle {
    ownership: RuntimeOwnership,
    runtime_instance_id: u64,
}

#[derive(Clone)]
enum RuntimeOwnership {
    Immutable(Arc<RelationalRuntime>),
    Shared(Arc<Mutex<RelationalRuntime>>),
}

impl RelationalRuntimeHandle {
    /// Handle a runtime that no writer mutates through a lock.
    pub fn immutable(runtime: Arc<RelationalRuntime>) -> Self {
        Self {
            runtime_instance_id: runtime.runtime_instance_id(),
            ownership: RuntimeOwnership::Immutable(runtime),
        }
    }

    /// Handle a runtime shared with writers behind a mutex.
    pub fn shared(runtime: Arc<Mutex<RelationalRuntime>>) -> Self {
        let runtime_instance_id = runtime
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .runtime_instance_id();
        Self {
            ownership: RuntimeOwnership::Shared(runtime),
            runtime_instance_id,
        }
    }

    /// Run `read` against the runtime, holding a shared runtime's mutex for
    /// the whole closure. See the type's locking rules.
    pub fn with_runtime<T>(&self, read: impl FnOnce(&RelationalRuntime) -> T) -> T {
        match &self.ownership {
            RuntimeOwnership::Immutable(runtime) => read(runtime),
            RuntimeOwnership::Shared(runtime) => {
                let runtime = runtime
                    .lock()
                    .unwrap_or_else(std::sync::PoisonError::into_inner);
                read(&runtime)
            }
        }
    }

    /// The instance id of the handled runtime. Read once when the handle is
    /// made, so it takes no lock and is safe inside
    /// [`with_runtime`](Self::with_runtime).
    pub const fn runtime_instance_id(&self) -> u64 {
        self.runtime_instance_id
    }
}

impl std::fmt::Debug for RelationalRuntimeHandle {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("RelationalRuntimeHandle")
            .field("runtime_instance_id", &self.runtime_instance_id)
            .field(
                "ownership",
                &match self.ownership {
                    RuntimeOwnership::Immutable(_) => "immutable",
                    RuntimeOwnership::Shared(_) => "shared",
                },
            )
            .finish()
    }
}
