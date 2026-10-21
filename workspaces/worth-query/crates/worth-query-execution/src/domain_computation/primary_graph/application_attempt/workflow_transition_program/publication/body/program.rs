//! Extend the approval's actual readset, build its installed body, then append native effects.
use super::*;
use crate::domain_computation::primary_graph::{
    application_entry::mutation::{
        build_handler_candidate, DecisionReader, HandlerInterruption, HandlerResult,
    },
    DecisionContextUse,
};
use std::cell::Cell;

pub(super) fn compose<Schema, Binding>(
    access: &crate::publication_boundary::WorthQueryProgramPublicationAccess,
    runtime: &WorthQueryPrimaryGraphApplicationRuntime<Schema>,
    phase: &WorthQueryAdvancementPhase<'_>,
    identities: &ApplicationMutationIdentities<'_, Schema, Binding>,
    principal_identity: &Binding::PrincipalIdentity,
    program: BodyProgram<Schema, Binding>,
) -> Result<BodyProgram<Schema, Binding>, WorkflowApprovalBodyDenial>
where
    Schema: ApplicationSchema,
    Binding: ApplicationMutationBinding<
        Schema,
        Result = (),
        Denial = WorthQueryOperationBodyDenial,
        Decision = WorthQueryOperationBodyDecision<Schema, BodyScope<Schema, Binding>>,
    >,
{
    let handler = runtime
        .mutation_body_for_approval::<Binding>()
        .map_err(WorkflowApprovalBodyDenial::Handler)?;
    let execution = phase
        .execution_for(&runtime.product_runtime)
        .map_err(|cause| {
            WorkflowApprovalBodyDenial::Handler(
                MutationHandlerExecutionDenial::ForeignAdvancementPhase(cause),
            )
        })?;
    let request = phase
        .execution_request_for(&runtime.product_runtime)
        .map_err(|cause| {
            WorkflowApprovalBodyDenial::Handler(
                MutationHandlerExecutionDenial::ForeignAdvancementPhase(cause),
            )
        })?;
    let WorthQueryApplicationEffectProgram {
        mut read_set,
        effects,
        conditional_definition,
        effect_posture,
        output_correspondence,
        retain_output_demand_observation,
        retain_client_observation,
        producer_required_invariants,
        output_currentness_facts,
    } = program;
    // Native approval has no application outputs/emissions/conditional definition.
    // Refuse an unimplemented future profile rather than erase its carried state.
    if conditional_definition.is_some() || !output_correspondence.is_empty()
        || !producer_required_invariants.is_empty()
        || effect_posture != crate::domain_computation::provider_session::WorthQueryApplicationEffectPosture::Platform {
        return Err(WorkflowApprovalBodyDenial::UnsupportedProfile);
    }
    if read_set
        .admission
        .governed_input_identity()
        .is_some_and(|input| input != identities.input_identity())
    {
        return Err(WorkflowApprovalBodyDenial::Handler(
            MutationHandlerExecutionDenial::InputNotAdmitted,
        ));
    }
    let context = Cell::new(DecisionContextUse::default());
    let interruption = Cell::new(None);
    let operation_scope = read_set.admission.operation_scope_binding().clone();
    let (plan, projection, _) = runtime
        .mutation_projection
        .project_existing_operation(
            &request,
            &read_set.admission,
            &read_set.lease,
            |reader, scope| {
                let mut reader = DecisionReader::<Schema, Binding>::new(
                    reader,
                    scope,
                    principal_identity,
                    &operation_scope,
                    identities,
                    execution,
                    &context,
                    &interruption,
                );
                handler.prepare_decision(identities.mutation_input(), &mut reader)
            },
            ExecutionAllocationPolicy::SystemAllocation,
        )
        .map_err(|cause| {
            WorkflowApprovalBodyDenial::Handler(MutationHandlerExecutionDenial::Projection(cause))
        })?;
    // A decision can swallow its poll result, but cannot turn an observed
    // interruption into a prepared body decision.
    if let Some(interruption) = interruption.get() {
        return Err(match interruption {
            HandlerInterruption::Cancelled => WorkflowApprovalBodyDenial::Cancelled,
            HandlerInterruption::DeadlineExceeded => WorkflowApprovalBodyDenial::DeadlineExceeded,
        });
    }
    let decision = ready(plan.run(execution))?;
    read_set
        .admission
        .record_decision_context_use(context.get());
    read_set
        .complete_compiled_body_projection(
            &request,
            projection,
            ExecutionAllocationPolicy::SystemAllocation,
        )
        .map_err(WorkflowApprovalBodyDenial::Attempt)?;
    let completed = ready(
        build_handler_candidate(
            read_set,
            handler.as_ref(),
            identities,
            decision,
            &request,
            ExecutionAllocationPolicy::SystemAllocation,
        )
        .map_err(WorkflowApprovalBodyDenial::Handler)?,
    )?;
    let (mut body, ()) = completed.into_parts();
    if body.output_currentness_facts.is_some() {
        return Err(WorkflowApprovalBodyDenial::UnsupportedProfile);
    }
    body.effects
        .try_reserve(effects.len())
        .map_err(|_| WorkflowApprovalBodyDenial::UnsupportedProfile)?;
    // The body stays first: its native deferred signing slot retains its index.
    body.effects.extend(effects);
    body.effect_posture = crate::domain_computation::provider_session::WorthQueryApplicationEffectPosture::ApplicationWithPlatform;
    body.retain_output_demand_observation |= retain_output_demand_observation;
    body.retain_client_observation |= retain_client_observation;
    body.output_currentness_facts = output_currentness_facts;
    let body =
        super::super::super::super::signed_creation::WorthQuerySignedCreationAdapter::prepare::<
            Schema,
            Binding,
        >(access, phase, runtime, body, identities)
        .map_err(WorkflowApprovalBodyDenial::Signing)?;
    if super::super::super::super::signed_creation::WorthQuerySignedCreationAdapter::authentication_intent(&body).is_none() {
        return Err(WorkflowApprovalBodyDenial::UnsupportedProfile);
    }
    Ok(body)
}

fn ready<T>(
    outcome: HandlerResult<T, WorthQueryOperationBodyDenial>,
) -> Result<T, WorkflowApprovalBodyDenial> {
    match outcome {
        HandlerResult::Completed(value) => Ok(value),
        HandlerResult::DomainDenied(cause) => Err(WorkflowApprovalBodyDenial::Body(cause)),
        HandlerResult::ExecutionDenied(cause) => Err(WorkflowApprovalBodyDenial::Handler(
            MutationHandlerExecutionDenial::Handler(cause),
        )),
        HandlerResult::Cancelled => Err(WorkflowApprovalBodyDenial::Cancelled),
        HandlerResult::DeadlineExceeded => Err(WorkflowApprovalBodyDenial::DeadlineExceeded),
    }
}
