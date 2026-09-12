use worth_store_physical_format::RootPublicationCell;

use crate::physical_runtime::{LifecycleGeneration, RuntimeIdentity};

/// Describes the actual root selected for a read, not authority to reacquire it.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PhysicalProtectedRootObservation {
    runtime: RuntimeIdentity,
    lifecycle: LifecycleGeneration,
    root: RootPublicationCell,
}

impl PhysicalProtectedRootObservation {
    pub(in crate::physical_runtime::stability) const fn new(
        runtime: RuntimeIdentity,
        lifecycle: LifecycleGeneration,
        root: RootPublicationCell,
    ) -> Self {
        Self {
            runtime,
            lifecycle,
            root,
        }
    }

    pub const fn runtime(self) -> RuntimeIdentity {
        self.runtime
    }

    pub const fn lifecycle(self) -> LifecycleGeneration {
        self.lifecycle
    }

    pub const fn root(self) -> RootPublicationCell {
        self.root
    }
}
