//! Lineage port adapter for `WorthQueryUnpublishedIdempotencyStore`.

use super::unpublished_idempotency::{preparation, recovered, SharedUnpublishedIdempotencyStore};
use worth_runtime_world::facade::ProductUnpublishedRecoveryHandle;

pub(super) struct UnpublishedIdempotencyStorePort(pub(super) SharedUnpublishedIdempotencyStore);

impl worth_query_execution_lineage::facade::ports::UnpublishedIdempotencyPort
    for UnpublishedIdempotencyStorePort
{
    fn abandon_preparing_world(&self, handle: &ProductUnpublishedRecoveryHandle) -> Result<(), ()> {
        preparation::abandon_preparing_world(&self.0, handle)
    }

    fn finish_recovered(&self, handle: &ProductUnpublishedRecoveryHandle) {
        recovered::finish_recovered(&self.0, handle);
    }

    fn as_any(&self) -> &(dyn std::any::Any + Send + Sync) {
        self
    }
}
