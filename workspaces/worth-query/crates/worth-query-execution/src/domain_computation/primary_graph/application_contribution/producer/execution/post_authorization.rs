//! Shared post-authorization producer progression for ordinary and selected Fresh.

mod preparation;
use super::*;
use crate::domain_computation::primary_graph::WorthQueryAdvancementPhase;
pub(super) use preparation::prepare_authorized;
use worth_query_declaration::facade::application_operation::{
    ApplicationMutationBinding, ApplicationMutationScopeBinding,
};

type MutationScope<Schema, Binding> = <<Operation<Schema, Binding> as ApplicationMutationBinding<
    Schema,
>>::ScopeBinding as ApplicationMutationScopeBinding<Schema>>::Scope;

#[allow(clippy::too_many_arguments)]
pub(super) fn execute_authorized<Schema, Binding>(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    phase: &WorthQueryAdvancementPhase<'_>,

    runtime: &WorthQueryPrimaryGraphApplicationRuntime<Schema>,
    selected: &crate::domain_computation::primary_graph::WorthQuerySelectedProductOperation<
        '_,
        Schema,
    >,
    principal: &crate::domain_computation::primary_graph::WorthQueryAuthenticatedPrincipal<
        Schema,
        <Operation<Schema, Binding> as ApplicationMutationBinding<Schema>>::Principal,
        <Operation<Schema, Binding> as ApplicationMutationBinding<Schema>>::PrincipalIdentity,
    >,
    admission: crate::domain_computation::authorization::WorthQueryAdmittedApplicationOperation<
        Schema,
        <Operation<Schema, Binding> as ApplicationMutationBinding<Schema>>::Operation,
        <Operation<Schema, Binding> as ApplicationMutationBinding<Schema>>::Input,
        MutationScope<Schema, Binding>,
    >,
    required_output: RequiredOutputDemandContext,
    provider: &std::sync::Arc<Binding::Provider>,
    source: SourceValue<Schema, Binding>,
    observed_source: crate::domain_computation::primary_graph::WorthQueryObservedSource<
        SourceQuery<Schema, Binding>,
    >,
    input: <Operation<Schema, Binding> as ApplicationMutationBinding<Schema>>::Input,
    matched_predecessors: Option<super::super::demand::MatchedRequiredPredecessors<'_>>,
    resources: super::super::WorthQueryProducerDemandResources,
    successor_of: Option<[u8; 32]>,
    commit_authority: WorthQueryProducerCommitAuthority,
    edition: super::super::InstalledProducerEdition,
    limits: WorthQueryOutputDemandLimits,
    request_admission: &mut InvalidationEditAdmission,
    producer_contacts: &mut usize,
) -> Result<ProducerExecutionOutcome, ProducerExecutionStop>
where
    Schema: ApplicationSchema + 'static,
    Binding: WorthQueryApplicationProducerBinding<Schema>,
    SourceValue<Schema, Binding>:
        crate::domain_computation::primary_graph::WorthQueryApplicationProjection<
            Schema,
            SourceQuery<Schema, Binding>,
        >,
{
    let compute_execution = phase.execution_for(&runtime.product_runtime).map_err(|_| {
        denial(
            WorthQueryOutputDemandDenialKind::ForeignSettlement,
            Binding::IDENTITY,
        )
    })?;
    let plan = prepare_authorized::<Schema, Binding>(
        *execution,
        phase,
        runtime,
        selected,
        principal,
        admission,
        required_output,
        std::sync::Arc::clone(provider),
        source,
        observed_source,
        input,
        matched_predecessors,
        resources,
        successor_of,
        commit_authority,
        edition,
        limits,
        request_admission,
        producer_contacts,
    )?;
    super::canonical_apply::run_one(plan, compute_execution, request_admission)
}
