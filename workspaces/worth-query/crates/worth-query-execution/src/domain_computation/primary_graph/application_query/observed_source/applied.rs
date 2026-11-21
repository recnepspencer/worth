//! Candidate source meaning is retained without issuing a Current observation.
use crate::domain_computation::primary_graph::application_attempt::fact::source_read_authorization::applied_query::SourceReadAuthorizationAppliedQueryExt;
use std::marker::PhantomData;
use super::*;
use worth_relational::facade::identity::EntityId;
use crate::domain_computation::primary_graph::{
    application_query::read_execution::{applied_source, read_plan::ReadPlan, ReadInterruption},
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
    // The admitted query scope is distinct from its selected projection row.
    pub(in crate::domain_computation::primary_graph) model_root: EntityId,
    pub(super) query: [u8; 32],
    pub(in crate::domain_computation::primary_graph) runtime_authority: u64,
    pub(in crate::domain_computation::primary_graph) schema_binding: worth_query_installation::facade::ApplicationSchemaBindingIdentity,
    pub(super) admitted_parameters: worth_query_admission::facade::application_query::WorthQueryAdmittedApplicationQueryParameters,
    parameters: [u8; 32],
    pub(super) meaning: std::sync::Arc<source_identity::WorthQueryObservedSourceMeaning>,
    pub(super) managed: Option<std::sync::Arc<
        super::super::managed_index::applied_read::AppliedManagedSourceSupport,
    >>,
    pub(in crate::domain_computation::primary_graph) closure: std::sync::Arc<super::super::SourceQueryReadClosure>,
    marker: PhantomData<fn() -> Query>,
}
impl<Query, Value> AppliedQuerySource<Query, Value> {
    pub(in crate::domain_computation::primary_graph) fn retain_selected_membership(
        &self,
    ) -> Option<super::WorthQueryObservedSourceSelection> {
        if self.managed.is_some() {
            return None;
        }
        super::WorthQueryObservedSourceSelection::complete(
            self.runtime_authority,
            self.schema_binding.clone(),
            std::sync::Arc::clone(&self.meaning),
            None,
        )
    }

    pub(in crate::domain_computation::primary_graph) fn has_complete_output_dependencies(
        &self,
    ) -> bool {
        let footprint = self.meaning.footprint();
        footprint.complete && footprint.root_selection.is_some()
    }
    pub(in crate::domain_computation::primary_graph) fn into_publication_source(
        self,
        lifecycle: crate::domain_computation::primary_graph::application_contribution::WorthQueryProducerLifecyclePosture,
    ) -> AppliedSourcePublication {
        AppliedSourcePublication {
            managed: self.managed.map(|m| m as std::sync::Arc<dyn worth_query_execution_lineage::facade::ports::AppliedManagedSourcePort>),
            meaning: self.meaning,
            closure: self.closure as std::sync::Arc<dyn worth_query_execution_lineage::facade::ports::SourceReadClosurePort>,
            lifecycle,
        }
    }

