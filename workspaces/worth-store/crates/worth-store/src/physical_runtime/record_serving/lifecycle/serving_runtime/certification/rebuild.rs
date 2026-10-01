use super::super::ServingPhysicalRuntime;

impl ServingPhysicalRuntime {
    /// A denied synchronous Rebuild read has no retained retry owner and must
    /// release only its own background fairness attempt.
    pub fn certification_reject_oversized_rebuild_read_quantum(
        &self,
    ) -> (
        bool,
        [crate::physical_runtime::PhysicalWorkCounterSnapshot; 2],
        [worth_store_io_scheduler::foreground_reservation::PhysicalInstanceForegroundCapacitySnapshot; 2],
    ){
        let before_work = self.physical_work_counters();
        let before_capacity = self.parts.scheduler_admission.capacity_snapshot();
        let denied = self
            .parts
            .scheduler_admission
            .rebuild_read(
                crate::physical_runtime::record_serving::RebuildReadShape::admit(),
                self.parts.record_work.scheduler_security(),
                Some(std::num::NonZeroU64::new(u64::MAX).expect("nonzero denial quantum")),
                0,
            )
            .is_err();
        (
            denied,
            [before_work, self.physical_work_counters()],
            [
                before_capacity,
                self.parts.scheduler_admission.capacity_snapshot(),
            ],
        )
    }
}
