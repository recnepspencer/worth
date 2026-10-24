use std::sync::Arc;

use worth_relational::facade::runtime::RelationalRuntime;
use worth_store::physical_runtime::{
    ClosedRuntime, ServingPhysicalRuntime, ServingShutdownOutcome,
};

use super::RelationalStoreHome;

/// What closing a home did to its Store.
#[non_exhaustive]
pub enum RelationalStoreHomeClosed {
    /// The Store shut down through its own close.
    Closed(ServingShutdownOutcome<ClosedRuntime>),
    /// Something outside the home still held the Store, so it was left to
    /// its holders. Durable state is unaffected: every acknowledged commit was
    /// durable before it was acknowledged.
    StillShared,
}

impl std::fmt::Debug for RelationalStoreHomeClosed {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(match self {
            Self::Closed(_) => "Closed",
            Self::StillShared => "StillShared",
        })
    }
}

impl RelationalStoreHome {
    /// Stop serving and close the Store. Durability never depends on close.
    pub fn close(self) -> RelationalStoreHomeClosed {
        let Self { runtime, serving } = self;
        drop(runtime);
        match Arc::try_unwrap(serving) {
            Ok(serving) => RelationalStoreHomeClosed::Closed(serving.close()),
            Err(_) => RelationalStoreHomeClosed::StillShared,
        }
    }
}

/// Release a home that refused to open: the runtime first, because it holds
/// the journal medium, then the Store.
pub(super) fn abandon(runtime: RelationalRuntime, serving: Arc<ServingPhysicalRuntime>) {
    drop(runtime);
    if let Ok(serving) = Arc::try_unwrap(serving) {
        let _closed = serving.close();
    }
}