    pub(in crate::domain_computation::primary_graph) fn runtime_source_identity(
        &self,
    ) -> WorthQueryRuntimeSourceIdentity {
        self.meaning.runtime_idempotency_identity()
    }
    pub(in crate::domain_computation::primary_graph) fn root(&self) -> EntityId {
        self.meaning.footprint().root
    }
    pub(in crate::domain_computation::primary_graph) fn binding_order(
        &self,
    ) -> ([u8; 32], [u8; 32], EntityId) {
        (self.query, self.parameters, self.root())
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
    pub(in crate::domain_computation::primary_graph) fn project_applied_source_plan<B, Projected>(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        reader: &super::super::output_source_observation::SourceQueryReaderAccess<'_, Schema>,
        plan: super::super::output_source_observation::SourceReadPlan<'_, Schema, B>,
        before: &RelationalPinnedRead,
        candidate: &RelationalPinnedRead<RelationalAppliedCandidateReadIdentity>,
        assembly: &worth_relational::facade::mvcc::RelationalAuthoredCoupledCandidate<'_>,
        native: &mut super::super::ApplicationQueryNativeAccess<'_>,
    ) -> Result<AppliedQuerySource<B::Query, Projected>, super::super::SourceQueryObservationStop>
    where
        B: worth_query_declaration::facade::application_query::ApplicationQueryBinding<Schema>,
        <<B as worth_query_declaration::facade::application_query::ApplicationQueryBinding<Schema>>::ResultBinding as worth_query_declaration::facade::application_schema::ApplicationStructuredValueBinding>::Value:
            super::super::WorthQueryApplicationProjection<Schema, B::Query>,
        Projected: super::super::WorthQueryApplicationProjection<Schema, B::Query>,
{
        use super::super::{
            authorized_read::execute_selected_authorized_body_with_native_access,
            one_shot::map_selected_read_stop, SourceQueryObservationStop as SourceStop,
        };
        let query_name = plan.query.name();
        let (projected, managed, proof) = if plan
            .query
            .read_family_binding()
            .planning_contract()
            .managed_index_selection()
            .is_some()
        {
            let (projected, support, proof) =
                super::super::managed_index::applied_read::project::<Schema, B, Projected>(
                    execution, reader, &plan, before, candidate, assembly, native,
                )?;
            (projected, Some(support), proof)
        } else {
            let (projected, _, proof) = execute_selected_authorized_body_with_native_access(
                execution,
                self,
                &plan,
                native,
                |_, layout, admitted| {
                    Ok(applied_source::read_applied_source::<
                        Schema,
                        B::Query,
                        Projected,
                    >(
                        execution,
                        &ValueDecodeState::new(self.installed_schema.value_bindings().clone()),
                        candidate,
                        layout,
                        &ReadPlan::of(admitted),
                        ReadInterruption::Query(admitted.controls.request_scope()),
                    ))
                },
            )
            .map_err(|cause| SourceStop::Read(map_selected_read_stop(cause, query_name)))?;
            (
                projected.map_err(|cause| cause.into_source_query_stop(query_name))?,
                None,
                proof,
            )
        };
        self.retain_applied_source_plan::<B, Projected>(
            execution, plan, projected, managed, proof, candidate,
        )
    }

    fn retain_applied_source_plan<B, Projected>(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        plan: super::super::output_source_observation::SourceReadPlan<'_, Schema, B>,
        projected: applied_source::AppliedSourceProjection<Projected>,
        managed: Option<
            std::sync::Arc<super::super::managed_index::applied_read::AppliedManagedSourceSupport>,
        >,
        proof: crate::domain_computation::provider_session::WorthQuerySessionGraphReadProof,
        candidate: &RelationalPinnedRead<RelationalAppliedCandidateReadIdentity>,
    ) -> Result<AppliedQuerySource<B::Query, Projected>, super::super::SourceQueryObservationStop>
    where
        B: worth_query_declaration::facade::application_query::ApplicationQueryBinding<Schema>,
    {
        let request = plan.controls.request_scope();
        let query_name = plan.query.name();
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
            .map_err(|cause| Stop::Native(cause).into_source_query_stop(query_name))?
            .ok_or_else(|| {
                Stop::Native(Cause::ExpectedIdentitiesNotCanonical)
                    .into_source_query_stop(query_name)
            })?;
        let graph = self
            .runtime
            .primary_graph()
            .ok_or(super::super::SourceQueryObservationStop::ForeignClosure)?;
        let facts = fact_conversion::applied_facts(
            meaning.footprint(),
            &graph.layout,
            query_name,
            fact_conversion::SourceFactExpansionControl::in_request(*execution, Some(request)),
        )?;
        let allocation =
            crate::domain_computation::primary_graph::source_facts::SourceFactsAllocation::open(
                execution,
            )
            .map_err(|cause| {
                super::super::SourceQueryObservationStop::Read(
                    super::super::WorthQueryAdmittedOneShotStop::Admission(
                        worth_relational::facade::mvcc::CompanionPreflightStop::Execution(cause),
                    ),
                )
            })?;
        let admission = |cause| {
            super::super::SourceQueryObservationStop::Read(
                super::super::WorthQueryAdmittedOneShotStop::Admission(cause),
            )
        };
        let authorization = crate::domain_computation::primary_graph::application_attempt::SourceReadAuthorization::retain_applied_query(
            execution, &proof, plan.query.binding_observation().cloned().ok_or(super::super::SourceQueryObservationStop::ForeignClosure)?,
            query, plan.principal.principal_entity_id(), plan.scope.entity_id(), candidate.candidate_identity(),
            plan.authorization, &self.runtime.authorization, &allocation,
        ).map_err(admission)?;
        let closure = super::super::SourceQueryReadClosure::retain_applied(
            execution,
            self,
            proof,
            authorization,
            candidate,
            facts,
        )
        .map_err(admission)?;
        Ok(AppliedQuerySource {
            value: projected.value,
            candidate: projected.identity,
            model_root: plan.scope.entity_id(),
            query,
            runtime_authority: plan.runtime_authority.as_u64(),
            schema_binding: self.installed_schema.binding_identity(),
            admitted_parameters: plan.parameters,
            parameters,
            meaning,
            managed,
            closure,
            marker: PhantomData,
        })
    }
}
