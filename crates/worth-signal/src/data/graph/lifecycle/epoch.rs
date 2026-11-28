use super::super::signal_graph::SignalGraph;

impl SignalGraph {
    pub fn tombstone_count(&self) -> u32 {
        self.arena.compaction.tombstone_count
    }

    pub fn gc_threshold(&self) -> u32 {
        self.arena.compaction.gc_threshold
    }

    #[cfg(test)]
    pub(crate) fn should_gc(&self) -> bool {
        self.arena.should_gc()
    }

    pub fn run_gc_epoch(&mut self) {
        let active_nodes = self.arena.active_nodes as usize;
        let should_run = self
            .arena
            .should_run_compaction_epoch(&self.topology, active_nodes);
        let family_budget = self
            .arena
            .compaction_epoch_budget(&self.topology, active_nodes);
        let Some(epoch_plan) = self.arena.plan_compaction_epoch(should_run, family_budget) else {
            return;
        };
        let gc_start = self
            .captures_observation_surface(
                crate::logic::transaction::SignalObservationSurface::OptionalTelemetry,
            )
            .then(|| self.time_sampler().monotonic());
        for family in epoch_plan.families() {
            self.compact_storage_family(family);
        }
        self.arena
            .complete_compaction_epoch(epoch_plan.family_budget());
        if let Some(gc_start) = gc_start {
            let elapsed_nanos = self.elapsed_nanos(gc_start);
            if let Some(mut telemetry) = self.telemetry_mut() {
                telemetry.storage.gc_epoch_count += 1;
                telemetry.storage.gc_epoch_nanos += elapsed_nanos;
            }
        }
    }

    #[cfg(test)]
    pub(crate) fn gc_compaction_debt_for_test(&self) -> u32 {
        self.arena.compaction.debt
    }

    #[cfg(test)]
    pub(crate) fn gc_compaction_cursor_for_test(&self) -> u8 {
        self.arena.compaction.cursor
    }

    #[cfg(test)]
    pub(crate) fn set_gc_compaction_state_for_test(&mut self, debt: u32, cursor: u8) {
        self.arena.compaction.debt = debt;
        self.arena.compaction.cursor = cursor % 3;
    }
}
