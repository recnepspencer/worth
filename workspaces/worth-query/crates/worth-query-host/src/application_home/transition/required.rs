//! Recorded and requested meaning remain distinct on a refused opening.
use super::super::refusal::WorthQueryApplicationHomeOpenRefused;
use super::refusal::{
    WorthQueryHomeProgramTransitionDenial as Denial,
    WorthQueryHomeProgramTransitionOpenPhase as Phase, WorthQueryHomeProgramTransitionOpenRefused,
};
use worth_foundational::facade::ApplicationHomeBinding;
use worth_query_declaration::facade::application_schema::ApplicationSchemaComposition;
use worth_query_execution::facade::{
    application_home_integration::{
        QueryHomeProgramTransitionSourceDenial, QueryHomeTransitionPreparationDenial,
        WorthQueryHomeProgramTransitionBodyDenial,
    },
    primary_graph::{
        WorthQueryPrimaryGraphInstallationDenialKind, WorthQueryProviderSessionDenialKind,
    },
};
use worth_relational::facade::{
    indexes::DerivedIndexMaintenanceDenialKind, runtime::RelationalHomeProgramContractDenial,
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
        let Self::Physical {
            cause:
                ApplicationHomeOpenRefusal::Descriptor(
                    ApplicationHomeDescriptorDenial::TransitionRequired {
                        recorded,
                        requested,
                        ..
                    },
                ),
            ..
        } = self
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
        let Self::Physical {
            cause: ApplicationHomeOpenRefusal::Descriptor(denial),
            ..
        } = self
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
        match self {
            Self::Physical { cause, .. } => open_refusal_is_budget_stop(cause),
            Self::NativeFinalizationInput { cause, .. } => open_refusal_is_budget_stop(cause),
            Self::WorldSourcePreparation { home, .. }
            | Self::WorldReadmission { home, .. }
            | Self::ExistingWorldSourcePreparation { home, .. }
            | Self::ExistingWorldReadmission { home, .. } => {
                open_refusal_is_budget_stop(home.reason())
            }
            Self::NativeReadmission { home, .. } => {
                matches!(
                    home.reason(),
                    worth_store_application::ApplicationHomeOpenRefusal::Execution(_)
                )
            }
            Self::NativeInstallation { query, .. } => {
                primary_graph_denial_is_budget_stop(query.reason().kind())
            }
            Self::NativeFinalization { query, .. } => {
                primary_graph_denial_is_budget_stop(query.reason().kind())
            }
            Self::ExistingNativeFinalization { query, .. } => {
                primary_graph_denial_is_budget_stop(query.reason().kind())
            }
            Self::BridgeDefinition { cause, .. } | Self::ExistingBridgeDefinition { cause, .. } => {
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

pub(crate) fn index_maintenance_denial_is_budget_stop(
    kind: &DerivedIndexMaintenanceDenialKind,
) -> bool {
    matches!(
        kind,
        DerivedIndexMaintenanceDenialKind::WorkBudgetExceeded
            | DerivedIndexMaintenanceDenialKind::Execution(_)
    )
}

fn transition_denial_is_budget_stop(cause: &Denial) -> bool {
    match cause {
        Denial::Request(_)
        | Denial::StoppedOpen { .. }
        | Denial::StoppedPublication { .. }
        | Denial::StoppedContinuation { .. } => true,
        Denial::Indexes(denial) => index_maintenance_denial_is_budget_stop(&denial.kind),
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
        _ => false,
    }
}

#[cfg(test)]
mod tests {
    use super::index_maintenance_denial_is_budget_stop;
    use worth_relational::facade::indexes::{DerivedIndexId, DerivedIndexMaintenanceDenialKind};

    #[test]
    fn index_denial_classifier_separates_budget_stops_from_structural_refusals() {
        assert!(index_maintenance_denial_is_budget_stop(
            &DerivedIndexMaintenanceDenialKind::WorkBudgetExceeded
        ));
        assert!(!index_maintenance_denial_is_budget_stop(
            &DerivedIndexMaintenanceDenialKind::IndexUnavailable(DerivedIndexId(1))
        ));
        assert!(!index_maintenance_denial_is_budget_stop(
            &DerivedIndexMaintenanceDenialKind::PriorEntryMismatch
        ));
        assert!(!index_maintenance_denial_is_budget_stop(
            &DerivedIndexMaintenanceDenialKind::ColdReconstructionRequired
        ));
    }
}
