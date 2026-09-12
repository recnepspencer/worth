use worth_store_physical_isolation::ReadPlanCounterSnapshot;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct StablePhysicalReadExecutionCounters {
    guard_admissions: u64,
    guarded_byte_reads: u64,
    guarded_bytes: u64,
    execution_time_reference_discoveries: u64,
    retry_decisions: u64,
    blocking_io_events: u64,
    hidden_latch_io_denials: u64,
    compact_footprint_checks: u64,
    broad_footprint_scans: u64,
    plan_allocations: u64,
    diagnostic_materializations: u64,
}

impl StablePhysicalReadExecutionCounters {
    #[cfg(any(test, feature = "certification-test-authority"))]
    pub(crate) const fn for_certification_test(guarded_bytes: u64) -> Self {
        Self {
            guard_admissions: 1,
            guarded_byte_reads: 1,
            guarded_bytes,
            execution_time_reference_discoveries: 0,
            retry_decisions: 0,
            blocking_io_events: 0,
            hidden_latch_io_denials: 0,
            compact_footprint_checks: 1,
            broad_footprint_scans: 0,
            plan_allocations: 1,
            diagnostic_materializations: 0,
        }
    }

    pub const fn from_plan_counters(plan: ReadPlanCounterSnapshot) -> Self {
        Self {
            guard_admissions: 0,
            guarded_byte_reads: 0,
            guarded_bytes: 0,
            execution_time_reference_discoveries: 0,
            retry_decisions: plan.retry_decisions(),
            blocking_io_events: 0,
            hidden_latch_io_denials: 0,
            compact_footprint_checks: 0,
            broad_footprint_scans: 0,
            plan_allocations: plan.allocation_events(),
            diagnostic_materializations: 0,
        }
    }

    pub const fn with_guard_admission(self) -> Self {
        Self {
            guard_admissions: self.guard_admissions + 1,
            ..self
        }
    }

    pub const fn with_guarded_byte_read(self, bytes: u64) -> Self {
        Self {
            guarded_byte_reads: self.guarded_byte_reads + 1,
            guarded_bytes: self.guarded_bytes + bytes,
            ..self
        }
    }

    pub const fn with_execution_time_reference_discovery(self) -> Self {
        Self {
            execution_time_reference_discoveries: self.execution_time_reference_discoveries + 1,
            ..self
        }
    }

    pub const fn with_retry_decision(self) -> Self {
        Self {
            retry_decisions: self.retry_decisions + 1,
            ..self
        }
    }

    pub const fn with_blocking_io_event(self) -> Self {
        Self {
            blocking_io_events: self.blocking_io_events + 1,
            ..self
        }
    }

    pub const fn with_compact_footprint_check(self) -> Self {
        Self {
            compact_footprint_checks: self.compact_footprint_checks + 1,
            ..self
        }
    }

    pub const fn with_hidden_latch_io_denial(self) -> Self {
        Self {
            hidden_latch_io_denials: self.hidden_latch_io_denials + 1,
            ..self
        }
    }

    pub const fn guard_admissions(self) -> u64 {
        self.guard_admissions
    }

    pub const fn guarded_byte_reads(self) -> u64 {
        self.guarded_byte_reads
    }

    pub const fn guarded_bytes(self) -> u64 {
        self.guarded_bytes
    }

    pub const fn execution_time_reference_discoveries(self) -> u64 {
        self.execution_time_reference_discoveries
    }

    pub const fn retry_decisions(self) -> u64 {
        self.retry_decisions
    }

    pub const fn blocking_io_events(self) -> u64 {
        self.blocking_io_events
    }

    pub const fn hidden_latch_io_denials(self) -> u64 {
        self.hidden_latch_io_denials
    }

    pub const fn compact_footprint_checks(self) -> u64 {
        self.compact_footprint_checks
    }

    pub const fn broad_footprint_scans(self) -> u64 {
        self.broad_footprint_scans
    }

    pub const fn plan_allocations(self) -> u64 {
        self.plan_allocations
    }

    pub const fn diagnostic_materializations(self) -> u64 {
        self.diagnostic_materializations
    }
}
