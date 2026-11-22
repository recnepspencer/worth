//! Lineage handle to provider unpublished-idempotency disposition custody.

use crate::ports::UnpublishedIdempotencyPort;
use std::sync::Arc;
use worth_runtime_world::facade::ProductUnpublishedRecoveryHandle;

#[derive(Clone)]
pub struct WorthQueryUnpublishedIdempotencyDisposition {
    port: Arc<dyn UnpublishedIdempotencyPort>,
}

impl WorthQueryUnpublishedIdempotencyDisposition {
    pub fn from_port(port: Arc<dyn UnpublishedIdempotencyPort>) -> Self {
        Self { port }
    }

    pub fn port(&self) -> &Arc<dyn UnpublishedIdempotencyPort> {
        &self.port
    }

    pub fn abandon_preparing_world(
        &self,
        handle: &ProductUnpublishedRecoveryHandle,
    ) -> Result<(), ()> {
        self.port.abandon_preparing_world(handle)
    }

    pub fn finish_recovered(&self, handle: &ProductUnpublishedRecoveryHandle) {
        self.port.finish_recovered(handle);
    }
}
