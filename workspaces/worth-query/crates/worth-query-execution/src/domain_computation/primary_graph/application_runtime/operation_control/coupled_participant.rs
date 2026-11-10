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
        prepared_writers: usize,
    ) -> u8 {
        if self.coupled_participant_fault.load(Ordering::Acquire) == 5 && prepared_writers < 2 {
            return 0;
        }
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
    /// The second actual writer destructively targets the first writer's real allocation.
    #[doc(hidden)]
    pub fn destroy_another_writers_fresh_record_for_test(
        &self,
    ) -> WorthQueryCoupledParticipantCorruptionForTest {
        self.arm_fresh_record_corruption(5)
    }
    /// The actual producer destructively targets its authored prefix's new source.
    #[doc(hidden)]
    pub fn destroy_the_authored_prefixs_fresh_record_for_test(
        &self,
    ) -> WorthQueryCoupledParticipantCorruptionForTest {
        self.arm_fresh_record_corruption(6)
    }
    /// The actual writer's own complete commands return as a continued round of its route.
    #[doc(hidden)]
    pub fn continue_next_coupled_route_with_its_effect_for_test(
        &self,
    ) -> WorthQueryCoupledParticipantCorruptionForTest {
        self.arm_fresh_record_corruption(7)
    }
    /// Only the actual writer's in-place patches return, at values D already holds.
    #[doc(hidden)]
    pub fn continue_next_coupled_route_unchanged_for_test(
        &self,
    ) -> WorthQueryCoupledParticipantCorruptionForTest {
        self.arm_fresh_record_corruption(8)
    }
    /// The actual writer's in-place patches return with each unsigned value moved by one.
    #[doc(hidden)]
    pub fn continue_next_coupled_route_with_a_changed_patch_for_test(
        &self,
    ) -> WorthQueryCoupledParticipantCorruptionForTest {
        self.arm_fresh_record_corruption(9)
    }
    fn arm_fresh_record_corruption(
        &self,
        fault: u8,
    ) -> WorthQueryCoupledParticipantCorruptionForTest {
        let arm = Arc::clone(
            &self
                .runtime
                .primary_provider
                .application_attempt_operation_control
                .coupled_participant_fault,
        );
        assert_eq!(
            arm.swap(fault, Ordering::AcqRel),
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
