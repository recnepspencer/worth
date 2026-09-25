use worth_query_execution::facade::primary_graph::{
    WorthQueryOutputDemandDenial, WorthQueryOutputDemandRecoveryPosture,
};

use crate::application_entry::WorthQueryApplicationOutputDemandDenial;

use super::WorthQueryRequiredOutputPreparationDenial;

mod query;
use query::query_posture;

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
            | Denial::ReadObservation(_)
            | Denial::Connection(_)
            | Denial::Closed => WorthQueryRequiredOutputRecoveryPosture::Terminal,
            Denial::SourceQuery(query) => query_posture(query),
            Denial::DemandExecution(denial) => output_posture(denial),
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
        Denial::Demand(denial) => output_posture(denial),
        Denial::MissingSource
        | Denial::FreshRequestMismatch
        | Denial::ProgramOutputUndeclared
        | Denial::Superseded
        | Denial::Closed => WorthQueryRequiredOutputRecoveryPosture::Terminal,
    }
}

fn output_posture(
    denial: &WorthQueryOutputDemandDenial,
) -> WorthQueryRequiredOutputRecoveryPosture {
    match denial.recovery_posture() {
        WorthQueryOutputDemandRecoveryPosture::Retryable => {
            WorthQueryRequiredOutputRecoveryPosture::Retryable
        }
        WorthQueryOutputDemandRecoveryPosture::Terminal => {
            WorthQueryRequiredOutputRecoveryPosture::Terminal
        }
    }
}
