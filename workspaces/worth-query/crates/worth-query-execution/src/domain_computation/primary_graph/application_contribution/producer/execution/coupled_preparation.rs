//! The ordinary installed handler prepares one writer on Native's applied image.
use super::*;
use crate::domain_computation::primary_graph::{
    application_attempt::prepare_coupled_application, application_query::AppliedQuerySource,
};
use worth_relational::facade::runtime::{
    RelationalAppliedCandidateReadIdentity, RelationalPinnedRead,
};

pub(super) fn prepare<Schema, Binding>(
    installed: &TypedInstalledProducer<Schema, Binding>,
    phase: &WorthQueryAdvancementPhase<'_>,
    coordination: &crate::domain_computation::primary_graph::provider::WorthQueryApplicationBranchCommitCoordination<'_>,
    runtime: &WorthQueryPrimaryGraphApplicationRuntime<Schema>,
    external: &WorthQueryAuthenticatedExternalPrincipal<Schema>,
    request: &WorthQueryRequestScope,
    before: &crate::basis::WorthQueryProductBranchLease,
    before_pin: &RelationalPinnedRead,
    candidate: &RelationalPinnedRead<RelationalAppliedCandidateReadIdentity>,
    erased: ErasedAppliedProducerSource,
    predecessor: Option<&worth_relational::facade::mvcc::RelationalRequiredOutputPublication>,
    edition: super::super::InstalledProducerEdition,
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
            .graph
            .output_lineage
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
    admission.graph_work_mut().arm_coupled_handler();
    let predecessor_reads =
        super::super::predecessor_reads::ProducerPredecessorReadAccess::prepare_coupled::<
            Schema,
            Binding,
        >(
            admission
                .graph_work_mut()
                .coupled_handler_mut()
                .expect("armed handler"),
            source.checkpoint_identity(),
            edition,
        );
    let producer_request = admission.retain_admitted_request();
    let producer_principal = admission.operation_scope_binding().principal();
    let read_admission = prepared.scope.as_ref().map(|scope| {
        crate::domain_computation::authorization::BorrowedReadOperationAdmission::new(
            runtime,
            prepared.principal.principal(),
            scope,
            request,
            admission.graph_work().identity(),
        )
    });
    let source_query = crate::domain_computation::primary_graph::application_query::SourceQueryReaderAccess::ordinary(
        runtime, &shared, external, request,
    ).with_coordinated_control(
        crate::domain_computation::primary_graph::product_operation::current_read::CurrentReadControl::new(
            shared.selected().product().read_lease_ref(), &producer_request, &producer_principal,
        ), coordination,
    ).map_err(super::denial::request_admission_rejected)?
        .with_predecessor_reads(read_admission, predecessor_reads);
    let prepared_handler = runtime
        .prepare_mutation_handler_plan_on_basis::<Operation<Schema, Binding>>(
            phase,
            &identities,
            prepared.principal.principal().principal_identity(),
            Some(&source_query),
            admission,
            Some((before_pin, candidate)),
            || {
                contact_observation::record(
                    runtime.runtime.authority_identity().as_u64(),
                    source.root(),
                );
            },
            execution.allocation_policy(),
        );
    let report = match prepared_handler {
        Ok(handler) => handler.run(runtime, &identities),
        Err(report) => report,
    };
    if let Some(stop) = source_query.take_control_failure() {
        return Err(super::denial::request_admission_rejected(stop));
    }
    let completed = completed_handler::<
        _,
        _,
        <Binding::Provider as WorthQueryApplicationProducerProvider<Schema, Binding>>::DenialValue,
    >(
        Binding::IDENTITY,
        report
            .into_outcome()
            .map_err(|cause| execution_failed(Binding::IDENTITY, cause))?,
        |cause| installed.provider.domain_denial_reason(cause),
    )?;
    let (mut program, _) = completed.into_parts();
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
    let idempotency = bound
        .bind_idempotency(
            WorthQueryApplicationIdempotencyBinding::new(key, *identities.input_identity())
                .bind_mutation::<Schema, Operation<Schema, Binding>>(),
        )
        .bind_producer_dependency(&dependency)
        .bind_producing_implementation(implementation);
    let program = program
        .with_output_demand_observation()
        .with_producer_required_invariants(Binding::REQUIRED_INVARIANTS);
    prepare_coupled_application::<Schema, Binding>(
        &execution,
        phase,
        runtime,
        coordination,
        program,
        idempotency,
        (*source).into_publication_source(),
    )
    .map_err(|outcome| preparation_denial(Binding::IDENTITY, outcome))
}

pub(in crate::domain_computation::primary_graph::application_contribution::producer) fn preparation_denial(
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
