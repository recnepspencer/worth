//! Binding-aware one-shot entry; ordinary queries keep their existing engine.
use super::{
    denial::{
        ReceiverCause, WorthQueryManagedIndexDenial as Denial,
        WorthQueryManagedIndexRefusal as Refusal,
    },
    execution::ProviderExecutionDenial,
};
use crate::domain_computation::primary_graph::application_query::OneShotScopedOps;
use std::sync::Arc;
use worth_query_admission::facade::authenticated_principal::WorthQueryAuthenticatedExternalPrincipal;
use worth_query_declaration::facade::{
    application_query::{ApplicationQueryBinding, ApplicationQueryScopeBinding},
    application_schema::{ApplicationSchema, ApplicationStructuredValueBinding},
};
use worth_query_installation::facade::WorthQueryInstalledApplicationQueryBinding;
use {
    crate::domain_computation::primary_graph::application_query::authorized_read::execute_authorized_read,
    crate::domain_computation::primary_graph::application_query::one_shot,
    crate::domain_computation::primary_graph::application_query::read_execution,
    crate::domain_computation::primary_graph::application_query::read_execution::read_plan::ReadPlan,
    crate::domain_computation::primary_graph::application_query::read_execution::ReadInterruption,
    crate::domain_computation::primary_graph::application_query::WorthQueryAdmittedApplicationQueryPlan,
    crate::domain_computation::primary_graph::application_query::WorthQueryApplicationOneShotResult,
    crate::domain_computation::primary_graph::application_query::WorthQueryApplicationProjection,
    crate::domain_computation::primary_graph::source_facts::SourceFactsAllocation,
    crate::domain_computation::request_execution::WorthQueryAdvancementPhase,
};

mod context;
mod eviction;
mod lifecycle;
mod watch;
pub use watch::{
    WorthQueryManagedIndexWatch, WorthQueryManagedIndexWatchOutcome,
    WorthQueryManagedIndexWatchUpdate,
};

pub(super) type ResultValue<S, B> =
    <<B as ApplicationQueryBinding<S>>::ResultBinding as ApplicationStructuredValueBinding>::Value;
pub(super) type BoundPlan<'a, S, B> = WorthQueryAdmittedApplicationQueryPlan<
    'a, S, <B as ApplicationQueryBinding<S>>::Query,
    <<B as ApplicationQueryBinding<S>>::ParameterBinding as ApplicationStructuredValueBinding>::Value,
    ResultValue<S, B>, <B as ApplicationQueryBinding<S>>::Principal,
    <B as ApplicationQueryBinding<S>>::PrincipalIdentity,
    <<B as ApplicationQueryBinding<S>>::ScopeBinding as ApplicationQueryScopeBinding<S>>::Scope,
>;
enum ManagedParentCapture {
    None,
    Watch,
}
struct ManagedParentRead<'plan, S: ApplicationSchema, B: ApplicationQueryBinding<S>> {
    plan: BoundPlan<'plan, S, B>,
    kernel: read_execution::RawNonLiveKernelOutcome,
    authorization: super::super::WorthQueryApplicationAuthorizationWorkEvidence,
    proof: crate::domain_computation::provider_session::WorthQuerySessionGraphReadProof,
    selection: Option<super::selection::MaintainedWatchSelection>,
}

