//! Original Query session and authorization custody through publication.

use super::*;
use crate::domain_computation::authorization::SourceReadSupport;
use crate::domain_computation::authorization::WorthQueryRetainedAuthorizationDecisionFacts;

use std::sync::Arc;

pub(in crate::domain_computation::primary_graph) struct SourceQueryReadClosure {
    control:crate::domain_computation::primary_graph::application_runtime::WorthQueryApplicationAttemptOperationControl,
    receipt: WorthQueryApplicationQueryAccessReceipt,
    support: SourceReadSupport,
    facts: crate::domain_computation::primary_graph::application_checkpoint::facts::FactPayload,

}

impl SourceQueryReadClosure {
    pub(super) fn retain<Schema: ApplicationSchema>(
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        application: &dyn crate::domain_computation::primary_graph::application_query::host::ApplicationQueryHost<Schema>,
        receipt: WorthQueryApplicationQueryAccessReceipt,
        authorization: WorthQueryRetainedAuthorizationDecisionFacts,
        facts: Vec<crate::domain_computation::primary_graph::WorthQueryApplicationObservedFact>,
    ) -> Result<Arc<Self>, CompanionPreflightStop> {
        let allocation =
            crate::domain_computation::primary_graph::source_facts::SourceFactsAllocation::open(
                execution,
            )
            .map_err(CompanionPreflightStop::Execution)?;
        allocation
            .claim_arc::<Self>(execution)
            .map_err(CompanionPreflightStop::Execution)?;
        let support = SourceReadSupport::retain(
            execution,
            authorization,
            application.authorization(),
            &allocation,
        )?;
        let facts = crate::domain_computation::primary_graph::application_checkpoint::facts::FactPayload::from(Arc::<[crate::domain_computation::primary_graph::WorthQueryApplicationObservedFact]>::from(facts));
        Ok(Arc::new(Self {
            control: application
                .primary_provider()
                .application_attempt_operation_control
                .clone(),
            receipt,
            support,
            facts,
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
                .support
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
        self.support.principal_facts()
    }

    pub(in crate::domain_computation::primary_graph) fn validate_for_publication(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        runtime: &worth_relational::facade::runtime::RelationalRuntime,
        snapshot: &worth_relational::facade::snapshots::SnapshotHandle,

    ) -> Result<(), crate::domain_computation::primary_graph::invariant_projection::ConsumedOutputVerificationStop>{
        self.support.validate_current(execution, runtime, snapshot).map_err(crate::domain_computation::primary_graph::source_facts::map_source_authorization_currentness)
    }
}
