//! Recorded and requested meaning remain distinct on a refused opening.
use super::super::refusal::{
    WorthQueryApplicationHomeOpenRefused, WorthQueryApplicationHomeOpenRefusedPhase,
};
use super::refusal::{
    WorthQueryHomeProgramTransitionDenial as Denial,
    WorthQueryHomeProgramTransitionOpenPhase as Phase, WorthQueryHomeProgramTransitionOpenRefused,
};
use worth_foundational::facade::ApplicationHomeBinding;
use worth_query_declaration::facade::application_schema::ApplicationSchemaComposition;
use worth_query_execution::facade::{
    application_home_integration::{
        QueryHomeProgramActivationDenial, QueryHomeProgramTransitionSourceDenial,
        QueryHomeTransitionPreparationDenial, WorthQueryHomeProgramTransitionBodyDenial,
    },
    primary_graph::{
        WorthQueryPrimaryGraphInstallationDenialKind, WorthQueryProviderSessionDenialKind,
    },
};
use worth_relational::facade::{
    indexes::{DerivedIndexMaintenanceDenialKind, RelationalExecutionDenialCause},
    runtime::RelationalHomeProgramContractDenial,
    transactions::{CommitExecutionDenial, CommitExecutionDenialKind, TransactionCommitError},
};
use worth_store_application::{
    ApplicationHomeDescriptorDenial, ApplicationHomeOpenRefusal,
    ApplicationHomeTransitionIntentDenial, ApplicationHomeTransitionPlanningDenial,
    ApplicationHomeTransitionPlanningRequestDenial,
};

/// Descriptive revisions from the actual descriptor refusal. This value neither
/// admits a predecessor nor permits transition execution.
#[derive(Debug)]
pub struct WorthQueryApplicationHomeTransitionRequired<'refusal> {
    recorded: &'refusal ApplicationHomeBinding,
    requested: &'refusal ApplicationHomeBinding,
}
impl WorthQueryApplicationHomeTransitionRequired<'_> {
    pub fn recorded(&self) -> &ApplicationHomeBinding {
        self.recorded
    }
    pub fn requested(&self) -> &ApplicationHomeBinding {
        self.requested
    }
}

/// Typed classification for an opening refused because the recorded Home
/// binding does not match the transition's declared predecessor program.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WorthQueryHomePredecessorRefusal {
    /// The recorded Home program is not even rostered for the supplied predecessor.
    PredecessorNotRostered,
    /// The recorded Home binding differs from the supplied predecessor binding.
    PredecessorMismatch,
}

impl<Schema: ApplicationSchemaComposition, Program>
    WorthQueryApplicationHomeOpenRefused<Schema, Program>
{
    /// Inspect a transition requirement without importing the Store composition
    /// facade or interpreting a diagnostic string.
    pub fn transition_required(&self) -> Option<WorthQueryApplicationHomeTransitionRequired<'_>> {
        let WorthQueryApplicationHomeOpenRefusedPhase::Physical {
            cause:
                ApplicationHomeOpenRefusal::Descriptor(
                    ApplicationHomeDescriptorDenial::TransitionRequired {
                        recorded,
                        requested,
                        ..
                    },
                ),
            ..
        } = self.phase()
        else {
            return None;
        };
        Some(WorthQueryApplicationHomeTransitionRequired {
            recorded,
            requested,
        })
    }

    /// Inspect a predecessor-mismatch refusal from Store descriptor admission.
    pub fn predecessor_mismatch(&self) -> Option<WorthQueryHomePredecessorRefusal> {
        let WorthQueryApplicationHomeOpenRefusedPhase::Physical {
            cause: ApplicationHomeOpenRefusal::Descriptor(denial),
            ..
        } = self.phase()
        else {
            return None;
        };
        match denial {
            ApplicationHomeDescriptorDenial::TransitionPredecessorNotRostered => {
                Some(WorthQueryHomePredecessorRefusal::PredecessorNotRostered)
            }
            ApplicationHomeDescriptorDenial::TransitionPredecessorMismatch => {
                Some(WorthQueryHomePredecessorRefusal::PredecessorMismatch)
            }
            _ => None,
        }
    }

    /// Returns true when the opening was stopped by execution work/memory
    /// ceilings or cancellation before completing.
    pub fn is_budget_or_cancellation_stop(&self) -> bool {
        use WorthQueryApplicationHomeOpenRefusedPhase as OpenPhase;
        match self.phase() {
            OpenPhase::Physical { cause, .. } => open_refusal_is_budget_stop(cause),
            OpenPhase::NativeFinalizationInput { cause, .. } => open_refusal_is_budget_stop(cause),
            OpenPhase::WorldSourcePreparation { home, .. }
            | OpenPhase::WorldReadmission { home, .. }
            | OpenPhase::ExistingWorldSourcePreparation { home, .. }
            | OpenPhase::ExistingWorldReadmission { home, .. } => {
                open_refusal_is_budget_stop(home.reason())
            }
            OpenPhase::NativeReadmission { home, .. } => {
                matches!(
                    home.reason(),
                    worth_store_application::ApplicationHomeOpenRefusal::Execution(_)
                )
            }
            OpenPhase::NativeInstallation { query, .. } => {
                primary_graph_denial_is_budget_stop(query.reason().kind())
            }
            OpenPhase::NativeFinalization { query, .. } => {
                primary_graph_denial_is_budget_stop(query.reason().kind())
            }
            OpenPhase::ExistingNativeFinalization { query, .. } => {
                primary_graph_denial_is_budget_stop(query.reason().kind())
            }
            OpenPhase::BridgeDefinition { cause, .. }
            | OpenPhase::ExistingBridgeDefinition { cause, .. } => {
                primary_graph_denial_is_budget_stop(cause.kind())
            }
            _ => false,
        }
    }
}

