use super::ExecutionPosture;

/// Physical scheduling measurements. They describe placement and discarded
/// work, and are never the canonical charged-work gate.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct ExecutionPhysicalReport {
    active_workers_high_watermark: usize,
    peak_charged_memory_bytes: u64,
    /// `None` when a backend cannot observe its scheduler's internal steals.
    steals: Option<u64>,
    peak_queue_width: usize,
    discarded_in_flight_work: u64,
}

impl ExecutionPhysicalReport {
    pub const fn new(
        active_workers_high_watermark: usize,
        peak_charged_memory_bytes: u64,
        steals: Option<u64>,
        peak_queue_width: usize,
        discarded_in_flight_work: u64,
    ) -> Self {
        Self {
            active_workers_high_watermark,
            peak_charged_memory_bytes,
            steals,
            peak_queue_width,
            discarded_in_flight_work,
        }
    }

    pub const fn active_workers_high_watermark(self) -> usize {
        self.active_workers_high_watermark
    }

    pub const fn peak_charged_memory_bytes(self) -> u64 {
        self.peak_charged_memory_bytes
    }

    pub const fn steals(self) -> Option<u64> {
        self.steals
    }

    pub const fn peak_queue_width(self) -> usize {
        self.peak_queue_width
    }

    pub const fn discarded_in_flight_work(self) -> u64 {
        self.discarded_in_flight_work
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExecutionFallbackCause {
    NoWork,
    NoLease,
    PolicySerial,
    PlatformSerial,
    WorkerLimit,
    Capacity,
    OracleSerial,
}

/// Portable computation cost and resolved execution posture. Future pattern
/// reports carry their own partition and reduction detail alongside this core.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ExecutionReport {
    resolved_posture: ExecutionPosture,
    charged_work: u64,
    charged_span: u64,
    physical: ExecutionPhysicalReport,
    fallback: Option<ExecutionFallbackCause>,
}

impl ExecutionReport {
    pub const fn new(
        resolved_posture: ExecutionPosture,
        charged_work: u64,
        charged_span: u64,
        physical: ExecutionPhysicalReport,
    ) -> Self {
        Self {
            resolved_posture,
            charged_work,
            charged_span,
            physical,
            fallback: None,
        }
    }

    pub const fn resolved_posture(self) -> ExecutionPosture {
        self.resolved_posture
    }

    pub const fn charged_work(self) -> u64 {
        self.charged_work
    }

    pub const fn charged_span(self) -> u64 {
        self.charged_span
    }

    pub const fn physical(self) -> ExecutionPhysicalReport {
        self.physical
    }

    pub const fn with_fallback(mut self, cause: ExecutionFallbackCause) -> Self {
        self.fallback = Some(cause);
        self
    }

    pub const fn fallback(self) -> Option<ExecutionFallbackCause> {
        self.fallback
    }
}