pub trait ReceiverOps<S: ApplicationSchema>:
    crate::domain_computation::primary_graph::application_query::host::ApplicationQueryHost<S>
{
    fn execute_application_query_binding_one_shot<B>(
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
            .execution_request_for(self.product_runtime())
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
}

impl<
        S: ApplicationSchema,
        HostRuntime: crate::domain_computation::primary_graph::application_query::host::ApplicationQueryHost<
                S,
            > + ?Sized,
    > ReceiverOps<S> for HostRuntime
{
}

trait ReceiverOwnOps<S: ApplicationSchema>:
    crate::domain_computation::primary_graph::application_query::host::ApplicationQueryHost<S>
{
    fn execute_managed_parent<B>(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        binding: &WorthQueryInstalledApplicationQueryBinding<S, B>,
        external: &WorthQueryAuthenticatedExternalPrincipal<S>,
        mut plan: BoundPlan<'_, S, B>,
        allocation: &Arc<SourceFactsAllocation>,
    ) -> Result<WorthQueryApplicationOneShotResult<B::Query, ResultValue<S, B>>, ReceiverCause>
    where
        B: ApplicationQueryBinding<S>,
        ResultValue<S, B>: WorthQueryApplicationProjection<S, B::Query>,
    {
        let context = context::prepare(
            execution,
            self.application_query_host(),
            binding,
            &mut plan,
            allocation,
        )?;
        let read = self.read_managed_parent::<B>(
            execution,
            plan,
            context,
            external,
            allocation,
            ManagedParentCapture::None,
        )?;
        self.finish_managed_parent::<B>(
            execution,
            read,
            watch::ManagedParentCompletionCapture::None,
        )
        .map(|completed| completed.read)
    }

    fn finish_managed_parent<B>(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        read: ManagedParentRead<'_, S, B>,
        capture: watch::ManagedParentCompletionCapture,
    ) -> Result<
        one_shot::outcome::CompletedOneShotRead<
            B::Query,
            ResultValue<S, B>,
            watch::ManagedParentCompletionCapture,
        >,
        ReceiverCause,
    >
    where
        B: ApplicationQueryBinding<S>,
        ResultValue<S, B>: WorthQueryApplicationProjection<S, B::Query>,
    {
        let decode = crate::domain_computation::primary_graph::value_decode::ValueDecodeState::new(
            self.installed_schema().value_bindings().clone(),
        );
        one_shot::outcome::complete_one_shot_with_capture(
            execution,
            self.application_query_host(),
            &decode,
            read.plan,
            read.kernel,
            read.authorization,
            read.proof,
            |kernel, governance, request, query| {
                read_execution::project_non_live_kernel_with_decode::<
                    S,
                    B::Query,
                    ResultValue<S, B>,
                    _,
                >(
                    execution,
                    &decode,
                    kernel,
                    governance,
                    || one_shot::plan_admission::admit_request(request, query),
                    |projection| {
                        one_shot::denial(
                            crate::domain_computation::primary_graph::vocabulary::application_query::one_shot_denial::WorthQueryApplicationOneShotDenialKind::Projection(
                                projection.kind(),
                            ),
                            query,
                            projection.subject(),
                        )
                    },
                )
            },
            |_, _| Ok(capture),
        )
    }

    fn read_managed_parent<'plan, B>(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        plan: BoundPlan<'plan, S, B>,
        context: context::ParentContext<'_, S>,
        external: &WorthQueryAuthenticatedExternalPrincipal<S>,
        allocation: &Arc<SourceFactsAllocation>,
        capture: ManagedParentCapture,
    ) -> Result<ManagedParentRead<'plan, S, B>, ReceiverCause>
    where
        B: ApplicationQueryBinding<S>,
        ResultValue<S, B>: WorthQueryApplicationProjection<S, B::Query>,
    {
        let adapter = self
            .managed_indexes()
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
                self.application_query_host(),
                &plan.graph_work,
                plan.controls.request_scope(),
                &context.association,
                &context.selected,
                external,
                &context.selector,
                probe,
                &plan.parameters,
                &plan.governance,
                allocation,
            )
            .map_err(ReceiverCause::Provider)?;
        let rows = lease
            .rows(&plan.graph_work, plan.controls.request_scope())
            .map_err(|cause| ReceiverCause::Provider(ProviderExecutionDenial::Selection(cause)))?;
        let (kernel, authorization, proof) = execute_authorized_read(
            execution,
            self.application_query_host(),
            &plan,
            |runtime, graph, plan| {
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
            },
        )
        .map_err(|cause| {
            ReceiverCause::Read(crate::domain_computation::primary_graph::vocabulary::application_query::one_shot::map_authorized_read_denial(
                cause,
                plan.query.name(),
            ))
        })?;
        lease
            .rows(&plan.graph_work, plan.controls.request_scope())
            .map_err(|cause| ReceiverCause::Provider(ProviderExecutionDenial::Selection(cause)))?;
        let selection = match capture {
            ManagedParentCapture::None => None,
            ManagedParentCapture::Watch => Some(
                lease
                    .capture_watch(&plan.graph_work, plan.controls.request_scope())
                    .map_err(|cause| {
                        ReceiverCause::Provider(ProviderExecutionDenial::Selection(cause))
                    })?,
            ),
        };
        // The genuine SDK request/session loan never enters the Sync read plan
        // and ends before the Parent's consuming Query terminal.
        drop(lease);
        Ok(ManagedParentRead {
            plan,
            kernel,
            authorization,
            proof,
            selection,
        })
    }
}

impl<
        S: ApplicationSchema,
        HostRuntime: crate::domain_computation::primary_graph::application_query::host::ApplicationQueryHost<
                S,
            > + ?Sized,
    > ReceiverOwnOps<S> for HostRuntime
{
}
pub use eviction::EvictionOps;
pub use lifecycle::LifecycleOps;
pub use watch::NextOps;
pub use watch::OpenOps;
