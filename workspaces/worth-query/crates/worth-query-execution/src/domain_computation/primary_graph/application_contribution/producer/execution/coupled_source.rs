//! Independent source-query admission for a declared unpublished owner demand.
use super::*;
use crate::domain_computation::primary_graph::application_attempt::coupled_outputs::DeclaredCoupledOutputDemand;
use crate::domain_computation::primary_graph::application_query::ApplicationQueryNativeAccess;
use crate::equality_type_id::EqualityTypeId;
use worth_query_declaration::facade::application_query::{
    ApplicationQueryBinding, ApplicationQueryParameterSet,
};
use worth_relational::facade::runtime::RelationalPinnedRead;
mod created;

pub(in crate::domain_computation::primary_graph::application_contribution) enum CoupledSourceReadStop
{
    Native(worth_relational::facade::mvcc::RelationalCoupledAssemblyFailure),
    Query(ProducerExecutionStop),
}
impl CoupledSourceReadStop {
    pub(in crate::domain_computation::primary_graph::application_contribution) fn capture(
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

pub(in crate::domain_computation::primary_graph::application_contribution) struct ErasedAppliedProducerSource
{
    pub(in crate::domain_computation::primary_graph::application_contribution) scope:
        worth_relational::facade::identity::EntityId,
    pub(in crate::domain_computation::primary_graph::application_contribution) partition: [u8; 32],
    pub(in crate::domain_computation::primary_graph::application_contribution) predecessor:
        Option<worth_relational::facade::mvcc::RelationalRequiredOutputPublication>,
    pub(in crate::domain_computation::primary_graph::application_contribution) profile:
        &'static str,
    pub(in crate::domain_computation::primary_graph::application_contribution) lifecycle:
        super::super::WorthQueryProducerLifecyclePosture,
    pub(in crate::domain_computation::primary_graph::application_contribution) source:
        Box<dyn std::any::Any>,
}

pub(in crate::domain_computation::primary_graph::application_contribution) enum CoupledSourceRead {
    Retiring,
    Prepared(usize),
    Verified(super::super::VerifiedAppliedParent),
    Source(ErasedAppliedProducerSource),
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
) -> Result<CoupledSourceRead, CoupledSourceReadStop>
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
            let captured = crate::domain_computation::primary_graph::application_query::capture_parameters::<Schema, SourceBinding<Schema, Binding>>(
                execution, runtime, 0, || Ok(parameters),
            ).map_err(CoupledSourceReadStop::capture)?;
            return created::read(installed, execution, runtime, external, request, selected, before_pin,
                &candidate, &arguments.value, &captured, cause, assembly, demand);

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
    if demand.reuse_prepared
        && let Some(index) =
            assembly.prepared_route_index(execution, demand.family, root, &partition)?
    {
        return Ok(CoupledSourceRead::Prepared(index));
    }
    let predecessor = match assembly.declared_owner(execution, demand.family, root, &partition)? {
        Some(worth_relational::facade::mvcc::RelationalDeclaredCoupledOwner::Retiring) => {
            return Ok(CoupledSourceRead::Retiring)
        }
        Some(worth_relational::facade::mvcc::RelationalDeclaredCoupledOwner::Surviving(owner)) => {
            Some(owner.clone())
        }
        None => None,
    };
    let source = runtime
        .project_applied_source_plan(execution, plan, &candidate)
        .map_err(|cause| failed(Binding::IDENTITY, cause))?;
    let predecessor = if predecessor.is_some() {
        predecessor
    } else {
        let graph = runtime.runtime.primary_graph().ok_or_else(foreign)?;
        let publisher = graph.application_publication.as_ref().ok_or_else(foreign)?;
        let meaning = worth_relational::facade::mvcc::RequiredOutputPublicationMeaning {
            implementation:
                worth_relational::facade::mvcc::RequiredOutputProducingImplementation::new(
                    runtime
                        .installed_producers
                        .entries
                        .get(Binding::IDENTITY)
                        .expect("installed source reader")
                        .edition
                        .build_identity(),
                    runtime
                        .installed_producers
                        .entries
                        .get(Binding::IDENTITY)
                        .expect("installed source reader")
                        .edition
                        .digest(),
                ),
            family: demand.family.into(),
            source_scope: root,
            source_partition: partition,
            source_epoch: source.checkpoint_identity(),
        };
        graph.with_runtime(execution, |native| match Binding::OWNERSHIP {
            worth_foundational::facade::ProducerPublicationOwnership::SourceScopeCascade => native
                .readmit_current_owned_output_successor(
                    execution,
                    &publisher.native,
                    before.relational_basis(),
                    Binding::IDENTITY,
                    &meaning,
                ),
            worth_foundational::facade::ProducerPublicationOwnership::Independent => native
                .readmit_current_independent_output_successor(
                    execution,
                    &publisher.native,
                    before.relational_basis(),
                    Binding::IDENTITY,
                    &meaning,
                ),
        })?
    };
    if demand.reuse_prepared
        && assembly.permits_unchanged_output_selection()
        && let Some(parent) = runtime.verify_applied_output_parent::<Binding::OutputFamily>(
            execution,
            request,
            before,
            &candidate,
            &source,
            predecessor.as_ref(),
        )?
    {
        return Ok(CoupledSourceRead::Verified(parent));
    }
    let profile = <Binding::OutputFamily as super::super::WorthQueryProducerOutputFamily<Schema>>::profile_kind(&source.value);
    let lifecycle = super::super::demand::selection::retained_liveness::applied_lifecycle::<
        Schema,
        Binding::OutputFamily,
    >(
        runtime,
        execution,
        before.observation(),
        source.root(),
        source.partition(),
        source.checkpoint_identity(),
        &candidate,
        predecessor.as_ref(),
    )?;
    Ok(CoupledSourceRead::Source(ErasedAppliedProducerSource {
        scope: source.root(),
        partition: source.partition(),
        predecessor,
        profile,
        lifecycle,
        source: Box::new(source),
    }))
}

use worth_query_declaration::facade::application_schema::fragment::RecordSelection;