impl<A: ApplicationSchemaComposition, AP, B: ApplicationSchemaComposition, BP>
    WorthQueryHomeProgramTransitionOpenRefused<A, AP, B, BP>
{
    /// Classify a predecessor-mismatch refusal without exposing Store internals.
    pub fn predecessor_mismatch(&self) -> Option<WorthQueryHomePredecessorRefusal> {
        match self.phase() {
            Phase::Successor(refused) => refused.predecessor_mismatch(),
            Phase::Predecessor { refused, .. } => refused.predecessor_mismatch(),
            _ => None,
        }
    }

    /// Returns the declared scan bound when a transition entity scan stopped
    /// because live rows exceeded `max_entities_per_kind`.
    pub fn entity_scan_limit_exceeded(&self) -> Option<usize> {
        match self.phase() {
            Phase::Transition {
                cause: Denial::Author(denial),
                ..
            } => match denial.kind() {
                WorthQueryPrimaryGraphInstallationDenialKind::TransitionEntityScanBoundExceeded {
                    maximum_work_units,
                } => Some(maximum_work_units),
                _ => None,
            },
            _ => None,
        }
    }

    /// Returns true when a budget, work ceiling, allocation limit, or
    /// cancellation stopped the transition before promotion.
    pub fn is_budget_or_cancellation_stop(&self) -> bool {
        match self.phase() {
            Phase::Successor(refused) => refused.is_budget_or_cancellation_stop(),
            Phase::Predecessor { refused, .. } => refused.is_budget_or_cancellation_stop(),
            Phase::BeforeCompilation { cause, .. }
            | Phase::Transition { cause, .. }
            | Phase::Configuration(cause) => transition_denial_is_budget_stop(cause),
            Phase::Compilation { successor, .. } => match successor.reason() {
                QueryHomeTransitionPreparationDenial::Compiler(cause) => {
                    primary_graph_denial_is_budget_stop(cause.kind())
                }
                QueryHomeTransitionPreparationDenial::Native(
                    RelationalHomeProgramContractDenial::Request(_),
                ) => true,
                _ => false,
            },
            Phase::Promotion { .. } => false,
        }
    }
}

fn open_refusal_is_budget_stop(cause: &ApplicationHomeOpenRefusal) -> bool {
    matches!(
        cause,
        ApplicationHomeOpenRefusal::Execution(_)
            | ApplicationHomeOpenRefusal::Descriptor(ApplicationHomeDescriptorDenial::Execution(_))
            | ApplicationHomeOpenRefusal::Transition(
                ApplicationHomeTransitionIntentDenial::Execution(_)
            )
    )
}

fn primary_graph_denial_is_budget_stop(kind: WorthQueryPrimaryGraphInstallationDenialKind) -> bool {
    matches!(
        kind,
        WorthQueryPrimaryGraphInstallationDenialKind::ExecutionDenied {
            kind: WorthQueryProviderSessionDenialKind::ExecutionResource { .. }
        } | WorthQueryPrimaryGraphInstallationDenialKind::ExecutionControlStopped { .. }
            | WorthQueryPrimaryGraphInstallationDenialKind::NativeAllocation { .. }
    )
}

