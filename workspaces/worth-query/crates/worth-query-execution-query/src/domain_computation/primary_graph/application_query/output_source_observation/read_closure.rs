//! Original Query session and authorization custody through publication.

use worth_query_execution_attempt::facade::ports::PrimaryGraphProviderInternalPort as _;
use worth_query_execution_lineage::facade::ports::ObservedSourceInternalPort as _;
use crate::domain_computation::primary_graph::application_attempt::fact::source_read_authorization::applied_query::SourceReadAuthorizationAppliedQueryExt;
use super::*;
use crate::domain_computation::authorization::{
    SourceReadSupport, WorthQueryRetainedAuthorizationDecisionFacts,
};
use crate::domain_computation::primary_graph::{
    application_attempt::{
        Movement, WorthQueryApplicationObservedFact as Fact,
        WorthQuerySourceCurrentnessFailure as CurrentnessStop,
    },
    application_checkpoint::facts::FactPayload,
    application_runtime::WorthQueryApplicationAttemptOperationControl,
    invariant_projection::{consumed_output::map_admission_stop, ConsumedOutputVerificationStop},
    source_facts::{map_source_authorization_currentness, SourceFactsAllocation},
};
use worth_relational::facade::{runtime::RelationalRuntime, snapshots::SnapshotHandle};

use std::sync::Arc;
mod applied;
use applied::AppliedSourceQueryRead;

enum SourceQueryReadAuthority {
    Committed {
        receipt: WorthQueryApplicationQueryAccessReceipt,
        support: SourceReadSupport,
    },
    Applied(AppliedSourceQueryRead),
}

pub struct SourceQueryReadClosure {
    control: WorthQueryApplicationAttemptOperationControl,
    authority: SourceQueryReadAuthority,
    facts: FactPayload,
}

impl SourceQueryReadClosure {
    pub fn retain<Schema: ApplicationSchema>(
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        application: &WorthQueryPrimaryGraphApplicationRuntime<Schema>,
        receipt: WorthQueryApplicationQueryAccessReceipt,
        authorization: WorthQueryRetainedAuthorizationDecisionFacts,
        facts: Vec<Fact>,
    ) -> Result<Arc<Self>, CompanionPreflightStop> {
        let allocation =
            SourceFactsAllocation::open(execution).map_err(CompanionPreflightStop::Execution)?;
        allocation
            .claim_arc::<Self>(execution)
            .map_err(CompanionPreflightStop::Execution)?;
        let support = SourceReadSupport::retain(
            execution,
            authorization,
            &application.runtime.authorization,
            &allocation,
        )?;
        let facts = FactPayload::from(Arc::<[Fact]>::from(facts));
        Ok(Arc::new(Self {
            control: application
                .runtime
                .primary_provider
                .application_attempt_operation_control_ref()
                .clone(),
            authority: SourceQueryReadAuthority::Committed { receipt, support },
            facts,
        }))
    }

    pub fn admits_source<Query>(&self, source: &WorthQueryObservedSource<Query>) -> bool {
        let SourceQueryReadAuthority::Committed { receipt, support } = &self.authority else {
            return false;
        };
        receipt.query_identity() == &source.query_identity()
            && receipt.parameter_binding_identity() == &source.parameter_binding_identity()
            && receipt.basis_identity().selection() == &source.selection()
            && support.belongs_to_session(receipt.read_completion().session_identity())
    }

    pub fn admits_source_identity(
        &self,
        _runtime_authority: u64,
        _schema_binding: &worth_query_installation::facade::ApplicationSchemaBindingIdentity,
        selection: &crate::domain_computation::primary_graph::application_query::WorthQueryApplicationBasisSelectionIdentity,
    ) -> bool {
        let SourceQueryReadAuthority::Committed { receipt, support } = &self.authority else {
            return false;
        };
        receipt.basis_identity().selection() == selection
            && support.belongs_to_session(receipt.read_completion().session_identity())
    }

    pub fn facts(&self) -> &FactPayload {
        &self.facts
    }

    pub fn source_facts_current(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        runtime: &RelationalRuntime,
        snapshot: &SnapshotHandle,
    ) -> Result<bool, CurrentnessStop> {
        if let SourceQueryReadAuthority::Applied(applied) = &self.authority
            && let Some(current) =
                applied.source_facts_at_acquisition(execution, runtime, snapshot, &self.facts)?
        {
            return Ok(current);
        }
        for fact in self.facts.iter() {
            self.control.before_source_fact_comparison();
            let movement = fact
                .source_currentness_in(runtime, execution, snapshot)
                .map_err(|stop| CurrentnessStop::OutputObservation(map_admission_stop(stop)))??;
            if movement.movement() != Movement::Unmoved {
                return Ok(false);
            }
        }
        Ok(true)
    }

    pub fn authorization_facts(&self) -> &[Fact] {
        match &self.authority {
            SourceQueryReadAuthority::Committed { support, .. } => support.principal_facts(),
            SourceQueryReadAuthority::Applied(applied) => applied.authorization.principal_facts(),
        }
    }

    pub fn validate_for_publication(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        runtime: &RelationalRuntime,
        snapshot: &SnapshotHandle,
    ) -> Result<(), ConsumedOutputVerificationStop> {
        match &self.authority {
            SourceQueryReadAuthority::Committed { support, .. } => support
                .validate_current(execution, runtime, snapshot)
                .map_err(map_source_authorization_currentness),
            SourceQueryReadAuthority::Applied(applied) => applied
                .authorization
                .validate_current(runtime, execution, snapshot),
        }
    }
}
