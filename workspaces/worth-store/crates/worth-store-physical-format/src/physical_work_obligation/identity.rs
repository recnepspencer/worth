use core::num::NonZeroU64;

/// Canonical identity encoded by a physical-work obligation filename.
///
/// Store identity is not included: obligation bytes can describe Store bytes
/// but cannot establish the namespace publication that makes them stable.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct PhysicalWorkObligationIdentity {
    runtime: NonZeroU64,
    generation: NonZeroU64,
    operation: NonZeroU64,
}

impl PhysicalWorkObligationIdentity {
    pub const fn new(runtime: NonZeroU64, generation: NonZeroU64, operation: NonZeroU64) -> Self {
        Self {
            runtime,
            generation,
            operation,
        }
    }

    pub const fn runtime(self) -> NonZeroU64 {
        self.runtime
    }

    pub const fn generation(self) -> NonZeroU64 {
        self.generation
    }

    pub const fn operation(self) -> NonZeroU64 {
        self.operation
    }
}
