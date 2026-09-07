#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PhysicalIntegrityComparisonDenial {
    InputBoundExceeded,
    ArtifactBoundExceeded,
    ReportBoundExceeded,
    MalformedProtocol,
    UnsupportedProtocol,
    InvalidCompatibility,
    InvalidRole,
    InvalidIdentity,
    StoreMismatch,
    ScenarioMismatch,
    SameObserver,
    DuplicateScope,
    InvalidObservation,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PhysicalIntegrityComparisonCounters {
    pub(super) agreements: u64,
    pub(super) disagreements: u64,
    pub(super) input_bytes: u64,
    pub(super) report_bytes: u64,
}
impl PhysicalIntegrityComparisonCounters {
    pub const fn agreements(self) -> u64 {
        self.agreements
    }
    pub const fn disagreements(self) -> u64 {
        self.disagreements
    }
    pub const fn input_bytes(self) -> u64 {
        self.input_bytes
    }
    pub const fn report_bytes(self) -> u64 {
        self.report_bytes
    }
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PhysicalIntegrityComparison {
    pub(super) wire: String,
    pub(super) counters: PhysicalIntegrityComparisonCounters,
}
impl PhysicalIntegrityComparison {
    pub fn encoded_report(&self) -> &str {
        &self.wire
    }
    pub const fn counters(&self) -> PhysicalIntegrityComparisonCounters {
        self.counters
    }
}
