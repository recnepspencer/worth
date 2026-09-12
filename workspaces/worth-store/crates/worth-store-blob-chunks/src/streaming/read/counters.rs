use worth_store::physical_runtime::stability::StablePhysicalReadExecutionCounters;
use worth_store_budgets::CounterEvidenceStrength;
use worth_store_io_scheduler::BackgroundPacingCounterSnapshot;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BlobStreamingReadCounterSnapshot {
    windows_observed: u64,
    bytes_read: u64,
    chunks_read: u64,
    chunks_verified: u64,
    chunk_checksum_verifications: u64,
    digest_updates: u64,
    read_amplification_bytes: u64,
    allocation_count: u64,
    scheduler_waits: u64,
    pressure_yield_denials: u64,
    pressure_deferred_denials: u64,
    pressure_denied_denials: u64,
    pressure_throttles: u64,
    pressure_admitted_with_debt: u64,
    pressure_violations: u64,
    protected_read_denials: u64,
    cold_unavailable_denials: u64,
    stale_read_denials: u64,
    corrupt_chunk_denials: u64,
    order_denials: u64,
    missing_chunk_denials: u64,
    peak_resident_bytes: u64,
    counter_strength: CounterEvidenceStrength,
}

impl BlobStreamingReadCounterSnapshot {
    pub(crate) const fn start(counter_strength: CounterEvidenceStrength) -> Self {
        Self {
            windows_observed: 0,
            bytes_read: 0,
            chunks_read: 0,
            chunks_verified: 0,
            chunk_checksum_verifications: 0,
            digest_updates: 0,
            read_amplification_bytes: 0,
            allocation_count: 0,
            scheduler_waits: 0,
            pressure_yield_denials: 0,
            pressure_deferred_denials: 0,
            pressure_denied_denials: 0,
            pressure_throttles: 0,
            pressure_admitted_with_debt: 0,
            pressure_violations: 0,
            protected_read_denials: 0,
            cold_unavailable_denials: 0,
            stale_read_denials: 0,
            corrupt_chunk_denials: 0,
            order_denials: 0,
            missing_chunk_denials: 0,
            peak_resident_bytes: 0,
            counter_strength,
        }
    }

    pub(crate) const fn record_allocation(self) -> Self {
        Self {
            allocation_count: self.allocation_count + 1,
            ..self
        }
    }

    pub(crate) const fn record_stable_read(
        self,
        counters: StablePhysicalReadExecutionCounters,
    ) -> Self {
        Self {
            scheduler_waits: self.scheduler_waits + counters.retry_decisions(),
            protected_read_denials: self.protected_read_denials
                + counters.hidden_latch_io_denials(),
            ..self
        }
    }

    pub(crate) const fn merge_pressure_counters(self, other: Self) -> Self {
        Self {
            scheduler_waits: self.scheduler_waits + other.scheduler_waits,
            pressure_yield_denials: self.pressure_yield_denials + other.pressure_yield_denials,
            pressure_deferred_denials: self.pressure_deferred_denials
                + other.pressure_deferred_denials,
            pressure_denied_denials: self.pressure_denied_denials + other.pressure_denied_denials,
            pressure_throttles: self.pressure_throttles + other.pressure_throttles,
            pressure_admitted_with_debt: self.pressure_admitted_with_debt
                + other.pressure_admitted_with_debt,
            pressure_violations: self.pressure_violations + other.pressure_violations,
            ..self
        }
    }

    pub(crate) const fn record_background_pressure(
        self,
        counters: BackgroundPacingCounterSnapshot,
    ) -> Self {
        Self {
            scheduler_waits: self.scheduler_waits
                + counters.yield_events()
                + counters.deferred_events()
                + counters.denied_events()
                + counters.throttle_events()
                + counters.admitted_with_debt_events()
                + counters.violation_events()
                + counters.foreground_pressure_events(),
            pressure_yield_denials: self.pressure_yield_denials + counters.yield_events(),
            pressure_deferred_denials: self.pressure_deferred_denials + counters.deferred_events(),
            pressure_denied_denials: self.pressure_denied_denials + counters.denied_events(),
            pressure_throttles: self.pressure_throttles + counters.throttle_events(),
            pressure_admitted_with_debt: self.pressure_admitted_with_debt
                + counters.admitted_with_debt_events(),
            pressure_violations: self.pressure_violations + counters.violation_events(),
            ..self
        }
    }

