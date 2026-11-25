//! The ordinary installed handler prepares one writer on Native's applied image.
use super::*;
use crate::domain_computation::primary_graph::InstalledProducerEditionExt;
use crate::domain_computation::primary_graph::{
    application_attempt::prepare_coupled_application, application_query::AppliedQuerySource,
};
use worth_query_execution_attempt::facade::ports::AdmittedOperationExecutionPort as _;
use worth_query_execution_attempt::facade::ports::AuthenticatedPrincipalInternalPort as _;
use worth_query_execution_attempt::facade::ports::EffectProgramMethodsInternalPort as _;
use worth_query_execution_attempt::facade::ports::PrimaryGraphProviderInternalPort as _;
use worth_query_execution_lineage::facade::ports::IdempotencyBindingInternalPort as _;
use worth_query_execution_lineage::facade::ports::PrimaryGraphIntegrationHandleInternalPort as _;
use worth_query_execution_lineage::facade::ports::PrimaryGraphInternalPort as _;
use worth_query_execution_runtime::facade::ports::ProductBranchLeaseInternalPort as _;
use worth_query_execution_runtime::facade::ports::RelationalSourceOwnerInternalPort as _;
use worth_relational::facade::runtime::{
    RelationalAppliedCandidateReadIdentity, RelationalPinnedRead,
};

mod handler_run;

