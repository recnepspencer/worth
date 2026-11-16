//! Pure producer input reads complete under their actual before admission and D pin.
use super::*;
use crate::domain_computation::primary_graph::{
    application_query::AppliedQuerySource, MutationHandlerExecutionDenial,
    WorthQueryApplicationProjection, WorthQueryCompleteApplicationReadSet,
    WorthQueryProjectedApplicationMutation,
};
use worth_query_declaration::facade::application_operation::{
    ApplicationMutationBinding, ApplicationMutationScopeBinding, ApplicationOwnedMutationIdentities,
};
use worth_relational::facade::runtime::{
    RelationalAppliedCandidateReadIdentity, RelationalPinnedRead,
};

type Scope<S, B> = <<Operation<S, B> as ApplicationMutationBinding<S>>::ScopeBinding as ApplicationMutationScopeBinding<S>>::Scope;
type Op<S, B> = <Operation<S, B> as ApplicationMutationBinding<S>>::Operation;
type Input<S, B> = <Operation<S, B> as ApplicationMutationBinding<S>>::Input;

/// The original complete read set stays owned, not copied into a later admission.
pub(super) struct PreparedProducerInputs<S, B>
where
    S: ApplicationSchema,
    B: WorthQueryApplicationProducerBinding<S>,
{
    pub(super) inputs: super::super::input_capture::ErasedCapturedInputs,
    pub(super) reads: WorthQueryCompleteApplicationReadSet<
        S,
        Op<S, B>,
        Input<S, B>,
        Scope<S, B>,
        WorthQueryProjectedApplicationMutation,
    >,
    pub(super) identities: ApplicationOwnedMutationIdentities<S, Operation<S, B>>,
    pub(super) edition: super::super::InstalledProducerEdition,
    // Keeps the exact encoded input's original request allocation alive.
    _input: unpublished::PreparedProducerPublication,
}

impl<S: ApplicationSchema, B: WorthQueryApplicationProducerBinding<S>>
    PreparedProducerInputs<S, B>
{
    pub(super) fn into_mutation_inputs(
        self,
        installed: &TypedInstalledProducer<S, B>,
        identities: &worth_query_declaration::facade::application_operation::ApplicationMutationIdentities<'_, S, Operation<S, B>>,
        source: worth_relational::facade::identity::EntityId,
        edition: super::super::InstalledProducerEdition,
        component: std::any::TypeId,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        candidate: &RelationalPinnedRead<RelationalAppliedCandidateReadIdentity>,
    ) -> Result<crate::domain_computation::primary_graph::application_entry::mutation::PreparedMutationInputs<S, Operation<S, B>>, ProducerExecutionStop>{
        let foreign = || denial(WorthQueryOutputDemandDenialKind::ForeignSource, B::IDENTITY);
        let owner = installed.input_capture.ok_or_else(foreign)?;
        if self.edition != edition
            || self.identities.borrow().input_identity() != identities.input_identity()
            || self.inputs.affinity.source != source
            || self.inputs.affinity.component != component
        {
            return Err(foreign().into());
        }
        if !self
            .reads
            .captured_inputs_remain_equal(execution, candidate)
            .map_err(|cause| {
                execution_failed(B::IDENTITY, MutationHandlerExecutionDenial::Attempt(cause))
            })?
        {
            return Err(execution_failed(B::IDENTITY, MutationHandlerExecutionDenial::Attempt(
                crate::domain_computation::primary_graph::WorthQueryApplicationAttemptDenial::new(
                    crate::domain_computation::primary_graph::WorthQueryApplicationAttemptDenialKind::SourceChanged,
                    B::IDENTITY,
                ),
            )));
        }
        let (value, memory, context_use) = self.inputs.into_parts();
        Ok(crate::domain_computation::primary_graph::application_entry::mutation::PreparedMutationInputs::new(
            value, self.reads, memory, owner.prepare, context_use,
        ))
    }
}

