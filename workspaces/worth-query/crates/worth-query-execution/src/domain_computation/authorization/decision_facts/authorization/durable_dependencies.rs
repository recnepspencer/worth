//! Portable descriptions of Relational-owned native authorization dependencies.

use serde::{Deserialize, Serialize};
use worth_relational::facade::authorization::{
    RelationalAuthorizationDurableDependencies, RelationalAuthorizationObservationEvidence,
};
use worth_relational::facade::runtime::RelationalRuntime;
use worth_relational::facade::snapshots::SnapshotHandle;

use super::WorthQueryAuthorizationDecisionFact;

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(in crate::domain_computation) struct WorthQueryDurableAuthorizationDependencies {
    primary: String,
    recorded_actor: DurableRecordedActor,
    delegation: Option<WorthQueryDurableDelegationDependencies>,
    activation: Option<String>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
enum DurableRecordedActor {
    NotApplicable,
    Native(String),
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) enum WorthQueryDurableDelegationDependencies {
    Root {
        discovery: String,
    },
    Delegated {
        discovery: String,
        transition: String,
        parent: Box<WorthQueryDurableAuthorizationDependencies>,
    },
}

mod encoding;
pub(in crate::domain_computation) use encoding::DurableDependenciesView;

impl WorthQueryAuthorizationDecisionFact {
    pub(in crate::domain_computation) fn retained_durable_dependencies(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
    ) -> Result<
        DurableDependenciesView<'_>,
        crate::domain_computation::authorization::WorkflowApprovalRequestEncodingDenial,
    > {
        encoding::capture(self, execution)
    }
}

impl WorthQueryDurableAuthorizationDependencies {
    pub(in crate::domain_computation) fn validate(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
    ) -> Result<(), crate::domain_computation::authorization::WorkflowApprovalRequestDecodingDenial>
    {
        validate(execution, &self.primary)?;
        if let DurableRecordedActor::Native(encoded) = &self.recorded_actor {
            crate::domain_computation::primary_graph::application_attempt::workflow_transition_program::WorkflowRecordedActorObservation::validate_encoding(execution, encoded)?;
        }
        if let Some(delegation) = &self.delegation {
            delegation.validate(execution)?;
        }
        if let Some(activation) = &self.activation {
            validate(execution, activation)?;
        }
        Ok(())
    }

    pub(in crate::domain_computation) fn matches_with_retained_activation(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        observed: &DurableDependenciesView<'_>,
        runtime: &RelationalRuntime,
        snapshot: &SnapshotHandle,
    ) -> Result<
        bool,
        crate::domain_computation::authorization::WorthQueryOperationAuthorizationDenial,
    > {
        #[cfg(test)]
        let _scope = crate::domain_computation::primary_graph::application_attempt::consult_observation::ReadConsultScopeForTest::enter_for_test(crate::domain_computation::primary_graph::application_attempt::consult_observation::Door::Activation);
        execution.check_live().map_err(|stop| denial::admission(
            worth_relational::facade::authorization::RelationalAuthorizationDependencyDenial::Execution(stop.into())))?;
        if !self.matches_view_without_activation(observed) || observed.has_activation() {
            return Ok(false);
        }
        let Some(activation) = &self.activation else {
            return Ok(false);
        };
        use worth_relational::facade::authorization::RelationalAuthorizationDependencyDenial as Stop;
        let stamp =
            match RelationalAuthorizationDurableDependencies::from_wire(execution, activation) {
                Ok(stamp) => stamp,
                Err(cause @ (Stop::Execution(_) | Stop::Allocation(_))) => {
                    return Err(denial::admission(cause))
                }
                Err(_) => return Ok(false),
            };
        match stamp.source_revisions_match(runtime, execution, snapshot) {
            Ok(current) => Ok(current),
            Err(cause @ (Stop::Execution(_) | Stop::Allocation(_))) => {
                Err(denial::admission(cause))
            }
            Err(_) => Ok(false),
        }
    }
}

impl WorthQueryDurableDelegationDependencies {
    fn validate(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
    ) -> Result<(), crate::domain_computation::authorization::WorkflowApprovalRequestDecodingDenial>
    {
        match self {
            Self::Root { discovery } => validate(execution, discovery),
            Self::Delegated {
                discovery,
                transition,
                parent,
            } => {
                validate(execution, discovery)?;
                validate(execution, transition)?;
                parent.validate(execution)
            }
        }
    }
}

mod denial;

pub(super) fn capture(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    runtime: &RelationalRuntime,
    evidence: &RelationalAuthorizationObservationEvidence,
) -> Result<
    worth_relational::facade::authorization::RelationalAuthorizationDependencyWire,
    crate::domain_computation::authorization::WorthQueryOperationAuthorizationDenial,
> {
    runtime
        .capture_authorization_durable_dependencies(evidence, execution)
        .map_err(denial::admission)?
        .to_wire(execution)
        .map_err(denial::admission)
}

fn validate(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    encoded: &str,
) -> Result<(), crate::domain_computation::authorization::WorkflowApprovalRequestDecodingDenial> {
    RelationalAuthorizationDurableDependencies::from_wire(execution, encoded)
        .map(|_| ())
        .map_err(denial::decode)
}

#[cfg(test)]
mod cancellation_tests;
