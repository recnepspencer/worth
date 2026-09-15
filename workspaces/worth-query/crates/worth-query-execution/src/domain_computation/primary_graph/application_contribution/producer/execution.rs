use std::any::Any;
use std::marker::PhantomData;
use std::sync::Arc;

use worth_query_admission::facade::authenticated_principal::{
    WorthQueryAuthenticatedExternalPrincipal, WorthQueryRequestScope,
};
use worth_query_declaration::facade::application_operation::{
    ApplicationMutationBinding, ApplicationMutationIntent, ApplicationMutationScopeResolution,
};
use worth_query_declaration::facade::application_query::ApplicationQueryBinding;
use worth_query_declaration::facade::application_schema::{
    ApplicationSchema, ApplicationStructuredValueBinding, TypedMutationPreconditions,
};

use super::{
    WorthQueryApplicationProducerBinding, WorthQueryApplicationProducerProvider,
    WorthQueryProducerOutputFamily,
};
use crate::basis::WorthQueryProductBranch;
use crate::domain_computation::primary_graph::{
    HandlerResult, WorthQueryApplicationCommitDenialKind, WorthQueryApplicationCommitOutcome,
    WorthQueryApplicationCommitReceipt, WorthQueryApplicationIdempotencyBinding,
    WorthQueryApplicationIdempotencyResolution, WorthQueryApplicationNoEffectCause,
    WorthQueryObservedSource, WorthQueryPrimaryGraphApplicationRuntime,
    WorthQueryPrincipalResolutionMode,
};

use super::demand::{WorthQueryOutputDemandDenial, WorthQueryOutputDemandDenialKind};

mod denial;
use denial::{denial, execution_failed, failed};

type SourceBinding<Schema, Binding> =
    <<Binding as WorthQueryApplicationProducerBinding<Schema>>::OutputFamily as WorthQueryProducerOutputFamily<Schema>>::Source;
type SourceValue<Schema, Binding> = <<SourceBinding<Schema, Binding> as ApplicationQueryBinding<
    Schema,
>>::ResultBinding as ApplicationStructuredValueBinding>::Value;
type SourceQuery<Schema, Binding> =
    <SourceBinding<Schema, Binding> as ApplicationQueryBinding<Schema>>::Query;
type Operation<Schema, Binding> =
    <Binding as WorthQueryApplicationProducerBinding<Schema>>::Operation;

pub(super) trait InstalledProducerExecutor<Schema>: Send + Sync {
    fn authorize_interest(
        &self,
        runtime: &WorthQueryPrimaryGraphApplicationRuntime<Schema>,
        principal: &WorthQueryAuthenticatedExternalPrincipal<Schema>,
        request_scope: &WorthQueryRequestScope,
        branch: WorthQueryProductBranch,
        source: &dyn Any,
    ) -> Result<(), WorthQueryOutputDemandDenial>;

    fn execute(
        &self,
        runtime: &WorthQueryPrimaryGraphApplicationRuntime<Schema>,
        principal: &WorthQueryAuthenticatedExternalPrincipal<Schema>,
        request_scope: &WorthQueryRequestScope,
        branch: WorthQueryProductBranch,
        source: &dyn Any,
        observed_source: &dyn Any,
    ) -> Result<WorthQueryApplicationCommitReceipt, WorthQueryOutputDemandDenial>;

    fn resources(&self, source: &dyn Any) -> Option<super::WorthQueryProducerDemandResources>;

    fn readiness_record(
        &self,
        receipt: &WorthQueryApplicationCommitReceipt,
    ) -> Result<
        worth_runtime_bridge::facade::RelationalBridgeRecordIdentityParts,
        WorthQueryOutputDemandDenial,
    >;
}

pub(super) struct TypedInstalledProducer<Schema, Binding>
where
    Schema: ApplicationSchema,
    Binding: WorthQueryApplicationProducerBinding<Schema>,
{
    provider: Arc<Binding::Provider>,
    marker: PhantomData<fn() -> Schema>,
}

impl<Schema, Binding> TypedInstalledProducer<Schema, Binding>
where
    Schema: ApplicationSchema,
    Binding: WorthQueryApplicationProducerBinding<Schema>,
{
    pub(super) fn new(provider: Arc<Binding::Provider>) -> Self {
        Self {
            provider,
            marker: PhantomData,
        }
    }
}