pub(crate) fn relational_execution_denial_is_budget_stop(
    cause: RelationalExecutionDenialCause,
) -> bool {
    match cause {
        RelationalExecutionDenialCause::WorkerLimitExceedsParent
        | RelationalExecutionDenialCause::MemoryLimitExceedsParent
        | RelationalExecutionDenialCause::WorkLimitExceedsParent
        | RelationalExecutionDenialCause::PolicyMemoryExhausted { .. }
        | RelationalExecutionDenialCause::ProcessMemoryExhausted { .. }
        | RelationalExecutionDenialCause::DeclaredMemoryExhausted { .. }
        | RelationalExecutionDenialCause::ChargedBytesOverflow
        | RelationalExecutionDenialCause::Cancelled
        | RelationalExecutionDenialCause::DeadlineElapsed
        | RelationalExecutionDenialCause::WorkCounterOverflow
        | RelationalExecutionDenialCause::WorkExhausted
        | RelationalExecutionDenialCause::NestedStopped
        | RelationalExecutionDenialCause::ResultCapacityExceeded
        | RelationalExecutionDenialCause::AllocationFailed
        | RelationalExecutionDenialCause::ScratchCapacityExceeded
        | RelationalExecutionDenialCause::MemoryOverflow => true,
        RelationalExecutionDenialCause::HistoricalVisibility(_)
        | RelationalExecutionDenialCause::ProjectionContract(_)
        | RelationalExecutionDenialCause::StorageUnavailable(_)
        | RelationalExecutionDenialCause::UnrelatedNestedLease
        | RelationalExecutionDenialCause::NoActiveExecutionScope
        | RelationalExecutionDenialCause::EquivalenceContractUnavailable
        | RelationalExecutionDenialCause::AccountingUnavailable
        | RelationalExecutionDenialCause::WorkerFailed
        | RelationalExecutionDenialCause::UncheckedCustomKernel
        | RelationalExecutionDenialCause::ExpectedIdentitiesNotCanonical
        | RelationalExecutionDenialCause::ContentInconsistency
        | RelationalExecutionDenialCause::Content(_) => false,
    }
}

pub(crate) fn index_maintenance_denial_is_budget_stop(
    kind: &DerivedIndexMaintenanceDenialKind,
) -> bool {
    match kind {
        DerivedIndexMaintenanceDenialKind::WorkBudgetExceeded
        | DerivedIndexMaintenanceDenialKind::ColdRecordSlotBudgetExceeded => true,
        DerivedIndexMaintenanceDenialKind::Execution(cause) => {
            relational_execution_denial_is_budget_stop(*cause)
        }
        _ => false,
    }
}

pub(crate) fn commit_error_is_budget_stop(cause: &TransactionCommitError) -> bool {
    match cause {
        TransactionCommitError::Execution {
            denial:
                CommitExecutionDenial {
                    kind: CommitExecutionDenialKind::Cause(cause),
                    ..
                },
            ..
        } => relational_execution_denial_is_budget_stop(*cause),
        TransactionCommitError::Interrupted { .. } => true,
        TransactionCommitError::Publication { error, .. } => error
            .index_maintenance_denial()
            .is_some_and(|denial| index_maintenance_denial_is_budget_stop(&denial.kind)),
        TransactionCommitError::Conflict { .. }
        | TransactionCommitError::Preparation { .. }
        | TransactionCommitError::PublicationDenied { .. }
        | TransactionCommitError::PublicationDeferred { .. }
        | TransactionCommitError::PublicationFailed { .. }
        | TransactionCommitError::DurabilityUnresolved { .. } => false,
    }
}

pub(super) fn transition_denial_is_budget_stop(cause: &Denial) -> bool {
    match cause {
        Denial::Request(_)
        | Denial::StoppedOpen { .. }
        | Denial::StoppedPublication { .. }
        | Denial::StoppedContinuation { .. } => true,
        Denial::Indexes(denial) => index_maintenance_denial_is_budget_stop(&denial.kind),
        Denial::Commit(cause) => commit_error_is_budget_stop(cause),
        Denial::Author(denial) => primary_graph_denial_is_budget_stop(denial.kind()),
        Denial::AuthoredBody(WorthQueryHomeProgramTransitionBodyDenial::Request(_)) => true,
        Denial::PlanningRequest(
            ApplicationHomeTransitionPlanningRequestDenial::Execution(_)
            | ApplicationHomeTransitionPlanningRequestDenial::Transition(
                ApplicationHomeTransitionIntentDenial::Execution(_),
            )
            | ApplicationHomeTransitionPlanningRequestDenial::Native(
                RelationalHomeProgramContractDenial::Request(_),
            ),
        ) => true,
        Denial::Plan(
            ApplicationHomeTransitionPlanningDenial::Execution(_)
            | ApplicationHomeTransitionPlanningDenial::Intent(
                ApplicationHomeTransitionIntentDenial::Execution(_),
            ),
        ) => true,
        Denial::Intent(ApplicationHomeTransitionIntentDenial::Execution(_)) => true,
        Denial::Native(RelationalHomeProgramContractDenial::Request(_)) => true,
        Denial::Source(QueryHomeProgramTransitionSourceDenial::Native(
            RelationalHomeProgramContractDenial::Request(_),
        )) => true,
        Denial::Source(QueryHomeProgramTransitionSourceDenial::Meaning(denial)) => {
            primary_graph_denial_is_budget_stop(denial.kind())
        }
        Denial::Source(QueryHomeProgramTransitionSourceDenial::Activation(
            QueryHomeProgramActivationDenial::Execution(_),
        )) => true,
        Denial::Source(QueryHomeProgramTransitionSourceDenial::Activation(
            QueryHomeProgramActivationDenial::Native(denial),
        )) => primary_graph_denial_is_budget_stop(denial.kind()),
        _ => false,
    }
}

#[cfg(test)]
#[path = "required_tests.rs"]
mod tests;
