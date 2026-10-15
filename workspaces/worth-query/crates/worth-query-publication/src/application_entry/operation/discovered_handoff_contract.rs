//! Compiler-only bodies exercise the actual private descriptor/request/owner seam.
//! No handler execution or runtime evidence is claimed by these contracts.
use super::*;
use crate::application_entry::mutation::{
    WorthQueryApplicationDiscoveredMutationOutcome as Outcome,
    WorthQueryPerformedMutationExecutionDenial,
    WorthQueryUnpublishedDiscoveredApplicationMutation as Partial,
};
use worth_query_declaration::facade::application_operation::ApplicationMutationAdmissionBinding;
use worth_query_declaration::facade::application_program::{
    ApplicationActionDeclaration, ApplicationDiscoveredOutputRoot,
};
use worth_query_execution::facade::primary_graph::WorthQueryApplicationDiscoveredOutputConnection;
use worth_query_execution::facade::runtime::ExecutionAllocationPolicy;

// Deliberately has NO Clone implementation, even when its inner domain input does.
pub(super) struct NonCloneOriginal<Intent>(Intent);
impl<S: ApplicationSchema, I: ApplicationMutationIntent<S>> ApplicationMutationIntent<S>
    for NonCloneOriginal<I>
{
    type Binding = I::Binding;
    fn input(&self) -> &<I::Binding as ApplicationMutationBinding<S>>::Input {
        self.0.input()
    }
    fn scope_binding(&self) -> <I::Binding as ApplicationMutationBinding<S>>::ScopeBinding {
        self.0.scope_binding()
    }
}
// These are complete generic compiler bodies under precisely the existing intake bounds,
// not pseudocode, todo/unreachable values, or an assumed Clone/authority implementation.
// They retain the original descriptor NEXT TO the returned owner, making a self-borrow fail.
pub(super) fn borrowed_intake_then_fresh_retention_retry<S, I, P, R>(
    description: WorthQueryOperationDescription<S, NonCloneOriginal<I>>,
    request: &WorthQueryApplicationRequest<'_, '_, '_, S>,
    application: &WorthQueryProgramApplicationRuntime<S, P>,
    installed_action: &ApplicationActionDeclaration,
    phase: &WorthQueryAdvancementPhase<'_>,
    allocation: ExecutionAllocationPolicy<'_, '_>,
) -> (
    WorthQueryOperationDescription<S, NonCloneOriginal<I>>,
    Result<Outcome<S, NonCloneOriginal<I>, P, R>, WorthQueryPerformedMutationExecutionDenial>,
    Option<WorthQueryApplicationRecoveryRequestDenial>,
)
where
    S: ApplicationSchema + 'static,
    I: ApplicationMutationIntent<S>,
    I::Binding: ApplicationMutationAdmissionBinding<S>,
    <I::Binding as ApplicationMutationAdmissionBinding<S>>::Admission:
        crate::application_entry::mutation::authorization::WorthQueryBindingAdmission<
            S,
            I::Binding,
        >,
    <I::Binding as ApplicationMutationBinding<S>>::Input: Clone + Send + Sync,
    <I::Binding as ApplicationMutationBinding<S>>::ScopeBinding: ApplicationMutationScopeResolution<
        S,
        <I::Binding as ApplicationMutationBinding<S>>::PrincipalIdentity,
    >,
    P: ApplicationProgramDefinition<S>,
    P::Outputs: ApplicationProgramOutputsShape<S>,
    R: ApplicationOutputGraphShape<S> + ApplicationDiscoveredOutputRoot,
    Connection<S, R>: WorthQueryApplicationDiscoveredOutputConnection<S, Source = I::Binding>,
{
    let initial = description
        .request_view(request, installed_action)
        .execute_performed_discovered_in_advancement::<P, R>(phase, application, allocation)
        .map(|outcome| outcome.into_same_binding_intent::<NonCloneOriginal<I>>());
    let mut retry_denial = None;
    let stored = match initial {
        Ok(Outcome::RequiredOutputDenied(failure)) => {
            // This new request borrows the unchanged descriptor; the prior owner is moved,
            // temporarily given exactly that binding's borrowed nominal intent, then restored.
            match description
                .request_view(request, installed_action)
                .retry_discovered_output_retention_in_advancement::<P, R>(
                    phase,
                    failure.into_same_binding_intent::<&NonCloneOriginal<I>>(),
                    application,
                ) {
                Ok(performed) => Ok(Outcome::Performed(
                    performed.into_same_binding_intent::<NonCloneOriginal<I>>(),
                )),
                Err((denial, same_failure)) => {
                    retry_denial = Some(denial);
                    Ok(Outcome::RequiredOutputDenied(
                        same_failure.into_same_binding_intent::<NonCloneOriginal<I>>(),
                    ))
                }
            }
        }
        other => other,
    };
    (description, stored, retry_denial)
}