impl<Schema, Binding> InstalledProducerExecutor<Schema> for TypedInstalledProducer<Schema, Binding>
where
    Schema: ApplicationSchema + 'static,
    Binding: WorthQueryApplicationProducerBinding<Schema>,
    Binding::Provider: WorthQueryApplicationProducerProvider<Schema, Binding>,
    SourceValue<Schema, Binding>: 'static,
    SourceQuery<Schema, Binding>: 'static,
{
    fn readiness_record(
        &self,
        receipt: &WorthQueryApplicationCommitReceipt,
    ) -> Result<
        worth_runtime_bridge::facade::RelationalBridgeRecordIdentityParts,
        WorthQueryOutputDemandDenial,
    > {
        let entity = receipt
            .output_correspondence()
            .entity_for_binding_role::<Binding::Operation>(Binding::OUTPUT_ROLE)
            .map_err(|error| failed(Binding::IDENTITY, error))?;
        Ok(
            worth_runtime_bridge::facade::RelationalBridgeRecordIdentityParts::entity(
                entity.partition_id.0,
                entity.local_slot.0,
                entity.generation.0,
            ),
        )
    }

    fn authorize_interest(
        &self,
        runtime: &WorthQueryPrimaryGraphApplicationRuntime<Schema>,
        principal: &WorthQueryAuthenticatedExternalPrincipal<Schema>,
        request_scope: &WorthQueryRequestScope,
        branch: WorthQueryProductBranch,
        source: &dyn Any,
    ) -> Result<(), WorthQueryOutputDemandDenial> {
        let source = source
            .downcast_ref::<SourceValue<Schema, Binding>>()
            .ok_or_else(|| {
                denial(
                    WorthQueryOutputDemandDenialKind::ForeignSource,
                    Binding::IDENTITY,
                )
            })?;
        authorize_typed::<Schema, Binding>(
            runtime,
            principal,
            request_scope,
            branch,
            self.provider.operation_input(source),
        )
    }

    fn resources(&self, source: &dyn Any) -> Option<super::WorthQueryProducerDemandResources> {
        source
            .downcast_ref::<SourceValue<Schema, Binding>>()
            .map(|source| self.provider.demand_resources(source))
    }

    fn execute(
        &self,
        runtime: &WorthQueryPrimaryGraphApplicationRuntime<Schema>,
        principal: &WorthQueryAuthenticatedExternalPrincipal<Schema>,
        request_scope: &WorthQueryRequestScope,
        branch: WorthQueryProductBranch,
        source: &dyn Any,
        observed_source: &dyn Any,
    ) -> Result<WorthQueryApplicationCommitReceipt, WorthQueryOutputDemandDenial> {
        let source = source
            .downcast_ref::<SourceValue<Schema, Binding>>()
            .ok_or_else(|| {
                denial(
                    WorthQueryOutputDemandDenialKind::ForeignSource,
                    Binding::IDENTITY,
                )
            })?;
        let observed_source = observed_source
            .downcast_ref::<WorthQueryObservedSource<SourceQuery<Schema, Binding>>>()
            .ok_or_else(|| {
                denial(
                    WorthQueryOutputDemandDenialKind::ForeignSource,
                    Binding::IDENTITY,
                )
            })?;
        execute_typed::<Schema, Binding>(
            runtime,
            principal,
            request_scope,
            branch,
            self.provider.as_ref(),
            source,
            observed_source.clone(),
        )
    }
}

fn authorize_typed<Schema, Binding>(
    runtime: &WorthQueryPrimaryGraphApplicationRuntime<Schema>,
    external_principal: &WorthQueryAuthenticatedExternalPrincipal<Schema>,
    request_scope: &WorthQueryRequestScope,
    branch: WorthQueryProductBranch,
    input: <Operation<Schema, Binding> as ApplicationMutationBinding<Schema>>::Input,
) -> Result<(), WorthQueryOutputDemandDenial>
where
    Schema: ApplicationSchema + 'static,
    Binding: WorthQueryApplicationProducerBinding<Schema>,
{
    let binding = runtime
        .installed_schema()
        .installed_mutation_binding::<Operation<Schema, Binding>>()
        .map_err(|error| failed(Binding::IDENTITY, error))?;
    let selected = runtime
        .on_branch(branch)
        .select()
        .map_err(|error| failed(Binding::IDENTITY, error))?;
    let principal = selected
        .resolve_authenticated_principal(
            binding.principal_binding(),
            external_principal,
            request_scope,
            WorthQueryPrincipalResolutionMode::Ordinary,
        )
        .map_err(|error| failed(Binding::IDENTITY, error))?;
    let (scope_field, scope_value) = input
        .scope_binding()
        .into_field_parts(principal.principal_identity());
    let scope = selected
        .resolve_entity(
            scope_field,
            scope_value,
            request_scope,
            WorthQueryPrincipalResolutionMode::Ordinary,
        )
        .map_err(|error| failed(Binding::IDENTITY, error))?;
    selected
        .authorize_operation(
            &principal,
            &scope,
            binding.operation(),
            TypedMutationPreconditions::default(),
            request_scope,
        )
        .map(|_| ())
        .map_err(|error| failed(Binding::IDENTITY, error))
}

