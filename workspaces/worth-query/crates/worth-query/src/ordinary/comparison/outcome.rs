use crate::correspondence::CorrespondenceEvidenceResolved;
use crate::memory_workspace::WorthQueryEntityIdentity;
use crate::runtime::WorthQueryReadResult;

use super::{
    WorthQueryComparisonBasisFamily, WorthQueryComparisonBasisPairEvidence,
    WorthQueryComparisonCostClass, WorthQueryComparisonRowChange,
};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WorthQueryComparisonChange {
    Unchanged,
    Changed,
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct WorthQueryComparisonJourneyCounters {
    basis_pair_validation_count: usize,
    left_execution_attempt_count: usize,
    right_execution_attempt_count: usize,
    historical_materialization_attempt_count: usize,
    correspondence_resolution_attempt_count: usize,
    left_row_scan_count: usize,
    right_row_scan_count: usize,
    emitted_row_change_count: usize,
}

impl WorthQueryComparisonJourneyCounters {
    pub fn basis_pair_validation_count(&self) -> usize {
        self.basis_pair_validation_count
    }
    pub fn left_execution_attempt_count(&self) -> usize {
        self.left_execution_attempt_count
    }
    pub fn right_execution_attempt_count(&self) -> usize {
        self.right_execution_attempt_count
    }
    pub fn historical_materialization_attempt_count(&self) -> usize {
        self.historical_materialization_attempt_count
    }
    pub fn correspondence_resolution_attempt_count(&self) -> usize {
        self.correspondence_resolution_attempt_count
    }
    pub fn left_row_scan_count(&self) -> usize {
        self.left_row_scan_count
    }
    pub fn right_row_scan_count(&self) -> usize {
        self.right_row_scan_count
    }
    pub fn emitted_row_change_count(&self) -> usize {
        self.emitted_row_change_count
    }
    pub(crate) fn validate_pair() -> Self {
        Self {
            basis_pair_validation_count: 1,
            ..Self::default()
        }
    }
    pub(crate) fn execute_left(mut self) -> Self {
        self.left_execution_attempt_count = 1;
        self
    }
    pub(crate) fn execute_right(mut self) -> Self {
        self.right_execution_attempt_count = 1;
        self
    }
    pub(crate) fn materialize_historical(mut self) -> Self {
        self.historical_materialization_attempt_count = 1;
        self
    }
    pub(crate) fn resolve_correspondence(mut self) -> Self {
        self.correspondence_resolution_attempt_count = 1;
        self
    }
    pub(crate) fn record_diff_breadth(
        mut self,
        left_rows: usize,
        right_rows: usize,
        emitted_changes: usize,
    ) -> Self {
        self.left_row_scan_count = left_rows;
        self.right_row_scan_count = right_rows;
        self.emitted_row_change_count = emitted_changes;
        self
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WorthQueryComparisonNextAction {
    ReviseDeclaration,
    RefreshBasisPair,
    ResolveAuthority,
    NarrowCandidates,
    RetryRuntime,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WorthQueryComparisonStopSource {
    Handle(worth_query_execution::facade::primary_graph::WorthQueryHandleDenial),
    LeftBasisAdmission,
    RightBasisAdmission,
    InvalidBasisPair,
    StaleBasisPair,
    LeftExecution,
    RightExecution,
    ComparisonAssembly,
    CorrespondenceDenied,
}

#[derive(Debug)]
pub struct WorthQueryComparisonStop {
    source: WorthQueryComparisonStopSource,
    next_action: WorthQueryComparisonNextAction,
    reason: String,
    counters: WorthQueryComparisonJourneyCounters,
}

impl WorthQueryComparisonStop {
    pub fn source(&self) -> WorthQueryComparisonStopSource {
        self.source
    }
    pub fn next_action(&self) -> WorthQueryComparisonNextAction {
        self.next_action
    }
    pub fn reason(&self) -> &str {
        &self.reason
    }
    pub fn journey_counters(&self) -> &WorthQueryComparisonJourneyCounters {
        &self.counters
    }
    pub(crate) fn new(
        source: WorthQueryComparisonStopSource,
        next_action: WorthQueryComparisonNextAction,
        reason: impl Into<String>,
        counters: WorthQueryComparisonJourneyCounters,
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
pub struct WorthQueryComparisonCompletion {
    left: WorthQueryReadResult,
    right: WorthQueryReadResult,
    basis_pair: WorthQueryComparisonBasisPairEvidence,
    row_changes: Vec<WorthQueryComparisonRowChange>,
    change: WorthQueryComparisonChange,
    counters: WorthQueryComparisonJourneyCounters,
}

impl WorthQueryComparisonCompletion {
    pub fn left(&self) -> &WorthQueryReadResult {
        &self.left
    }
    pub fn right(&self) -> &WorthQueryReadResult {
        &self.right
    }
    pub fn basis_pair(&self) -> &WorthQueryComparisonBasisPairEvidence {
        &self.basis_pair
    }
    pub fn row_changes(&self) -> &[WorthQueryComparisonRowChange] {
        &self.row_changes
    }
    pub fn change(&self) -> WorthQueryComparisonChange {
        self.change
    }
    pub fn cost_class(&self) -> WorthQueryComparisonCostClass {
        match self.basis_pair.family() {
            WorthQueryComparisonBasisFamily::CurrentToHistorical => {
                WorthQueryComparisonCostClass::CurrentAndRetainedMaterialization
            }
            WorthQueryComparisonBasisFamily::BranchToBranch => {
                WorthQueryComparisonCostClass::DeterministicIdentityIndexBuildAndMerge
            }
        }
    }
    pub fn journey_counters(&self) -> &WorthQueryComparisonJourneyCounters {
        &self.counters
    }
    pub(crate) fn new(
        left: WorthQueryReadResult,
        right: WorthQueryReadResult,
        basis_pair: WorthQueryComparisonBasisPairEvidence,
        row_changes: Vec<WorthQueryComparisonRowChange>,
        change: WorthQueryComparisonChange,
        counters: WorthQueryComparisonJourneyCounters,
    ) -> Self {
        Self {
            left,
            right,
            basis_pair,
            row_changes,
            change,
            counters,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WorthQueryComparisonCorrespondencePosture {
    AuthoritativeContinuity,
    Advisory,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorthQueryComparisonCorrespondence {
    subject: WorthQueryEntityIdentity,
    correspondence: CorrespondenceEvidenceResolved,
    posture: WorthQueryComparisonCorrespondencePosture,
    basis_pair: WorthQueryComparisonBasisPairEvidence,
    counters: WorthQueryComparisonJourneyCounters,
}

impl WorthQueryComparisonCorrespondence {
    pub fn subject(&self) -> &WorthQueryEntityIdentity {
        &self.subject
    }
    pub fn correspondence(&self) -> &CorrespondenceEvidenceResolved {
        &self.correspondence
    }
    pub fn posture(&self) -> WorthQueryComparisonCorrespondencePosture {
        self.posture
    }
    pub fn basis_pair(&self) -> &WorthQueryComparisonBasisPairEvidence {
        &self.basis_pair
    }
    pub fn journey_counters(&self) -> &WorthQueryComparisonJourneyCounters {
        &self.counters
    }
    pub(crate) fn new(
        subject: WorthQueryEntityIdentity,
        correspondence: CorrespondenceEvidenceResolved,
        posture: WorthQueryComparisonCorrespondencePosture,
        basis_pair: WorthQueryComparisonBasisPairEvidence,
        counters: WorthQueryComparisonJourneyCounters,
    ) -> Self {
        Self {
            subject,
            correspondence,
            posture,
            basis_pair,
            counters,
        }
    }
}

#[derive(Debug)]
pub enum WorthQueryComparisonOutcome {
    Completed(WorthQueryComparisonCompletion),
    Correspondence(WorthQueryComparisonCorrespondence),
    Stopped(WorthQueryComparisonStop),
}

impl WorthQueryComparisonOutcome {
    pub fn completed(&self) -> Option<&WorthQueryComparisonCompletion> {
        match self {
            Self::Completed(value) => Some(value),
            _ => None,
        }
    }
    pub fn correspondence(&self) -> Option<&WorthQueryComparisonCorrespondence> {
        match self {
            Self::Correspondence(value) => Some(value),
            _ => None,
        }
    }
    pub fn stop(&self) -> Option<&WorthQueryComparisonStop> {
        match self {
            Self::Stopped(value) => Some(value),
            _ => None,
        }
    }
}
