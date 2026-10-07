use super::{comparison, count, history, inspection, live, mutation, preview, read, workflow};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WorthQueryOutcomePosture {
    Completed,
    Advisory,
    Violation,
    Deferred,
    Unavailable,
}

/// Shared structural navigation for ordinary outcomes.
///
/// The trait deliberately does not erase family payloads. Consumers use this
/// vocabulary to branch on structural fate, then use the concrete outcome's
/// own accessors for receipts, results, stops, and next actions.
pub trait WorthQueryOutcomeNavigation {
    fn posture(&self) -> WorthQueryOutcomePosture;

    fn is_completed(&self) -> bool {
        self.posture() == WorthQueryOutcomePosture::Completed
    }

    fn is_advisory(&self) -> bool {
        self.posture() == WorthQueryOutcomePosture::Advisory
    }

    fn is_violation(&self) -> bool {
        self.posture() == WorthQueryOutcomePosture::Violation
    }

    fn is_deferred(&self) -> bool {
        self.posture() == WorthQueryOutcomePosture::Deferred
    }

    fn is_unavailable(&self) -> bool {
        self.posture() == WorthQueryOutcomePosture::Unavailable
    }
}

impl WorthQueryOutcomeNavigation for read::WorthQueryReadOutcome {
    fn posture(&self) -> WorthQueryOutcomePosture {
        match self {
            Self::Completed(_) => WorthQueryOutcomePosture::Completed,
            Self::Stopped(stop) => read_stop_posture(stop),
        }
    }
}

impl WorthQueryOutcomeNavigation for read::WorthQueryProjectionOutcome {
    fn posture(&self) -> WorthQueryOutcomePosture {
        match self {
            Self::Completed(_) => WorthQueryOutcomePosture::Completed,
            Self::Advisory(_) => WorthQueryOutcomePosture::Advisory,
            Self::Violation(_) => WorthQueryOutcomePosture::Violation,
            Self::Deferred(_) => WorthQueryOutcomePosture::Deferred,
            Self::Unavailable(_) => WorthQueryOutcomePosture::Unavailable,
        }
    }
}

impl WorthQueryOutcomeNavigation for count::WorthQueryCountOutcome {
    fn posture(&self) -> WorthQueryOutcomePosture {
        match self {
            Self::Completed(_) => WorthQueryOutcomePosture::Completed,
            Self::Stopped(stop) => read_stop_posture(stop),
        }
    }
}

impl WorthQueryOutcomeNavigation for live::WorthQueryLiveOpenOutcome {
    fn posture(&self) -> WorthQueryOutcomePosture {
        match self {
            Self::Opened(_) => WorthQueryOutcomePosture::Completed,
            Self::Stopped(stop) => read_stop_posture(stop.read_stop()),
        }
    }
}

impl WorthQueryOutcomeNavigation for history::WorthQueryHistoricalOutcome {
    fn posture(&self) -> WorthQueryOutcomePosture {
        match self {
            Self::Completed(_) => WorthQueryOutcomePosture::Completed,
            Self::Stopped(stop) => match stop.source() {
                history::WorthQueryHistoricalStopSource::Handle(_)
                | history::WorthQueryHistoricalStopSource::HistoryUnavailable
                | history::WorthQueryHistoricalStopSource::Runtime => {
                    WorthQueryOutcomePosture::Unavailable
                }
                history::WorthQueryHistoricalStopSource::StaleContext
                | history::WorthQueryHistoricalStopSource::ContextAdmission
                | history::WorthQueryHistoricalStopSource::Planning
                | history::WorthQueryHistoricalStopSource::BasisAdmission => {
                    WorthQueryOutcomePosture::Violation
                }
            },
        }
    }
}

impl WorthQueryOutcomeNavigation for comparison::WorthQueryComparisonOutcome {
    fn posture(&self) -> WorthQueryOutcomePosture {
        match self {
            Self::Completed(_) => WorthQueryOutcomePosture::Completed,
            Self::Correspondence(correspondence) => match correspondence.posture() {
                comparison::WorthQueryComparisonCorrespondencePosture::AuthoritativeContinuity => {
                    WorthQueryOutcomePosture::Completed
                }
                comparison::WorthQueryComparisonCorrespondencePosture::Advisory => {
                    WorthQueryOutcomePosture::Advisory
                }
            },
            Self::Stopped(stop) => match stop.source() {
                comparison::WorthQueryComparisonStopSource::Handle(_)
                | comparison::WorthQueryComparisonStopSource::LeftExecution
                | comparison::WorthQueryComparisonStopSource::RightExecution => {
                    WorthQueryOutcomePosture::Unavailable
                }
                comparison::WorthQueryComparisonStopSource::LeftBasisAdmission
                | comparison::WorthQueryComparisonStopSource::RightBasisAdmission
                | comparison::WorthQueryComparisonStopSource::InvalidBasisPair
                | comparison::WorthQueryComparisonStopSource::StaleBasisPair
                | comparison::WorthQueryComparisonStopSource::ComparisonAssembly
                | comparison::WorthQueryComparisonStopSource::CorrespondenceDenied => {
                    WorthQueryOutcomePosture::Violation
                }
            },
        }
    }
}

impl WorthQueryOutcomeNavigation for mutation::WorthQueryMutationOutcome {
    fn posture(&self) -> WorthQueryOutcomePosture {
        match self {
            Self::Completed(_) => WorthQueryOutcomePosture::Completed,
            Self::Stopped(stop) => match stop.source() {
                mutation::WorthQueryMutationStopSource::InspectionUnavailable
                | mutation::WorthQueryMutationStopSource::LowerRuntime => {
                    WorthQueryOutcomePosture::Unavailable
                }
                mutation::WorthQueryMutationStopSource::ForeignAuthority
                | mutation::WorthQueryMutationStopSource::StaleBasis => {
                    WorthQueryOutcomePosture::Violation
                }
            },
        }
    }
}