pub(super) fn capture<S, B>(
    installed: &TypedInstalledProducer<S, B>,
    phase: &WorthQueryAdvancementPhase<'_>,
    runtime: &WorthQueryPrimaryGraphApplicationRuntime<S>,
    external: &WorthQueryAuthenticatedExternalPrincipal<S>,
    request: &WorthQueryRequestScope,
    before: &crate::basis::WorthQueryProductBranchLease,
    before_pin: &RelationalPinnedRead,
    candidate: &RelationalPinnedRead<RelationalAppliedCandidateReadIdentity>,
    erased: ErasedAppliedProducerSource,
    edition: super::super::InstalledProducerEdition,
    component: std::any::TypeId,
) -> Result<PreparedProducerInputs<S, B>, ProducerExecutionStop>
where
    S: ApplicationSchema,
    B: WorthQueryApplicationProducerBinding<S>,
    SourceValue<S, B>: WorthQueryApplicationProjection<S, SourceQuery<S, B>>,
{
    let foreign = || denial(WorthQueryOutputDemandDenialKind::ForeignSource, B::IDENTITY);
    let owner = installed.input_capture.ok_or_else(foreign)?;
    let declaration = &runtime
        .installed_producers
        .entries
        .get(B::IDENTITY)
        .ok_or_else(foreign)?
        .declaration;
    let meaning = declaration.input_capture.as_ref().ok_or_else(foreign)?;
    if meaning.definition != owner.definition || !edition.admits_binding::<S, B>() {
        return Err(foreign().into());
    }
    let source = erased
        .source
        .downcast::<AppliedQuerySource<SourceQuery<S, B>, SourceValue<S, B>>>()
        .map_err(|_| foreign())?;
    if source.candidate.qualification() != candidate.candidate_identity().qualification() {
        return Err(foreign().into());
    }
    let query_execution = phase
        .execution_for(&runtime.runtime.product_runtime)
        .map_err(|cause| {
            execution_failed(
                B::IDENTITY,
                MutationHandlerExecutionDenial::ForeignAdvancementPhase(cause),
            )
        })?;
    let execution = query_execution
        .execution_request()
        .map_err(|cause| {
            denial(
                WorthQueryOutputDemandDenialKind::ExecutionRequest(
                    crate::domain_computation::primary_graph::application_contribution::WorthQueryAdvancementDenial::Resource(cause),
                ),
                B::IDENTITY,
            )
        })?;
    let selected = runtime
        .on_branch(before.product_branch())
        .select_with_request(&execution)
        .map_err(|cause| failed(B::IDENTITY, cause))?;
    if selected.product().observation() != before.observation() {
        return Err(foreign().into());
    }
    let shared = selected
        .prepare_shared_query_basis()
        .map_err(|(_, stop)| source_readmission::ready_resource_denial(B::IDENTITY, stop))?;
    let input = installed.provider.operation_input(&source.value);
    let key = installed
        .provider
        .idempotency_key(&source.value, &source.idempotency_identity());
    let (identities, retained_input) = unpublished::prepare_input::<S, B>(
        &execution,
        key,
        input,
        edition,
        WorthQueryProducerCommitAuthority::ProgramOutput,
    )?;
    let borrowed = identities.borrow();
    let prepared = selected_public::prepare_applied_public_mutation::<S, B>(
        &execution,
        installed,
        runtime,
        &shared,
        before_pin,
        candidate,
        external,
        request,
        borrowed.mutation_input(),
    )
    .map_err(selected_public::preparation_stop::<S, B>)?;
    let mut admission = prepared.operation;
    admission
        .validate_mutation_entry(
            runtime.runtime.authority_identity(),
            &runtime.installed_schema.binding_identity(),
            &runtime.runtime.installed_operations,
        )
        .map_err(|cause| execution_failed(B::IDENTITY, cause))?;
    if admission
        .governed_input_identity()
        .is_some_and(|input| input != borrowed.input_identity())
    {
        return Err(execution_failed(
            B::IDENTITY,
            MutationHandlerExecutionDenial::InputNotAdmitted,
        ));
    }
    runtime
        .admit_directory_mutation::<Operation<S, B>>()
        .map_err(|cause| execution_failed(B::IDENTITY, cause))?;
    admission
        .validate_installed_mutation_binding(
            runtime
                .installed_schema
                .installed_mutation_binding_descriptor(
                    <Operation<S, B> as ApplicationMutationBinding<S>>::IDENTITY,
                ),
        )
        .map_err(|cause| execution_failed(B::IDENTITY, cause))?;
    let _source = source
        .bind_applied_expectation::<S, Operation<S, B>, _>(
            runtime,
            &execution,
            &mut admission,
            borrowed.mutation_input(),
            candidate,
        )
        .map_err(|cause| failed(B::IDENTITY, cause))?;
    runtime
        .validate_projected_source_facts(&execution, &admission)
        .map_err(|cause| {
            execution_failed(B::IDENTITY, MutationHandlerExecutionDenial::Attempt(cause))
        })?;
    let affinity = super::super::input_capture::InputCaptureAffinity {
        admission: admission.admission_identity(),
        candidate: candidate.candidate_identity().qualification(),
        source: source.root(),
        component,
    };
    let (result, projection, _) = runtime
        .mutation_projection
        .project_applied_operation(
            &execution,
            &admission,
            before_pin,
            candidate,
            None,
            |reader, scope| {
                (owner.capture)(
                    runtime,
                    borrowed.mutation_input(),
                    reader,
                    scope,
                    query_execution,
                    &meaning.types,
                    affinity,
                )
            },
            execution.allocation_policy(),
        )
        .map_err(|cause| {
            execution_failed(
                B::IDENTITY,
                MutationHandlerExecutionDenial::Projection(cause),
            )
        })?;
    let inputs = completed_handler::<
        _,
        _,
        <B::Provider as WorthQueryApplicationProducerProvider<S, B>>::DenialValue,
    >(B::IDENTITY, result, |cause| {
        installed.provider.domain_denial_reason(cause)
    })?;
    let reads = runtime
        .begin_applied_application_read_attempt(
            &execution,
            admission,
            projection,
            candidate.clone(),
            execution.allocation_policy(),
        )
        .and_then(|reads| {
            reads.complete_projected_dependencies_with_request(
                &execution,
                execution.allocation_policy(),
            )
        })
        .map_err(|cause| {
            execution_failed(B::IDENTITY, MutationHandlerExecutionDenial::Attempt(cause))
        })?;
    Ok(PreparedProducerInputs {
        inputs,
        reads,
        identities,
        edition,
        _input: retained_input,
    })
}
