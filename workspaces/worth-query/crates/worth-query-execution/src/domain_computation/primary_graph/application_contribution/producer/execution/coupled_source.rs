//! Independent source-query admission for a declared unpublished owner demand.
use super::*;
use crate::domain_computation::primary_graph::application_attempt::coupled_outputs::DeclaredCoupledOutputDemand;
use crate::domain_computation::primary_graph::application_query::ApplicationQueryNativeAccess;
use crate::equality_type_id::EqualityTypeId;
use worth_query_declaration::facade::application_query::{
    ApplicationQueryBinding, ApplicationQueryParameterSet,
};
use worth_relational::facade::runtime::RelationalPinnedRead;

pub(in crate::domain_computation::primary_graph::application_contribution::producer) enum CoupledSourceReadStop
{
    Native(worth_relational::facade::mvcc::RelationalCoupledAssemblyFailure),
    Query(ProducerExecutionStop),
}
impl CoupledSourceReadStop {
    pub(in crate::domain_computation::primary_graph::application_contribution::producer) fn capture(
        cause: crate::domain_computation::primary_graph::WorthQueryApplicationAttemptDenial,
    ) -> Self {
        execution_failed(
            "declared coupled output",
            crate::domain_computation::primary_graph::MutationHandlerExecutionDenial::Attempt(
                cause,
            ),
        )
        .into()
    }
}
impl From<worth_relational::facade::mvcc::TransactionCommitError> for CoupledSourceReadStop {
    fn from(error: worth_relational::facade::mvcc::TransactionCommitError) -> Self {
        Self::Native(error.into())
    }
}
impl From<worth_relational::facade::mvcc::RelationalCoupledAssemblyFailure>
    for CoupledSourceReadStop
{
    fn from(error: worth_relational::facade::mvcc::RelationalCoupledAssemblyFailure) -> Self {
        Self::Native(error)
    }
}
impl From<ProducerExecutionStop> for CoupledSourceReadStop {
    fn from(error: ProducerExecutionStop) -> Self {
        Self::Query(error)
    }
}
impl From<WorthQueryOutputDemandDenial> for CoupledSourceReadStop {
    fn from(error: WorthQueryOutputDemandDenial) -> Self {
        Self::Query(error.into())
    }
}

pub(in crate::domain_computation::primary_graph::application_contribution::producer) struct ErasedAppliedProducerSource
{
    pub(in crate::domain_computation::primary_graph::application_contribution::producer) predecessor:
        worth_relational::facade::mvcc::RelationalRequiredOutputPublication,
    pub(in crate::domain_computation::primary_graph::application_contribution::producer) profile:
        &'static str,
    pub(in crate::domain_computation::primary_graph::application_contribution::producer) source:
        Box<dyn std::any::Any>,
}

