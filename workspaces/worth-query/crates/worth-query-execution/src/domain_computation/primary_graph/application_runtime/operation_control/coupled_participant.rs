//! Receiver-owned corruption of the next genuine coupled writer, cleared on drop.
use super::{
    attempt_registration::WorthQueryApplicationAttemptOperationControl,
    WorthQueryPrimaryGraphApplicationRuntime,
};
use std::sync::{
    atomic::{AtomicU8, Ordering},
    Arc,
};

#[must_use = "retain the guard until the real coupled attempt finishes"]
pub struct WorthQueryCoupledParticipantCorruptionForTest {
    arm: Arc<AtomicU8>,
}
impl Drop for WorthQueryCoupledParticipantCorruptionForTest {
    fn drop(&mut self) {
        self.arm.store(0, Ordering::Release);
    }
}
impl WorthQueryApplicationAttemptOperationControl {
    pub(in crate::domain_computation::primary_graph) fn take_coupled_participant_corruption(
        &self,
    ) -> u8 {
        self.coupled_participant_fault.swap(0, Ordering::AcqRel)
    }
}
impl<S> WorthQueryPrimaryGraphApplicationRuntime<S> {
    #[doc(hidden)]
    pub fn supply_private_membership_deletion_for_test(
        &self,
    ) -> WorthQueryCoupledParticipantCorruptionForTest {
        let arm = Arc::clone(
            &self
                .runtime
                .primary_provider
                .application_attempt_operation_control
                .coupled_participant_fault,
        );
        assert_eq!(
            arm.swap(3, Ordering::AcqRel),
            0,
            "coupled corruption is already armed"
        );
        WorthQueryCoupledParticipantCorruptionForTest { arm }
    }
    /// Replays a token issued from the actual Native-applied owned part.
    #[doc(hidden)]
    pub fn replay_next_coupled_continuation_for_test(
        &self,
    ) -> WorthQueryCoupledParticipantCorruptionForTest {
        let arm = Arc::clone(
            &self
                .runtime
                .primary_provider
                .application_attempt_operation_control
                .coupled_participant_fault,
        );
        assert_eq!(
            arm.swap(4, Ordering::AcqRel),
            0,
            "coupled corruption is already armed"
        );
        WorthQueryCoupledParticipantCorruptionForTest { arm }
    }
    /// Arms this installed receiver; the dispatcher still derives and prepares
    /// the real source, producer, predecessor, effects and read set.
    #[doc(hidden)]
    pub fn corrupt_next_coupled_participant_for_test(
        &self,
        birth: bool,
    ) -> WorthQueryCoupledParticipantCorruptionForTest {
        let arm = Arc::clone(
            &self
                .runtime
                .primary_provider
                .application_attempt_operation_control
                .coupled_participant_fault,
        );
        assert_eq!(
            arm.swap(if birth { 2 } else { 1 }, Ordering::AcqRel),
            0,
            "coupled corruption is already armed"
        );
        WorthQueryCoupledParticipantCorruptionForTest { arm }
    }
}
