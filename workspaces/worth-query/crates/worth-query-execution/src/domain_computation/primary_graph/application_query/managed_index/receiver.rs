//! Binding-aware one-shot entry; ordinary queries keep their existing engine.
use super::{
    denial::{
        ReceiverCause, WorthQueryManagedIndexDenial as Denial,
        WorthQueryManagedIndexRefusal as Refusal,
    },
    execution::ProviderExecutionDenial,
};
use crate::domain_computation::primary_graph::{
    application_query::{
        authorized_read::execute_authorized_read,
        one_shot::{self, outcome::finalize_one_shot},
        read_execution::{self, read_plan::ReadPlan, ReadInterruption},
        WorthQueryAdmittedApplicationQueryPlan, WorthQueryApplicationOneShotResult,
        WorthQueryApplicationProjection,
    },
    source_facts::SourceFactsAllocation,
    WorthQueryAdvancementPhase, WorthQueryPrimaryGraphApplicationRuntime,
};
use std::sync::Arc;
use worth_query_admission::facade::authenticated_principal::WorthQueryAuthenticatedExternalPrincipal;
use worth_query_declaration::facade::{
    application_query::{ApplicationQueryBinding, ApplicationQueryScopeBinding},
    application_schema::{ApplicationSchema, ApplicationStructuredValueBinding},
};
use worth_query_installation::facade::WorthQueryInstalledApplicationQueryBinding;

mod context;
mod lifecycle;

pub(super) type ResultValue<S, B> =
    <<B as ApplicationQueryBinding<S>>::ResultBinding as ApplicationStructuredValueBinding>::Value;
pub(super) type BoundPlan<'a, S, B> = WorthQueryAdmittedApplicationQueryPlan<
    'a, S, <B as ApplicationQueryBinding<S>>::Query,
    <<B as ApplicationQueryBinding<S>>::ParameterBinding as ApplicationStructuredValueBinding>::Value,
    ResultValue<S, B>, <B as ApplicationQueryBinding<S>>::Principal,
    <B as ApplicationQueryBinding<S>>::PrincipalIdentity,
    <<B as ApplicationQueryBinding<S>>::ScopeBinding as ApplicationQueryScopeBinding<S>>::Scope,
>;

impl<S: ApplicationSchema> WorthQueryPrimaryGraphApplicationRuntime<S> {
    /// Executes this actual typed binding. A committed managed selector uses
    /// its existing Ready registration; Created/Evicted requires explicit rebuild.
    pub fn execute_application_query_binding_one_shot<B>(
        &self,
        phase: &WorthQueryAdvancementPhase<'_>,
        binding: &WorthQueryInstalledApplicationQueryBinding<S, B>,
        external: &WorthQueryAuthenticatedExternalPrincipal<S>,
        plan: BoundPlan<'_, S, B>,
    ) -> Result<WorthQueryApplicationOneShotResult<B::Query, ResultValue<S, B>>, Denial>
    where
        B: ApplicationQueryBinding<S>,
        ResultValue<S, B>: WorthQueryApplicationProjection<S, B::Query>,
    {
        let execution = phase
            .execution_request_for(&self.product_runtime)
            .map_err(|_| Denial::new(ReceiverCause::Refused(Refusal::ForeignPhase), None))?;
        if !std::ptr::eq(binding.query(), plan.query) {
            return Err(Denial::new(
                ReceiverCause::Refused(Refusal::ForeignBinding),
                None,
            ));
        }
        if plan.query.read_graph().managed_index_selection().is_none() {
            return self
                .execute_application_query_one_shot_with_request(&execution, plan)
                .map_err(|cause| Denial::new(ReceiverCause::Read(cause), None));
        }
        let allocation = SourceFactsAllocation::open(&execution)
            .map_err(|cause| Denial::new(ReceiverCause::Execution(cause), None))?;
        self.execute_managed_parent(&execution, binding, external, plan, &allocation)
            .map_err(|cause| Denial::new(cause, Some(allocation)))
    }

    fn execute_managed_parent<B>(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        binding: &WorthQueryInstalledApplicationQueryBinding<S, B>,
        _external: &WorthQueryAuthenticatedExternalPrincipal<S>,
        mut plan: BoundPlan<'_, S, B>,
        allocation: &Arc<SourceFactsAllocation>,
    ) -> Result<WorthQueryApplicationOneShotResult<B::Query, ResultValue<S, B>>, ReceiverCause>
    where
        B: ApplicationQueryBinding<S>,
        ResultValue<S, B>: WorthQueryApplicationProjection<S, B::Query>,
    {
        let context = context::prepare(execution, self, binding, &mut plan, allocation)?;
        let adapter = self
            .managed_indexes
            .executor(execution, &context.association, allocation)?
            .ok_or(ReceiverCause::Refused(Refusal::MissingProvider))?;
        let parameter = &context
            .association
            .access_contract()
            .meaning()
            .parent_graph()
            .managed_index_selection()
            .ok_or(ReceiverCause::Refused(Refusal::NotIndexParent))?
            .parts()
            .probe_parameter;
        let storage = one_shot::outcome::material::MaterialCompletion::new(execution, allocation);
        let count = plan.parameters.bindings().len().checked_add(1)
            .and_then(|n| n.checked_mul(parameter.len().checked_add(1)?))
            .and_then(|n| u64::try_from(n).ok())
            .ok_or(worth_relational::facade::transactions::RelationalExecutionDenialCause::WorkCounterOverflow)?;
        storage.work(count)?;
        let probe = plan
            .parameters
            .bindings()
            .iter()
            .find(|(name, _)| *name == parameter.as_str())
            .map(|(_, value)| value)
            .ok_or(ReceiverCause::Refused(Refusal::MissingProbe))?;
        let lease = adapter
            .select(
                execution,
                self,
                &plan.graph_work,
                plan.controls.request_scope(),
                &context.association,
                &context.selected,
                probe,
                allocation,
            )
            .map_err(ReceiverCause::Provider)?;
        let rows = lease
            .rows(&plan.graph_work, plan.controls.request_scope())
            .map_err(|cause| ReceiverCause::Provider(ProviderExecutionDenial::Selection(cause)))?;
        let (kernel, authorization, proof) =
            execute_authorized_read(execution, self, &plan, |runtime, graph, plan| {
                let mut read = ReadPlan::of(plan);
                read.managed_rows = Some(rows);
                read.material_source = Some(allocation);
                read_execution::read_bounded_plan_rows(
                    runtime,
                    execution,
                    graph,
                    &read,
                    None,
                    ReadInterruption::Query(plan.controls.request_scope()),
                )
            })
            .map_err(|cause| {
                ReceiverCause::Read(one_shot::map_authorized_read_denial(
                    cause,
                    plan.query.name(),
                ))
            })?;
        lease
            .rows(&plan.graph_work, plan.controls.request_scope())
            .map_err(|cause| ReceiverCause::Provider(ProviderExecutionDenial::Selection(cause)))?;
        // The genuine SDK request/session loan never enters the Sync read plan
        // and ends before the Parent's consuming Query terminal.
        drop(lease);
        finalize_one_shot(execution, self, plan, kernel, authorization, proof)
            .map_err(ReceiverCause::Read)
    }
}
