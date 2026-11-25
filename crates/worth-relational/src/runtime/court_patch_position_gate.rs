//! Release the original scoped test pause at the owner boundary.
use super::operation_control::reservation_latch::PatchPositionReservationLatch;
use std::sync::Arc;

/// Release gate for the owner-issued reservation-held pause.
#[derive(Debug, Default)]
pub struct RelationalPatchPositionReservationGateForTest {
    latch: Arc<PatchPositionReservationLatch>,
}

impl RelationalPatchPositionReservationGateForTest {
    pub fn open(&self) {
        self.latch.open();
    }
}

impl AsRef<Arc<PatchPositionReservationLatch>> for RelationalPatchPositionReservationGateForTest {
    fn as_ref(&self) -> &Arc<PatchPositionReservationLatch> {
        &self.latch
    }
}