pub fn prepare<Schema, Binding>(
    installed: &TypedInstalledProducer<Schema, Binding>,
    phase: &WorthQueryAdvancementPhase<'_>,
    coordination: &crate::domain_computation::primary_graph::provider::WorthQueryApplicationBranchCommitCoordination<'_>,
    runtime: &WorthQueryPrimaryGraphApplicationRuntime<Schema>,
    external: &WorthQueryAuthenticatedExternalPrincipal<Schema>,
    request: &WorthQueryRequestScope,
    before: &crate::basis::WorthQueryProductBranchLease,
    before_pin: &RelationalPinnedRead,
    candidate: &RelationalPinnedRead<RelationalAppliedCandidateReadIdentity>,
    own_outputs: Option<&crate::domain_computation::primary_graph::invariant_projection::read_basis::OwnOutputOverlay>,
    participants: &dyn crate::domain_computation::primary_graph::provider::AppliedOutputParticipants,
    assembly: &worth_relational::facade::mvcc::RelationalAuthoredCoupledCandidate<'_>,
    erased: ErasedAppliedProducerSource,
    predecessor: Option<&worth_relational::facade::mvcc::RelationalRequiredOutputPublication>,
    edition: super::super::InstalledProducerEdition,
    captured: Option<(
        super::input_preparation::PreparedProducerInputs<Schema, Binding>,
        std::any::TypeId,
    )>,
) -> Result<Box<dyn std::any::Any>, ProducerExecutionStop>
where
    Schema: ApplicationSchema,
    Binding: WorthQueryApplicationProducerBinding<Schema>,
    SourceValue<Schema, Binding>:
        crate::domain_computation::primary_graph::WorthQueryApplicationProjection<
            Schema,
            SourceQuery<Schema, Binding>,
        >,
{
    let foreign = || {
        denial(
            WorthQueryOutputDemandDenialKind::ForeignSource,
            Binding::IDENTITY,
        )
    };
    let source = erased
        .source
        .downcast::<AppliedQuerySource<SourceQuery<Schema, Binding>, SourceValue<Schema, Binding>>>(
        )
        .map_err(|_| foreign())?;
    let execution = phase
        .checked_execution_request_for(&runtime.runtime.product_runtime)
        .map_err(|stop| {
            denial(
                WorthQueryOutputDemandDenialKind::ExecutionRequest(stop),
                Binding::IDENTITY,
            )
        })?;
    let selected = runtime
        .on_branch(before.product_branch())
        .select_with_request(&execution)
        .map_err(|cause| failed(Binding::IDENTITY, cause))?;
    if selected.product().observation() != before.observation()
        || source.candidate.qualification() != candidate.candidate_identity().qualification()
        || !edition.admits_binding::<Schema, Binding>()
    {
        return Err(foreign().into());
    }
    let shared = selected
        .prepare_shared_query_basis()
        .map_err(|(_, stop)| source_readmission::ready_resource_denial(Binding::IDENTITY, stop))?;
    {
        // Ordinary selection registers locator-only checkpoint predecessors
        // before a handler reads them. Coupled execution reaches the handler
        // directly, and must use that same registration owner first.
        let bindings = runtime
            .installed_producers
            .family_output_bindings::<Binding::OutputFamily>();
        let scope = crate::domain_computation::authorization::WorthQueryOperationScopeEntityBinding::from_entity(source.root());
        let observation = crate::basis::WorthQueryProductBranchReadIdentity::from_observation(
            before.observation(),
        );
        let mut lineage = runtime
            .runtime
            .primary_provider
            .graph_ref()
            .output_lineage_ref()
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        runtime
            .record_recovered_output_candidates(
                Some(&execution),
                source.partition(),
                &source.checkpoint_identity(),
                &observation,
                scope,
                &bindings,
                &mut lineage,
            )
            .map_err(|stop| {
                crate::domain_computation::primary_graph::output_reuse::retained_fact_denial(
                    crate::domain_computation::primary_graph::output_reuse::preflight_stop(stop),
                    "coupled checkpoint predecessor registration",
                )
            })?;
    }
    let qualified = runtime.qualify_applied_output_source::<Binding::OutputFamily, _>(
        &source,
        before,
        erased.lifecycle,
    )?;
    let graph = runtime.runtime.primary_graph().ok_or_else(foreign)?;
    let selected_publication = graph.with_runtime(&execution, |native| {
        native
            .read_truth()
            .positioned_snapshot(shared.selected().application_basis().snapshot_handle())
            .map_err(|_| foreign())
    })?;
    runtime
        .runtime
        .primary_provider
        .graph_ref()
        .source_owner_ref()
        .mint_mark_cell_at_head(&execution, selected_publication.branch_id())
        .map_err(|stop| match stop {
            crate::domain_computation::primary_graph::output_lineage::HeadCellRegistrationStop::Admission(cause) => {
                crate::domain_computation::primary_graph::output_reuse::retained_fact_denial(
                    crate::domain_computation::primary_graph::output_reuse::preflight_stop(cause),
                    "coupled source publication mark-cell registration",
                )
            }
            stop => failed(Binding::IDENTITY, stop),
        })?;
    let input = installed.provider.operation_input(&source.value);
    let key = installed
        .provider
        .idempotency_key(&source.value, &source.idempotency_identity());
    let (identities, _recovery) = unpublished::prepare_input::<Schema, Binding>(
        &execution,
        key,
        input,
        edition,
        WorthQueryProducerCommitAuthority::ProgramOutput,
    )?;
    let identities = identities.borrow();
    let prepared = selected_public::prepare_applied_public_mutation::<Schema, Binding>(
        &execution,
        installed,
        runtime,
        &shared,
        before_pin,
        candidate,
        external,
        request,
        identities.mutation_input(),
    )
    .map_err(selected_public::preparation_stop::<Schema, Binding>)?;
    let mut admission = prepared.operation;
    let bound = source
        .bind_applied_expectation::<Schema, Operation<Schema, Binding>, _>(
            runtime,
            &execution,
            &mut admission,
            identities.mutation_input(),
            candidate,
        )
        .map_err(|cause| failed(Binding::IDENTITY, cause))?;
    let (mut program, _) = handler_run::run_coupled_producer_handler::<Schema, Binding>(
        installed,
        phase,
        coordination,
        runtime,
        external,
        request,
        &execution,
        &shared,
        before_pin,
        candidate,
        own_outputs,
        participants,
        assembly,
        &source,
        qualified,
        selected_publication,
        &identities,
        prepared.principal.principal(),
        prepared.scope.as_ref(),
        admission,
        edition,
        captured,
    )?;
    let implementation = worth_relational::facade::mvcc::RequiredOutputProducingImplementation::new(
        edition.build_identity(),
        edition.digest(),
    );
    let (key, dependency) = program
        .coupled_producer_identities::<Operation<Schema, Binding>>(
            runtime,
            &execution,
            *identities.key_identity(),
            implementation,
            predecessor,
            execution.allocation_policy(),
            runtime
                .installed_producers
                .entries
                .get(Binding::IDENTITY)
                .expect("installed executor retains declaration")
                .declaration
                .installed_output_equivalence
                .is_some(),
        )
        .map_err(|cause| denial::identity_failed(Binding::IDENTITY, cause))?;
    contact_observation::record_coupled_key(
        runtime.runtime.authority_identity().as_u64(),
        Binding::IDENTITY,
        source.root(),
        key,
    );
    let idempotency = bound
        .bind_idempotency(
            WorthQueryApplicationIdempotencyBinding::from_identities(
                key,
                *identities.input_identity(),
            )
            .bind_mutation::<Schema, Operation<Schema, Binding>>(),
        )
        .bind_producer_dependency(&dependency)
        .bind_producing_implementation(implementation);
    let program = program
        .with_output_demand_observation()
        .with_producer_required_invariants(Binding::REQUIRED_INVARIANTS);
    prepare_coupled_application::<Schema, Binding::Operation>(
        &execution,
        phase,
        runtime,
        coordination,
        program,
        idempotency,
        Some((*source).into_publication_source(erased.lifecycle)),
    )
    .map_err(|outcome| preparation_denial(Binding::IDENTITY, outcome))
}

pub fn preparation_denial(
    producer: &str,
    outcome: crate::domain_computation::primary_graph::WorthQueryApplicationCommitOutcome,
) -> ProducerExecutionStop {
    ProducerExecutionStop::ExecutionStopped(match commit_receipt(producer, outcome) {
        Err(denial) => denial,
        Ok(_) => denial(
            WorthQueryOutputDemandDenialKind::Superseded,
            producer.to_owned(),
        ),
    })
}