fn execute_typed<Schema, Binding>(
    runtime: &WorthQueryPrimaryGraphApplicationRuntime<Schema>,
    external_principal: &WorthQueryAuthenticatedExternalPrincipal<Schema>,
    request_scope: &WorthQueryRequestScope,
    branch: WorthQueryProductBranch,
    provider: &Binding::Provider,
    source: &SourceValue<Schema, Binding>,
    observed_source: WorthQueryObservedSource<SourceQuery<Schema, Binding>>,
) -> Result<WorthQueryApplicationCommitReceipt, WorthQueryOutputDemandDenial>
where
    Schema: ApplicationSchema + 'static,
    Binding: WorthQueryApplicationProducerBinding<Schema>,
{
    let input = provider.operation_input(source);
    let source_epoch = observed_source.idempotency_identity();
    let key = provider.idempotency_key(source, &source_epoch);
    let binding = runtime
        .installed_schema()
        .installed_mutation_binding::<Operation<Schema, Binding>>()
        .map_err(|error| failed(Binding::IDENTITY, error))?;
    let selected = runtime
        .on_branch(branch)
        .select()
        .map_err(|error| failed(Binding::IDENTITY, error))?;
    let principal = selected
        .resolve_authenticated_principal(
            binding.principal_binding(),
            external_principal,
            request_scope,
            WorthQueryPrincipalResolutionMode::Ordinary,
        )
        .map_err(|error| failed(Binding::IDENTITY, error))?;
    let (scope_field, scope_value) = input
        .scope_binding()
        .into_field_parts(principal.principal_identity());
    let scope = selected
        .resolve_entity(
            scope_field,
            scope_value,
            request_scope,
            WorthQueryPrincipalResolutionMode::Ordinary,
        )
        .map_err(|error| failed(Binding::IDENTITY, error))?;
    let mut admission = selected
        .authorize_operation(
            &principal,
            &scope,
            binding.operation(),
            TypedMutationPreconditions::default(),
            request_scope,
        )
        .map_err(|error| failed(Binding::IDENTITY, error))?;
    let source_identity = runtime
        .bind_application_source_expectation::<Operation<Schema, Binding>, _>(
            &mut admission,
            observed_source,
        )
        .map_err(|error| failed(Binding::IDENTITY, error))?;
    let idempotency = WorthQueryApplicationIdempotencyBinding::new(
        Operation::<Schema, Binding>::idempotency_key_identity(&key),
        Operation::<Schema, Binding>::input_identity(&input),
    )
    .bind_source(Some(&source_identity));
    match runtime
        .resolve_admitted_application_idempotency(&admission, idempotency)
        .map_err(|error| failed(Binding::IDENTITY, error))?
        .into_resolution()
    {
        WorthQueryApplicationIdempotencyResolution::AlreadyCommitted(receipt) => {
            return Ok(receipt)
        }
        WorthQueryApplicationIdempotencyResolution::IntentDrift => {
            return Err(denial(
                WorthQueryOutputDemandDenialKind::ProducerUnavailable,
                Binding::IDENTITY,
            ))
        }
        WorthQueryApplicationIdempotencyResolution::Unseen => {}
    }
    let completed = match runtime
        .execute_mutation_handler::<Operation<Schema, Binding>>(&input, &key, admission)
        .map_err(|error| execution_failed(Binding::IDENTITY, error))?
    {
        HandlerResult::Completed(completed) => completed,
        HandlerResult::DomainDenied(domain_denial) => {
            return Err(denial(
                WorthQueryOutputDemandDenialKind::ProducerUnavailable,
                format!(
                    "{}: producer domain denial: {domain_denial:?}",
                    Binding::IDENTITY
                ),
            ))
        }
        HandlerResult::ExecutionDenied(error) => return Err(failed(Binding::IDENTITY, error)),
        HandlerResult::Cancelled => {
            return Err(denial(
                WorthQueryOutputDemandDenialKind::Cancelled,
                "producer cancelled",
            ))
        }
        HandlerResult::DeadlineExceeded => {
            return Err(denial(
                WorthQueryOutputDemandDenialKind::TimedOut,
                "producer deadline exceeded",
            ))
        }
    };
    let (program, _) = completed.into_parts();
    let program = program.with_output_demand_observation();
    match runtime.compare_and_commit_application(program, idempotency) {
        WorthQueryApplicationCommitOutcome::Committed(receipt)
        | WorthQueryApplicationCommitOutcome::AlreadyCommitted(receipt) => Ok(receipt),
        WorthQueryApplicationCommitOutcome::Stale(_)
        | WorthQueryApplicationCommitOutcome::ProductStale(_) => Err(denial(
            WorthQueryOutputDemandDenialKind::PublicationStale,
            Binding::IDENTITY,
        )),
        WorthQueryApplicationCommitOutcome::Cancelled => Err(denial(
            WorthQueryOutputDemandDenialKind::Cancelled,
            Binding::IDENTITY,
        )),
        WorthQueryApplicationCommitOutcome::TimedOut => Err(denial(
            WorthQueryOutputDemandDenialKind::TimedOut,
            Binding::IDENTITY,
        )),
        WorthQueryApplicationCommitOutcome::NoEffect(no_effect)
            if no_effect.cause() == WorthQueryApplicationNoEffectCause::CapacityExhausted =>
        {
            Err(denial(
                WorthQueryOutputDemandDenialKind::PublicationCapacityExceeded,
                Binding::IDENTITY,
            ))
        }
        WorthQueryApplicationCommitOutcome::Denied(commit_denial)
            if commit_denial.kind() == WorthQueryApplicationCommitDenialKind::ProductBasisStale =>
        {
            Err(denial(
                WorthQueryOutputDemandDenialKind::PublicationStale,
                Binding::IDENTITY,
            ))
        }
        outcome => Err(failed(Binding::IDENTITY, outcome)),
    }
}
