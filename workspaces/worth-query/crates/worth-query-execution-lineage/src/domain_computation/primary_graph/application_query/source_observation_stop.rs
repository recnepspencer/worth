//! Pure source-query observation stop enums.

use super::{
    WorthQueryAdmittedOneShotStop, WorthQueryApplicationQueryAdmissionDenial,
    WorthQuerySourceExpectationDenial,
};
use std::sync::Arc;

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum SourceQueryObservationStop {
    CurrentControl(crate::domain_computation::primary_graph::WorthQueryOutputDemandDenial),
    ManagedIndex(Arc<super::managed_index::WorthQueryManagedIndexDenial>),
    Installation(worth_query_installation::facade::WorthQueryApplicationQueryInstallationDenial),
    Principal(crate::domain_computation::primary_graph::WorthQueryPrincipalResolutionDenial),
    Scope(crate::domain_computation::primary_graph::WorthQueryEntityResolutionDenial),
    Admission(WorthQueryApplicationQueryAdmissionDenial),
    Preparation {
        phase: SourceQueryPreparationPhase,
        cause: WorthQueryApplicationQueryAdmissionDenial,
    },
    Read(WorthQueryAdmittedOneShotStop),
    Basis(crate::domain_computation::primary_graph::product_operation::SelectedQueryBasisRetentionStop),
    Missing,
    Ambiguous,
    ForeignClosure,
    SourceFacts(WorthQuerySourceExpectationDenial),
}

impl From<WorthQuerySourceExpectationDenial> for SourceQueryObservationStop {
    fn from(cause: WorthQuerySourceExpectationDenial) -> Self {
        Self::SourceFacts(cause)
    }
}

impl SourceQueryObservationStop {
    /// Authorized model-result outcomes retain the handler's existing semantics.
    pub fn is_control_refusal(&self) -> bool {
        use crate::domain_computation::primary_graph::WorthQueryApplicationOneShotDenialKind as Read;
        use crate::domain_computation::primary_graph::WorthQueryApplicationQueryAdmissionDenialKind as Admission;
        use SourceQueryObservationStop as Stop;
        match self {
            Stop::CurrentControl(_)
            | Stop::ManagedIndex(_)
            | Stop::Principal(_)
            | Stop::Scope(_)
            | Stop::Basis(_) => true,
            Stop::Admission(stop) | Stop::Preparation { cause: stop, .. } => matches!(
                stop.kind(),
                Admission::Authorization(_)
                    | Admission::StalePrincipal
                    | Admission::StaleScope
                    | Admission::HistoricalSourceNotLive { .. }
                    | Admission::HistoricalSourceKindMismatch { .. }
                    | Admission::HistoricalSourceOutsideDomain { .. }
                    | Admission::HistoricalSourceOwnershipUnavailable { .. }
                    | Admission::ForeignPrincipal
                    | Admission::ForeignScope
                    | Admission::StaleBasis
                    | Admission::Cancelled
                    | Admission::DeadlineExceeded
                    | Admission::ExecutionRequest(_)
                    | Admission::ActiveSnapshotCapacityExhausted { .. }
                    | Admission::SnapshotIdentityExhausted
                    | Admission::RetentionCapacityExhausted
                    | Admission::RetentionIdentityExhausted
                    | Admission::ReadmissionPreparationMemoryExhausted
                    | Admission::WorkLimitExceeded
                    | Admission::CanonicalWorkDenied
                    | Admission::GraphWorkAdmissionUnavailable
                    | Admission::DisclosureGovernanceRequired
                    | Admission::DisclosureAuthorizationMismatch
                    | Admission::InternalComputationDenied
            ),
            Stop::Read(WorthQueryAdmittedOneShotStop::Execution(stop)) => matches!(
                stop.kind(),
                Read::Authorization(_)
                    | Read::StalePrincipal
                    | Read::StaleScope
                    | Read::Cancelled
                    | Read::DeadlineExceeded
                    | Read::ExecutionRequest(_)
                    | Read::ActiveSnapshotCapacityExhausted { .. }
                    | Read::SnapshotIdentityExhausted
                    | Read::RetentionCapacityExhausted
                    | Read::RetentionIdentityExhausted
                    | Read::WorkLimitExceeded
                    | Read::ResultBufferLimitExceeded
                    | Read::SourceIdentityExhausted
                    | Read::SourceObservationAllocation { .. }
            ),
            Stop::Read(_) => true,
            Stop::SourceFacts(stop) => {
                stop.execution_denial().is_some() || stop.retention_interruption().is_some()
            }
            Stop::Missing | Stop::Ambiguous | Stop::ForeignClosure | Stop::Installation(_) => false,
        }
    }

    #[doc(hidden)]
    pub fn is_control_refusal_for_test(&self) -> bool {
        self.is_control_refusal()
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SourceQueryPreparationPhase {
    Parameters,
    Permission,
    Session,
}