pub(super) fn read<Schema, Binding>(
    installed: &TypedInstalledProducer<Schema, Binding>,
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    runtime: &WorthQueryPrimaryGraphApplicationRuntime<Schema>,
    external: &WorthQueryAuthenticatedExternalPrincipal<Schema>,
    request: &WorthQueryRequestScope,
    before: &crate::basis::WorthQueryProductBranchLease,
    before_pin: &RelationalPinnedRead,
    demand: &DeclaredCoupledOutputDemand,
    assembly: &worth_relational::facade::mvcc::RelationalAuthoredCoupledCandidate<'_>,
) -> Result<Option<ErasedAppliedProducerSource>, CoupledSourceReadStop>
where
    Schema: ApplicationSchema,
    Binding: WorthQueryApplicationProducerBinding<Schema>,
    SourceValue<Schema, Binding>:
        crate::domain_computation::primary_graph::WorthQueryApplicationProjection<
            Schema,
            SourceQuery<Schema, Binding>,
        >,
{
    let candidate = assembly.pin_source(execution)?;
    let foreign = || {
        denial(
            WorthQueryOutputDemandDenialKind::ForeignSource,
            demand.connection,
        )
    };
    if demand.family_type != EqualityTypeId::of::<Binding::OutputFamily>()
        || demand.source_binding_type != EqualityTypeId::of::<SourceBinding<Schema, Binding>>()
    {
        return Err(foreign().into());
    }
    let selected = runtime
        .on_branch(before.product_branch())
        .select_with_request(execution)
        .map_err(|cause| failed(Binding::IDENTITY, cause))?;
    if selected.product().observation() != before.observation()
        || candidate
            .candidate_identity()
            .proposal()
            .branch_observation()
            != before.relational_basis().reference()
    {
        return Err(foreign().into());
    }
    let principal = selected
        .resolve_authenticated_principal_admitted(
            execution,
            installed.source_query.principal_binding(),
            external,
            request,
            WorthQueryPrincipalResolutionMode::Ordinary,
        )
        .map_err(|cause| principal_rejected(Binding::IDENTITY, cause))?;
    let arguments = (demand.lower_source)(demand.demand.as_ref(), principal.principal_identity())
        .map_err(|cause| failed(Binding::IDENTITY, cause))?;
    let field = SourceBinding::<Schema, Binding>::scope_field();
    if arguments.entity != field.entity()
        || arguments.aspect != field.aspect()
        || arguments.field != field.field()
    {
        return Err(foreign().into());
    }
    let parameters = *arguments
        .parameters
        .downcast::<ApplicationQueryParameterSet<SourceQuery<Schema, Binding>>>()
        .map_err(|_| foreign())?;
    let scope = selected.resolve_query_scope_value_with_native_access(
        execution,
        field,
        None,
        &arguments.value,
        request,
        &mut ApplicationQueryNativeAccess::acquire(),
    );
    let scope = match scope {
        Ok(scope) => scope,
        Err(cause) if cause.kind() == crate::domain_computation::primary_graph::WorthQueryEntityResolutionDenialKind::Refused(
            crate::domain_computation::primary_graph::WorthQueryEntityResolutionRefusalKind::UnknownEntity,
        ) => {
            let graph = runtime.runtime.primary_graph().ok_or_else(foreign)?;
            let (resolved, _) = graph.retain_entity_resolution_context().resolve_applied_with_work(
                execution, before_pin, &candidate, field.entity(), field.aspect(), field.field(), &arguments.value,
            );
            let resolved = resolved.map_err(|cause| scope_rejected(Binding::IDENTITY, cause))?;
            let captured = crate::domain_computation::primary_graph::application_query::capture_parameters::<Schema, SourceBinding<Schema, Binding>>(
                execution, runtime, 0, || Ok(parameters),
            ).map_err(CoupledSourceReadStop::capture)?;
            match assembly.declared_owner(execution, demand.family, resolved.entity, captured.parameters().identity().bytes())? {
                worth_relational::facade::mvcc::RelationalDeclaredCoupledOwner::Retiring => return Ok(None),
                worth_relational::facade::mvcc::RelationalDeclaredCoupledOwner::Surviving(_) => return Err(foreign().into()),
            }
        }
        Err(cause) => return Err(scope_rejected(Binding::IDENTITY, cause).into()),
    };
    let (_, product, application_basis) = selected.into_parts();
    let query = installed.source_query.query();
    let maximum_results = runtime
        .resolve_application_query_limits(installed.source_query.limits())
        .maximum_results();
    let controls = crate::domain_computation::primary_graph::WorthQueryApplicationQueryControls::product_one_shot(product, application_basis, maximum_results, request);
    let access =
        crate::domain_computation::primary_graph::WorthQueryApplicationQueryAccessContext::new(
            &principal, &scope,
        );
    let plan = runtime
        .admit_application_query(execution, query, &access, parameters, controls)
        .map_err(|cause| query_admission_denied(Binding::IDENTITY, cause))?;
    let partition = *plan.parameter_binding_identity().bytes();
    let root = scope.entity_id();
    let predecessor = match assembly.declared_owner(execution, demand.family, root, &partition)? {
        worth_relational::facade::mvcc::RelationalDeclaredCoupledOwner::Retiring => {
            return Ok(None)
        }
        worth_relational::facade::mvcc::RelationalDeclaredCoupledOwner::Surviving(owner) => {
            owner.clone()
        }
    };
    let source = runtime
        .project_applied_source_plan(execution, plan, &candidate)
        .map_err(|cause| failed(Binding::IDENTITY, cause))?;
    let profile = <Binding::OutputFamily as super::super::WorthQueryProducerOutputFamily<Schema>>::profile_kind(&source.value);
    Ok(Some(ErasedAppliedProducerSource {
        predecessor,
        profile,
        source: Box::new(source),
    }))
}