impl WorthQueryOutcomeNavigation for preview::WorthQueryPreviewJourneyOutcome {
    fn posture(&self) -> WorthQueryOutcomePosture {
        match self {
            Self::ReadOnlyCompleted(_) => WorthQueryOutcomePosture::Completed,
            Self::PromotionCompleted(completion) => workflow_completion_posture(completion),
            Self::Stopped(stop) => workflow_stop_posture(stop),
        }
    }
}

impl WorthQueryOutcomeNavigation for workflow::WorthQueryWorkflowOutcome {
    fn posture(&self) -> WorthQueryOutcomePosture {
        match self {
            Self::Completed(completion) => workflow_completion_posture(completion),
            Self::Stopped(stop) => workflow_stop_posture(stop),
        }
    }
}

impl WorthQueryOutcomeNavigation for workflow::WorthQueryWritebackOutcome {
    fn posture(&self) -> WorthQueryOutcomePosture {
        match self {
            Self::Completed(_) => WorthQueryOutcomePosture::Completed,
            Self::Stopped(stop) => match stop.source() {
                workflow::WorthQueryWritebackStopSource::Handle(_)
                | workflow::WorthQueryWritebackStopSource::InspectionUnavailable
                | workflow::WorthQueryWritebackStopSource::BridgeExecution => {
                    WorthQueryOutcomePosture::Unavailable
                }
                workflow::WorthQueryWritebackStopSource::ForeignAuthority
                | workflow::WorthQueryWritebackStopSource::StaleAuthority
                | workflow::WorthQueryWritebackStopSource::Basis
                | workflow::WorthQueryWritebackStopSource::Intent
                | workflow::WorthQueryWritebackStopSource::Eligibility
                | workflow::WorthQueryWritebackStopSource::Lowering => {
                    WorthQueryOutcomePosture::Violation
                }
            },
        }
    }
}

impl WorthQueryOutcomeNavigation for workflow::WorthQueryBranchMergeOutcome {
    fn posture(&self) -> WorthQueryOutcomePosture {
        match self {
            Self::Completed(_) => WorthQueryOutcomePosture::Completed,
            Self::Deferred(_) | Self::SettlementDeferred(_) => WorthQueryOutcomePosture::Deferred,
            Self::ControlStopped(_) => WorthQueryOutcomePosture::Unavailable,
            Self::Stopped(stop) => match stop.source() {
                workflow::WorthQueryBranchMergeStopSource::InspectionUnavailable
                | workflow::WorthQueryBranchMergeStopSource::RelationalExecution => {
                    WorthQueryOutcomePosture::Unavailable
                }
                workflow::WorthQueryBranchMergeStopSource::ForeignAuthority
                | workflow::WorthQueryBranchMergeStopSource::StaleAuthority
                | workflow::WorthQueryBranchMergeStopSource::MismatchedContext
                | workflow::WorthQueryBranchMergeStopSource::Basis
                | workflow::WorthQueryBranchMergeStopSource::Intent
                | workflow::WorthQueryBranchMergeStopSource::Eligibility
                | workflow::WorthQueryBranchMergeStopSource::Lowering => {
                    WorthQueryOutcomePosture::Violation
                }
            },
        }
    }
}

impl WorthQueryOutcomeNavigation
    for crate::domain_installation::WorthQueryInstalledDomainWorkflowOutcome
{
    fn posture(&self) -> WorthQueryOutcomePosture {
        match self {
            Self::Completed(completion) => workflow_completion_posture(completion.completion()),
            Self::Stopped(stop) => workflow_stop_posture(stop.stop()),
        }
    }
}

impl WorthQueryOutcomeNavigation for inspection::WorthQueryInspectionOutcome {
    fn posture(&self) -> WorthQueryOutcomePosture {
        match self {
            Self::Completed(_) => WorthQueryOutcomePosture::Completed,
            Self::Advisory(_) => WorthQueryOutcomePosture::Advisory,
            Self::Violation(_) | Self::Stopped(_) => WorthQueryOutcomePosture::Violation,
            Self::Unavailable(_) => WorthQueryOutcomePosture::Unavailable,
        }
    }
}

fn read_stop_posture(stop: &read::WorthQueryReadStop) -> WorthQueryOutcomePosture {
    match stop.source() {
        read::WorthQueryReadStopSource::Runtime(_) => WorthQueryOutcomePosture::Unavailable,
        read::WorthQueryReadStopSource::Context(_)
        | read::WorthQueryReadStopSource::Planning(_) => WorthQueryOutcomePosture::Violation,
    }
}

fn workflow_completion_posture(
    completion: &workflow::WorthQueryWorkflowCompletion,
) -> WorthQueryOutcomePosture {
    if completion.advisories().is_empty() {
        WorthQueryOutcomePosture::Completed
    } else {
        WorthQueryOutcomePosture::Advisory
    }
}

fn workflow_stop_posture(stop: &workflow::WorthQueryWorkflowStop) -> WorthQueryOutcomePosture {
    match stop.source() {
        workflow::WorthQueryWorkflowStopSource::InspectionUnavailable
        | workflow::WorthQueryWorkflowStopSource::LowerRuntime => {
            WorthQueryOutcomePosture::Unavailable
        }
        workflow::WorthQueryWorkflowStopSource::CrossSession
        | workflow::WorthQueryWorkflowStopSource::ForeignAuthority
        | workflow::WorthQueryWorkflowStopSource::StalePreview => {
            WorthQueryOutcomePosture::Violation
        }
    }
}