pub(super) fn fresh_partial_retry_returns_storable_owner<S, I, P, R>(
    description: WorthQueryOperationDescription<S, NonCloneOriginal<I>>,
    partial: Partial<S, NonCloneOriginal<I>, P, R>,
    request: &WorthQueryApplicationRequest<'_, '_, '_, S>,
    application: &WorthQueryProgramApplicationRuntime<S, P>,
    installed_action: &ApplicationActionDeclaration,
    phase: &WorthQueryAdvancementPhase<'_>,
) -> (
    WorthQueryOperationDescription<S, NonCloneOriginal<I>>,
    Partial<S, NonCloneOriginal<I>, P, R>,
    Result<
        crate::application_entry::mutation::WorthQueryDiscoveredRecoveryProgress,
        WorthQueryApplicationRecoveryRequestDenial,
    >,
)
where
    S: ApplicationSchema + 'static,
    I: ApplicationMutationIntent<S>,
    I::Binding: ApplicationMutationAdmissionBinding<S>,
    <I::Binding as ApplicationMutationAdmissionBinding<S>>::Admission:
        crate::application_entry::mutation::authorization::WorthQueryBindingAdmission<
            S,
            I::Binding,
        >,
    <I::Binding as ApplicationMutationBinding<S>>::Input: Clone + Send + Sync,
    <I::Binding as ApplicationMutationBinding<S>>::ScopeBinding: ApplicationMutationScopeResolution<
        S,
        <I::Binding as ApplicationMutationBinding<S>>::PrincipalIdentity,
    >,
    P: ApplicationProgramDefinition<S>,
    P::Outputs: ApplicationProgramOutputsShape<S>,
    R: ApplicationOutputGraphShape<S> + ApplicationDiscoveredOutputRoot,
    Connection<S, R>: WorthQueryApplicationDiscoveredOutputConnection<S, Source = I::Binding>,
{
    let mut borrowed = partial.into_same_binding_intent::<&NonCloneOriginal<I>>();
    let progress = description
        .request_view(request, installed_action)
        .recover_unpublished_discovered_in_program_in_advancement::<P, R>(
            phase,
            &mut borrowed,
            application,
        );
    let stored = borrowed.into_same_binding_intent::<NonCloneOriginal<I>>();
    (description, stored, progress)
}

// This body is typechecked: capture refusal returns the complete move-only original
// builder and key, while successful freeze returns a persistent original descriptor.
#[allow(
    clippy::type_complexity,
    reason = "Compiler-only witness keeps the exact move-only success and refusal owners visible"
)]
pub(super) fn capture_retains_non_clone_original<'a, 'p, 's, S, I>(
    builder: crate::application_entry::WorthQueryOperationBuilder<
        'a,
        'p,
        's,
        S,
        NonCloneOriginal<I>,
        crate::application_entry::WorthQueryMutationSourcePrepared,
    >,
    key: <I::Binding as ApplicationMutationBinding<S>>::IdempotencyKey,
) -> Result<
    WorthQueryOperationDescription<S, NonCloneOriginal<I>>,
    (
        crate::application_entry::WorthQueryOperationBuilder<
            'a,
            'p,
            's,
            S,
            NonCloneOriginal<I>,
            crate::application_entry::WorthQueryMutationSourcePrepared,
        >,
        <I::Binding as ApplicationMutationBinding<S>>::IdempotencyKey,
        WorthQueryOperationInputCaptureDenial,
    ),
>
where
    S: ApplicationSchema,
    I: ApplicationMutationIntent<S>,
    <I::Binding as ApplicationMutationBinding<S>>::Input: Clone + Send + Sync,
{
    builder.idempotency_owned(key)
}

// Private compiler witness: a real description can release its original Intent,
// conditions/source/key while an encoded sibling retains the SAME backing. The
// runtime capability court measures that backend's last-owner charge separately.
pub(super) fn shared_input_survives_description<S, I>(
    description: WorthQueryOperationDescription<S, NonCloneOriginal<I>>,
) -> worth_query_declaration::facade::application_operation::ApplicationEncodedInput<
    <I::Binding as ApplicationMutationBinding<S>>::InputBinding,
>
where
    S: ApplicationSchema,
    I: ApplicationMutationIntent<S>,
{
    let shared = description.encoded_input.as_ref().unwrap().clone();
    drop(description);
    shared
}

// Scope-independent failure transfer is a consuming move, not borrowed original data.
pub(super) fn preserve_capture_refusal<S: ApplicationSchema, I: ApplicationMutationIntent<S>>(
    builder: crate::application_entry::WorthQueryOperationBuilder<
        '_,
        '_,
        '_,
        S,
        NonCloneOriginal<I>,
        WorthQueryMutationSourcePrepared,
    >,
    key: <I::Binding as ApplicationMutationBinding<S>>::IdempotencyKey,
    denial: WorthQueryOperationInputCaptureDenial,
) -> WorthQueryOperationInputCaptureRefusal<S, NonCloneOriginal<I>> {
    builder.into_capture_refusal(key, denial)
}
