use worth_query_execution::facade::primary_graph::WorthQueryOutputDemandDenialKind;

use crate::application_entry::{
    WorthQueryApplicationOutputDemandDenial, WorthQueryApplicationRequestQueryDenial,
};

use super::WorthQueryRequiredOutputPreparationDenial;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WorthQueryRequiredOutputRecoveryPosture {
    Retryable,
    Terminal,
}

impl WorthQueryRequiredOutputPreparationDenial {
    /// Whether a fresh request can still use the same committed source receipt.
    /// The receipt never grants authority by itself; recovery still rechecks its
    /// exact installed program, root, branch, producer, and current source.
    pub fn recovery_posture(&self) -> WorthQueryRequiredOutputRecoveryPosture {
        use WorthQueryRequiredOutputPreparationDenial as Denial;
        match self {
            Denial::ForeignProgram
            | Denial::UndeclaredOutputRoot
            | Denial::MissingConnection
            | Denial::MissingPerformedDelivery
            | Denial::MissingSource
            | Denial::Connection(_)
            | Denial::Closed => WorthQueryRequiredOutputRecoveryPosture::Terminal,
            Denial::SourceQuery(query) => query_posture(query),
            Denial::DemandExecution(denial) => output_posture(denial.kind()),
            Denial::Demand(denial) => demand_posture(denial),
        }
    }
}

fn demand_posture(
    denial: &WorthQueryApplicationOutputDemandDenial,
) -> WorthQueryRequiredOutputRecoveryPosture {
    use WorthQueryApplicationOutputDemandDenial as Denial;
    match denial {
        Denial::Source(query) => query_posture(query),
        Denial::Demand(denial) => output_posture(denial.kind()),
        Denial::MissingSource
        | Denial::FreshRequestMismatch
        | Denial::Superseded
        | Denial::Closed => WorthQueryRequiredOutputRecoveryPosture::Terminal,
    }
}

fn query_posture(
    denial: &WorthQueryApplicationRequestQueryDenial,
) -> WorthQueryRequiredOutputRecoveryPosture {
    use WorthQueryApplicationRequestQueryDenial as Denial;
    match denial {
        Denial::OutputSettlement(output) => output_posture(output.kind()),
        Denial::BindingInstallation(_)
        | Denial::ProductSelection(_)
        | Denial::PrincipalResolution(_)
        | Denial::ScopeResolution(_) => WorthQueryRequiredOutputRecoveryPosture::Terminal,
        Denial::Limit(_) | Denial::Admission(_) | Denial::Execution(_) => {
            WorthQueryRequiredOutputRecoveryPosture::Retryable
        }
    }
}

fn output_posture(
    kind: WorthQueryOutputDemandDenialKind,
) -> WorthQueryRequiredOutputRecoveryPosture {
    use WorthQueryOutputDemandDenialKind as Kind;
    match kind {
        Kind::SchedulingRejected
        | Kind::SchedulingDeferred
        | Kind::Cancelled
        | Kind::TimedOut
        | Kind::WorkBudgetExceeded
        | Kind::RetentionBudgetExceeded
        | Kind::PublicationCapacityExceeded => WorthQueryRequiredOutputRecoveryPosture::Retryable,
        Kind::ForeignSource
        | Kind::MissingApplicableProducer
        | Kind::AmbiguousApplicableProducer
        | Kind::ProducerUnavailable
        | Kind::PublicationStale
        | Kind::NoEffect
        | Kind::Superseded
        | Kind::ForeignDemand
        | Kind::ForeignSettlement
        | Kind::RetainedBasisUnavailable
        | Kind::Closed
        | Kind::DuplicatePerformedSource => WorthQueryRequiredOutputRecoveryPosture::Terminal,
    }
}
