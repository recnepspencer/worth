use std::marker::PhantomData;
use std::sync::atomic::{AtomicU64, Ordering};

static NEXT_COOPERATIVE_EXECUTION_ADMISSION: AtomicU64 = AtomicU64::new(1);

/// Move-only proof that a native provider execution was admitted through the
/// exact runtime-owned start or restore port.
///
/// The runtime-owned port takes physical ownership of the execution before
/// returning this proof. The proof's fields and constructor are private, so
/// returning a raw execution cannot satisfy the provider contract and a
/// provider panic cannot destroy an already-admitted execution in its own
/// stack frame.
#[must_use = "cooperative execution admission must be returned to the runtime"]
pub struct WorthQueryCooperativeGraphProviderExecution<E> {
    arena_identity: u64,
    admission_identity: u64,
    execution_type: PhantomData<fn() -> E>,
}

impl<E> WorthQueryCooperativeGraphProviderExecution<E> {
    pub(super) const fn new(arena_identity: u64, admission_identity: u64) -> Self {
        Self {
            arena_identity,
            admission_identity,
            execution_type: PhantomData,
        }
    }

    pub(super) const fn arena_identity(&self) -> u64 {
        self.arena_identity
    }

    pub(super) const fn admission_identity(&self) -> u64 {
        self.admission_identity
    }
}

impl<E> std::fmt::Debug for WorthQueryCooperativeGraphProviderExecution<E> {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("WorthQueryCooperativeGraphProviderExecution")
            .field("arena_identity", &self.arena_identity)
            .field("admission_identity", &self.admission_identity)
            .finish_non_exhaustive()
    }
}

pub(super) fn next_cooperative_execution_admission_identity() -> u64 {
    NEXT_COOPERATIVE_EXECUTION_ADMISSION.fetch_add(1, Ordering::Relaxed)
}
