//! Original Query session and authorization custody through publication.

use super::*;
use crate::domain_computation::authorization::WorthQueryInstalledAuthorizationRegistry;
use crate::domain_computation::authorization::WorthQueryRetainedAuthorizationDecisionFacts;

use std::sync::Arc;

pub(in crate::domain_computation::primary_graph) struct SourceQueryReadClosure {
    control:crate::domain_computation::primary_graph::application_runtime::WorthQueryApplicationAttemptOperationControl,
    receipt: WorthQueryApplicationQueryAccessReceipt,
    authorization: WorthQueryRetainedAuthorizationDecisionFacts,
    registry: Arc<WorthQueryInstalledAuthorizationRegistry>,
    principal_facts: Arc<[crate::domain_computation::primary_graph::WorthQueryApplicationObservedFact]>,
    facts: crate::domain_computation::primary_graph::application_checkpoint::facts::FactPayload,

}

impl SourceQueryReadClosure {
    pub(super) fn retain<Schema: ApplicationSchema>(
        application: &WorthQueryPrimaryGraphApplicationRuntime<Schema>,
        receipt: WorthQueryApplicationQueryAccessReceipt,
        authorization: WorthQueryRetainedAuthorizationDecisionFacts,
        facts: Vec<crate::domain_computation::primary_graph::WorthQueryApplicationObservedFact>,
    ) -> Result<Arc<Self>, CompanionPreflightStop> {
        let principal_facts = authorization.source_query_principal_facts()?;
        let facts = crate::domain_computation::primary_graph::application_checkpoint::facts::FactPayload::from(Arc::<[crate::domain_computation::primary_graph::WorthQueryApplicationObservedFact]>::from(facts));
        Ok(Arc::new(Self {
            control: application
                .primary_provider
                .application_attempt_operation_control
                .clone(),
            receipt,
            principal_facts: principal_facts.into(),
            authorization,
            facts,
            registry: Arc::clone(&application.authorization),
        }))
    }

    pub(in crate::domain_computation::primary_graph) fn admits_source<Query>(
        &self,
        source: &WorthQueryObservedSource<Query>,
    ) -> bool {
        self.receipt.query_identity() == &source.query_identity
            && self.receipt.parameter_binding_identity() == &source.parameter_binding_identity
            && self.receipt.basis_identity().selection() == &source.selection
            && self
                .authorization
                .belongs_to_session(self.receipt.read_completion().session_identity())
    }

    pub(in crate::domain_computation::primary_graph) fn facts(
        &self,
    ) -> &crate::domain_computation::primary_graph::application_checkpoint::facts::FactPayload {
        &self.facts
    }

    pub(in crate::domain_computation::primary_graph) fn source_facts_current(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        runtime: &worth_relational::facade::runtime::RelationalRuntime,
        snapshot: &worth_relational::facade::snapshots::SnapshotHandle,

    ) -> Result<bool, crate::domain_computation::primary_graph::application_attempt::WorthQuerySourceCurrentnessFailure>{
        use crate::domain_computation::primary_graph::application_attempt::Movement;
        use crate::domain_computation::primary_graph::application_attempt::WorthQuerySourceCurrentnessFailure as Stop;
        for fact in self.facts.iter() {
            self.control.before_source_fact_comparison();
            let movement = fact
                .source_currentness_in(runtime, execution, snapshot)
                .map_err(|stop| Stop::OutputObservation(
                    crate::domain_computation::primary_graph::invariant_projection::consumed_output::map_admission_stop(stop)
                ))??;
            if movement.movement() != Movement::Unmoved {
                return Ok(false);
            }
        }
        Ok(true)
    }

    pub(in crate::domain_computation::primary_graph) fn authorization_facts(
        &self,
    ) -> &[crate::domain_computation::primary_graph::WorthQueryApplicationObservedFact] {
        &self.principal_facts
    }

    pub(in crate::domain_computation::primary_graph) fn validate_for_publication(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        runtime: &worth_relational::facade::runtime::RelationalRuntime,
        snapshot: &worth_relational::facade::snapshots::SnapshotHandle,

    ) -> Result<(), crate::domain_computation::primary_graph::invariant_projection::ConsumedOutputVerificationStop>{
        use crate::domain_computation::authorization::AuthorizationCurrentnessStop;
        use crate::domain_computation::authorization::WorthQueryOperationAuthorizationDenialKind as Authorization;
        use crate::domain_computation::authorization::WorthQueryOperationAuthorizationRefusalKind as Refusal;
        use crate::domain_computation::primary_graph::application_attempt::Movement;
        use crate::domain_computation::primary_graph::invariant_projection::consumed_output::map_admission_stop;
        use crate::domain_computation::primary_graph::invariant_projection::ConsumedOutputVerificationStop as Stop;
        for fact in self.principal_facts.iter() {
            let current = fact.source_currentness_in(runtime, execution, snapshot)
                .map_err(map_admission_stop)?.map_err(|stop| match stop {
                    crate::domain_computation::primary_graph::application_attempt::WorthQuerySourceCurrentnessFailure::OutputObservation(stop) => stop,
                    crate::domain_computation::primary_graph::application_attempt::WorthQuerySourceCurrentnessFailure::Execution(stop) => Stop::Execution(stop),
                    crate::domain_computation::primary_graph::application_attempt::WorthQuerySourceCurrentnessFailure::WorkBudgetExceeded => Stop::WorkExhausted,
                    crate::domain_computation::primary_graph::application_attempt::WorthQuerySourceCurrentnessFailure::Unavailable => Stop::Unavailable,
                })?;
            if current.movement() != Movement::Unmoved {
                return Err(Stop::SourceQueryAuthorization(Authorization::Refused(
                    Refusal::StalePrincipal,
                )));
            }
        }
        let current = self
            .authorization
            .conventional_decisions_current_admitted(
                execution,
                runtime,
                snapshot,
                self.registry.bridge(),
            )
            .map_err(|stop| match stop {
                AuthorizationCurrentnessStop::Admission(stop) => map_admission_stop(stop),
                AuthorizationCurrentnessStop::Execution(stop) => Stop::Execution(stop),
            })?;
        current
            .then_some(())
            .ok_or(Stop::SourceQueryAuthorization(Authorization::Refused(
                Refusal::StaleAuthorization,
            )))
    }
}