    pub(crate) const fn observe_read_window(self, bytes: u64) -> Self {
        let peak_resident_bytes = if bytes > self.peak_resident_bytes {
            bytes
        } else {
            self.peak_resident_bytes
        };
        Self {
            windows_observed: self.windows_observed + 1,
            bytes_read: self.bytes_read + bytes,
            chunks_read: self.chunks_read + 1,
            read_amplification_bytes: self.read_amplification_bytes + bytes,
            peak_resident_bytes,
            ..self
        }
    }

    pub(crate) const fn record_verified_chunk(self) -> Self {
        Self {
            chunks_verified: self.chunks_verified + 1,
            chunk_checksum_verifications: self.chunk_checksum_verifications + 1,
            digest_updates: self.digest_updates + 1,
            ..self
        }
    }

    pub(crate) const fn record_cold_unavailable_denial(self) -> Self {
        Self {
            cold_unavailable_denials: self.cold_unavailable_denials + 1,
            ..self
        }
    }

    pub(crate) const fn record_corrupt_chunk_denial(self) -> Self {
        Self {
            corrupt_chunk_denials: self.corrupt_chunk_denials + 1,
            ..self
        }
    }

    pub(crate) const fn record_order_denial(self) -> Self {
        Self {
            order_denials: self.order_denials + 1,
            ..self
        }
    }

    pub(crate) const fn record_missing_chunk_denial(self) -> Self {
        Self {
            missing_chunk_denials: self.missing_chunk_denials + 1,
            ..self
        }
    }

    pub(crate) const fn record_stale_read_denial(self) -> Self {
        Self {
            stale_read_denials: self.stale_read_denials + 1,
            ..self
        }
    }

    pub const fn windows_observed(self) -> u64 {
        self.windows_observed
    }
    pub const fn bytes_read(self) -> u64 {
        self.bytes_read
    }
    pub const fn chunks_read(self) -> u64 {
        self.chunks_read
    }
    pub const fn chunks_verified(self) -> u64 {
        self.chunks_verified
    }
    pub const fn chunk_checksum_verifications(self) -> u64 {
        self.chunk_checksum_verifications
    }
    pub const fn digest_updates(self) -> u64 {
        self.digest_updates
    }
    pub const fn read_amplification_bytes(self) -> u64 {
        self.read_amplification_bytes
    }
    pub const fn allocation_count(self) -> u64 {
        self.allocation_count
    }
    pub const fn scheduler_waits(self) -> u64 {
        self.scheduler_waits
    }
    pub const fn pressure_yield_denials(self) -> u64 {
        self.pressure_yield_denials
    }
    pub const fn pressure_deferred_denials(self) -> u64 {
        self.pressure_deferred_denials
    }
    pub const fn pressure_denied_denials(self) -> u64 {
        self.pressure_denied_denials
    }
    pub const fn pressure_throttles(self) -> u64 {
        self.pressure_throttles
    }
    pub const fn pressure_admitted_with_debt(self) -> u64 {
        self.pressure_admitted_with_debt
    }
    pub const fn pressure_violations(self) -> u64 {
        self.pressure_violations
    }
    pub const fn protected_read_denials(self) -> u64 {
        self.protected_read_denials
    }
    pub const fn cold_unavailable_denials(self) -> u64 {
        self.cold_unavailable_denials
    }
    pub const fn stale_read_denials(self) -> u64 {
        self.stale_read_denials
    }
    pub const fn corrupt_chunk_denials(self) -> u64 {
        self.corrupt_chunk_denials
    }
    pub const fn order_denials(self) -> u64 {
        self.order_denials
    }
    pub const fn missing_chunk_denials(self) -> u64 {
        self.missing_chunk_denials
    }
    pub const fn peak_resident_bytes(self) -> u64 {
        self.peak_resident_bytes
    }
    pub const fn counter_strength(self) -> CounterEvidenceStrength {
        self.counter_strength
    }
}
