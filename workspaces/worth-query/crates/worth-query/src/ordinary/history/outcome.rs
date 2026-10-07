use crate::historical::HistoricalMaterializationPathMetadata;
use crate::ordinary::read::WorthQueryReadContextReceipt;
use crate::runtime::WorthQueryReadResult;

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct WorthQueryHistoricalJourneyCounters {
    path_admission_attempt_count: usize,
    path_admitted_count: usize,
    context_admission_attempt_count: usize,
    planning_attempt_count: usize,
    basis_binding_attempt_count: usize,
    lower_runtime_execution_attempt_count: usize,
}

impl WorthQueryHistoricalJourneyCounters {
    pub fn path_admission_attempt_count(&self) -> usize {
        self.path_admission_attempt_count
    }
    pub fn path_admitted_count(&self) -> usize {
        self.path_admitted_count
    }
    pub fn context_admission_attempt_count(&self) -> usize {
        self.context_admission_attempt_count
    }
    pub fn planning_attempt_count(&self) -> usize {
        self.planning_attempt_count
    }
    pub fn basis_binding_attempt_count(&self) -> usize {
        self.basis_binding_attempt_count
    }
    pub fn lower_runtime_execution_attempt_count(&self) -> usize {
        self.lower_runtime_execution_attempt_count
    }

    pub(crate) fn begin() -> Self {
        Self {
            path_admission_attempt_count: 1,
            ..Self::default()
        }
    }
    pub(crate) fn admit_path(mut self) -> Self {
        self.path_admitted_count = 1;
        self
    }
    pub(crate) fn admit_context(mut self) -> Self {
        self.context_admission_attempt_count = 1;
        self
    }
    pub(crate) fn plan(mut self) -> Self {
        self.planning_attempt_count = 1;
        self
    }
    pub(crate) fn bind_basis(mut self) -> Self {
        self.basis_binding_attempt_count = 1;
        self
    }
    pub(crate) fn execute(mut self) -> Self {
        self.lower_runtime_execution_attempt_count = 1;
        self
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WorthQueryHistoricalNextAction {
    ReviseDeclaration,
    SupplyAvailableHistory,
    RefreshContext,
    ResolveAuthority,
    RetryRuntime,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WorthQueryHistoricalStopSource {
    Handle(worth_query_execution::facade::primary_graph::WorthQueryHandleDenial),
    HistoryUnavailable,
    StaleContext,
    ContextAdmission,
    Planning,
    BasisAdmission,
    Runtime,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorthQueryHistoricalStop {
    source: WorthQueryHistoricalStopSource,
    next_action: WorthQueryHistoricalNextAction,
    reason: String,
    counters: WorthQueryHistoricalJourneyCounters,
}

impl WorthQueryHistoricalStop {
    pub fn source(&self) -> WorthQueryHistoricalStopSource {
        self.source
    }
    pub fn next_action(&self) -> WorthQueryHistoricalNextAction {
        self.next_action
    }
    pub fn reason(&self) -> &str {
        &self.reason
    }
    pub fn journey_counters(&self) -> &WorthQueryHistoricalJourneyCounters {
        &self.counters
    }

    pub(crate) fn new(
        source: WorthQueryHistoricalStopSource,
        next_action: WorthQueryHistoricalNextAction,
        reason: impl Into<String>,
        counters: WorthQueryHistoricalJourneyCounters,
    ) -> Self {
        Self {
            source,
            next_action,
            reason: reason.into(),
            counters,
        }
    }
}

#[derive(Debug)]
pub struct WorthQueryHistoricalCompletion {
    result: WorthQueryReadResult,
    context_receipt: WorthQueryReadContextReceipt,
    materialization: HistoricalMaterializationPathMetadata,
    counters: WorthQueryHistoricalJourneyCounters,
}

impl WorthQueryHistoricalCompletion {
    pub fn result(&self) -> &WorthQueryReadResult {
        &self.result
    }
    pub fn context_receipt(&self) -> &WorthQueryReadContextReceipt {
        &self.context_receipt
    }
    pub fn materialization(&self) -> &HistoricalMaterializationPathMetadata {
        &self.materialization
    }
    pub fn journey_counters(&self) -> &WorthQueryHistoricalJourneyCounters {
        &self.counters
    }

    pub(crate) fn into_parts(
        self,
    ) -> (WorthQueryReadResult, HistoricalMaterializationPathMetadata) {
        (self.result, self.materialization)
    }

    pub(crate) fn new(
        result: WorthQueryReadResult,
        context_receipt: WorthQueryReadContextReceipt,
        materialization: HistoricalMaterializationPathMetadata,
        counters: WorthQueryHistoricalJourneyCounters,
    ) -> Self {
        Self {
            result,
            context_receipt,
            materialization,
            counters,
        }
    }
}

#[derive(Debug)]
pub enum WorthQueryHistoricalOutcome {
    Completed(WorthQueryHistoricalCompletion),
    Stopped(WorthQueryHistoricalStop),
}

impl WorthQueryHistoricalOutcome {
    pub fn completed(&self) -> Option<&WorthQueryHistoricalCompletion> {
        match self {
            Self::Completed(value) => Some(value),
            Self::Stopped(_) => None,
        }
    }
    pub fn stop(&self) -> Option<&WorthQueryHistoricalStop> {
        match self {
            Self::Stopped(value) => Some(value),
            Self::Completed(_) => None,
        }
    }
    pub fn into_result(self) -> Result<WorthQueryHistoricalCompletion, WorthQueryHistoricalStop> {
        match self {
            Self::Completed(value) => Ok(value),
            Self::Stopped(value) => Err(value),
        }
    }
}
