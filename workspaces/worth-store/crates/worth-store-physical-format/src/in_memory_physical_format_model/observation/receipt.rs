use super::{
    InMemoryPhysicalFormatModelCounterSnapshot, PlatformPhysicalAppendReport,
    PlatformPhysicalRootPublicationReport,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PlatformPhysicalLayoutAccessIntent {
    HiddenBroadScan,
    ExplicitDegradedExactScan,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PlatformPhysicalLayoutAccessRequest {
    intent: PlatformPhysicalLayoutAccessIntent,
    budget_rows: u64,
}

impl PlatformPhysicalLayoutAccessRequest {
    pub const fn hidden_broad_scan() -> Self {
        Self {
            intent: PlatformPhysicalLayoutAccessIntent::HiddenBroadScan,
            budget_rows: 0,
        }
    }

    pub const fn explicit_degraded_exact_scan(budget_rows: u64) -> Self {
        Self {
            intent: PlatformPhysicalLayoutAccessIntent::ExplicitDegradedExactScan,
            budget_rows,
        }
    }

    pub const fn intent(self) -> PlatformPhysicalLayoutAccessIntent {
        self.intent
    }

    pub const fn budget_rows(self) -> u64 {
        self.budget_rows
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PlatformPhysicalHiddenScanDenialReceipt {
    request: PlatformPhysicalLayoutAccessRequest,
    counters_before: InMemoryPhysicalFormatModelCounterSnapshot,
    counters_after: InMemoryPhysicalFormatModelCounterSnapshot,
}

impl PlatformPhysicalHiddenScanDenialReceipt {
    pub const fn request(self) -> PlatformPhysicalLayoutAccessRequest {
        self.request
    }

    pub const fn counters(self) -> InMemoryPhysicalFormatModelCounterSnapshot {
        self.counters_after
    }

    pub const fn counters_before(self) -> InMemoryPhysicalFormatModelCounterSnapshot {
        self.counters_before
    }

    pub const fn has_exact_zero_work_delta(self) -> bool {
        self.counters_after
            .is_exact_hidden_scan_denial_delta_from(self.counters_before)
    }

    pub const fn is_owner_denial(self) -> bool {
        matches!(
            self.request.intent(),
            PlatformPhysicalLayoutAccessIntent::HiddenBroadScan
        ) && self.has_exact_zero_work_delta()
    }

    pub(crate) const fn from_rejected_request(
        request: PlatformPhysicalLayoutAccessRequest,
        counters_before: InMemoryPhysicalFormatModelCounterSnapshot,
        counters_after: InMemoryPhysicalFormatModelCounterSnapshot,
    ) -> Self {
        Self {
            request,
            counters_before,
            counters_after,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PlatformPhysicalDegradedExactScanReceipt {
    request: PlatformPhysicalLayoutAccessRequest,
    observed_rows: u64,
    counters: InMemoryPhysicalFormatModelCounterSnapshot,
}

impl PlatformPhysicalDegradedExactScanReceipt {
    pub(in crate::in_memory_physical_format_model) const fn new(
        request: PlatformPhysicalLayoutAccessRequest,
        observed_rows: u64,
        counters: InMemoryPhysicalFormatModelCounterSnapshot,
    ) -> Self {
        Self {
            request,
            observed_rows,
            counters,
        }
    }

    pub const fn request(self) -> PlatformPhysicalLayoutAccessRequest {
        self.request
    }

    pub const fn observed_rows(self) -> u64 {
        self.observed_rows
    }

    pub const fn counters(self) -> InMemoryPhysicalFormatModelCounterSnapshot {
        self.counters
    }

    pub const fn is_budget_exact(self) -> bool {
        matches!(
            self.request.intent(),
            PlatformPhysicalLayoutAccessIntent::ExplicitDegradedExactScan
        ) && self.request.budget_rows() > 0
            && self.observed_rows <= self.request.budget_rows()
            && self.counters.scans() > 0
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PlatformPhysicalModelReceipt {
    operation: PlatformPhysicalModelOperation,
    strategy: PlatformPhysicalModelStrategy,
    counters: InMemoryPhysicalFormatModelCounterSnapshot,
    outcome: PlatformPhysicalModelOutcome,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PlatformPhysicalModelOperation {
    AppendPhysicalRecord,
    PublishPhysicalRoot,
    DenyHiddenBroadScan,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PlatformPhysicalModelOutcome {
    AppendCompleted,
    RootPublished,
    HiddenScanDenied,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PlatformPhysicalModelReceiptDenial {
    NotOwnerHiddenScanDenial,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PlatformPhysicalModelStrategy {
    BTreeRange,
}

impl PlatformPhysicalModelReceipt {
    pub const fn from_append(report: PlatformPhysicalAppendReport) -> Self {
        Self {
            operation: PlatformPhysicalModelOperation::AppendPhysicalRecord,
            strategy: PlatformPhysicalModelStrategy::BTreeRange,
            counters: report.counters(),
            outcome: PlatformPhysicalModelOutcome::AppendCompleted,
        }
    }

    pub fn from_hidden_scan_denial(
        receipt: PlatformPhysicalHiddenScanDenialReceipt,
    ) -> Result<Self, PlatformPhysicalModelReceiptDenial> {
        if !receipt.is_owner_denial() {
            return Err(PlatformPhysicalModelReceiptDenial::NotOwnerHiddenScanDenial);
        }
        Ok(Self {
            operation: PlatformPhysicalModelOperation::DenyHiddenBroadScan,
            strategy: PlatformPhysicalModelStrategy::BTreeRange,
            counters: receipt.counters(),
            outcome: PlatformPhysicalModelOutcome::HiddenScanDenied,
        })
    }

    pub const fn from_root_publication(report: &PlatformPhysicalRootPublicationReport) -> Self {
        Self {
            operation: PlatformPhysicalModelOperation::PublishPhysicalRoot,
            strategy: PlatformPhysicalModelStrategy::BTreeRange,
            counters: report.counters(),
            outcome: PlatformPhysicalModelOutcome::RootPublished,
        }
    }

    pub const fn operation(self) -> PlatformPhysicalModelOperation {
        self.operation
    }

    pub const fn outcome(self) -> PlatformPhysicalModelOutcome {
        self.outcome
    }

    pub const fn strategy(self) -> PlatformPhysicalModelStrategy {
        self.strategy
    }

    pub const fn counters(self) -> InMemoryPhysicalFormatModelCounterSnapshot {
        self.counters
    }
}
