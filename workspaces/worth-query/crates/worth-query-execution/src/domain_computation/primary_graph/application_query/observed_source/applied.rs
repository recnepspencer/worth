//! Candidate source meaning is retained without issuing a Current observation.
use super::*;
use crate::domain_computation::primary_graph::{
    application_query::{
        read_execution::{applied_source, read_plan::ReadPlan, ReadInterruption},
        WorthQueryAdmittedApplicationQueryPlan,
    },
    source_read_equivalence::SourceReadEquivalence,
    value_decode::ValueDecodeState,
    WorthQueryPrimaryGraphApplicationRuntime,
};
use applied_source::AppliedSourceReadStop as Stop;
use worth_query_installation::facade::ApplicationSchema;
use worth_relational::facade::runtime::{
    RelationalAppliedCandidateReadIdentity, RelationalPinnedRead,
};
use worth_relational::facade::transactions::RelationalExecutionDenialCause as Cause;

pub(in crate::domain_computation::primary_graph) struct AppliedQuerySource<Query, Value> {
    pub(in crate::domain_computation::primary_graph) value: Value,
    pub(in crate::domain_computation::primary_graph) candidate:
        RelationalAppliedCandidateReadIdentity,
    pub(super) query: [u8; 32],
    pub(super) runtime_authority: u64,
    pub(super) admitted_parameters: worth_query_admission::facade::application_query::WorthQueryAdmittedApplicationQueryParameters,
    parameters: [u8; 32],
    pub(super) meaning: std::sync::Arc<source_identity::WorthQueryObservedSourceMeaning>,
    marker: PhantomData<fn() -> Query>,
}
/// The actual candidate source survives preparation until its whole publication
/// is validated and the real successor can issue a committed source epoch.
#[derive(Clone)]
pub(in crate::domain_computation::primary_graph) struct AppliedSourcePublication {
    meaning: std::sync::Arc<source_identity::WorthQueryObservedSourceMeaning>,
}

impl AppliedSourcePublication {
    pub(in crate::domain_computation::primary_graph) fn checkpoint_identity(&self) -> [u8; 32] {
        self.meaning.checkpoint_identity().bytes()
    }
}
impl<Query, Value> AppliedQuerySource<Query, Value> {
    pub(in crate::domain_computation::primary_graph) fn into_publication_source(
        self,
    ) -> AppliedSourcePublication {
        AppliedSourcePublication {
            meaning: self.meaning,
        }
    }

    pub(in crate::domain_computation::primary_graph) fn runtime_source_identity(
        &self,
    ) -> WorthQueryRuntimeSourceIdentity {
        self.meaning.runtime_idempotency_identity()
    }
    pub(in crate::domain_computation::primary_graph) fn checkpoint_source_identity(
        &self,
    ) -> WorthQueryCheckpointSourceIdentity {
        self.meaning.checkpoint_identity()
    }
    pub(in crate::domain_computation::primary_graph) fn root(&self) -> EntityId {
        self.meaning.footprint().root
    }
    pub(in crate::domain_computation::primary_graph) fn partition(&self) -> [u8; 32] {
        self.parameters
    }
    pub(in crate::domain_computation::primary_graph) fn idempotency_identity(&self) -> [u8; 32] {
        self.meaning.runtime_idempotency_identity().bytes()
    }
    pub(in crate::domain_computation::primary_graph) fn checkpoint_identity(&self) -> [u8; 32] {
        self.meaning.checkpoint_identity().bytes()
    }
}
impl<Schema: ApplicationSchema> WorthQueryPrimaryGraphApplicationRuntime<Schema> {
    pub(in crate::domain_computation::primary_graph) fn project_applied_source_plan<
        Query,
        Parameters,
        Value,
        Principal,
        PrincipalIdentity,
        Scope,
    >(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        plan: WorthQueryAdmittedApplicationQueryPlan<
            '_,
            Schema,
            Query,
            Parameters,
            Value,
            Principal,
            PrincipalIdentity,
            Scope,
        >,
        candidate: &RelationalPinnedRead<RelationalAppliedCandidateReadIdentity>,
    ) -> Result<AppliedQuerySource<Query, Value>, Stop>
    where
        Value: super::super::WorthQueryApplicationProjection<Schema, Query>,
    {
        let graph = self
            .runtime
            .primary_graph()
            .ok_or(Stop::Native(Cause::ExpectedIdentitiesNotCanonical))?;
        let request = plan.controls.request_scope();
        let projected = applied_source::read_applied_source::<Schema, Query, Value>(
            execution,
            &ValueDecodeState::new(self.installed_schema.value_bindings().clone()),
            candidate,
            &graph.layout,
            &ReadPlan::of(&plan),
            ReadInterruption::Query(request),
        )?;
        let query = *plan.query.identity().as_bytes();
        let parameters = *plan.parameters.identity().bytes();
        let mut comparisons = SourceReadEquivalence::default();
        let control = || {
            match request.interruption() {
            None => Ok(()),
            Some(worth_query_admission::facade::authenticated_principal::WorthQueryRequestInterruption::Cancelled) => Err(Cause::Cancelled),
            Some(worth_query_admission::facade::authenticated_principal::WorthQueryRequestInterruption::DeadlineExceeded) => Err(Cause::DeadlineElapsed),
        }
        };
        let meaning = self
            .runtime
            .source_meanings()
            .intern_applied_compared(
                &query,
                &parameters,
                projected.footprint,
                plan.security_product.observation(),
                &projected.identity,
                &mut |left, right| {
                    super::comparison::equal(left, right, &mut comparisons, execution, &control)
                },
            )
            .map_err(Stop::Native)?
            .ok_or(Stop::Native(Cause::ExpectedIdentitiesNotCanonical))?;
        Ok(AppliedQuerySource {
            value: projected.value,
            candidate: projected.identity,
            query,
            runtime_authority: plan.runtime_authority.as_u64(),
            admitted_parameters: plan.parameters,
            parameters,
            meaning,
            marker: PhantomData,
        })
    }
}
