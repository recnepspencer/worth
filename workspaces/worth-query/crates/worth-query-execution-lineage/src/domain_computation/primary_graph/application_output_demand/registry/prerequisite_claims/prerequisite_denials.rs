use crate::domain_computation::primary_graph::{
    WorthQueryOutputDemandDenial, WorthQueryOutputDemandDenialKind,
    WorthQueryOutputDemandRecoveryPosture,
};

pub fn capacity_denial() -> WorthQueryOutputDemandDenial {
    WorthQueryOutputDemandDenial::new(
        WorthQueryOutputDemandDenialKind::RetentionBudgetExceeded,
        "prerequisite registry custody capacity is exhausted",
    )
}

pub fn work_denial() -> WorthQueryOutputDemandDenial {
    WorthQueryOutputDemandDenial::new(
        WorthQueryOutputDemandDenialKind::WorkBudgetExceeded,
        "prerequisite registry lookup exceeds request work",
    )
}

pub fn closed_denial() -> WorthQueryOutputDemandDenial {
    WorthQueryOutputDemandDenial::new(
        WorthQueryOutputDemandDenialKind::Closed,
        "required output closed",
    )
}

pub fn coverage_denial() -> WorthQueryOutputDemandDenial {
    WorthQueryOutputDemandDenial::new(
        WorthQueryOutputDemandDenialKind::IncompleteDependencyCoverage,
        "an exact consumed upstream settlement has no managed ready output",
    )
}

/// An exact consumed upstream was retired by a newer settlement of its demand
/// after the execution read it. The output is stale, not uncovered: a retry
/// reads the current upstream row.
pub fn stale_upstream_denial() -> WorthQueryOutputDemandDenial {
    WorthQueryOutputDemandDenial::new(
        WorthQueryOutputDemandDenialKind::PublicationStale,
        "an exact consumed upstream settlement was superseded before publication",
    )
    .with_recovery_posture(WorthQueryOutputDemandRecoveryPosture::Retryable)
}
