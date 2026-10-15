//! Compiler-only start/promotion witnesses retain descriptor plus every refused native owner.
use super::discovered_handoff_contract::NonCloneOriginal;
use super::*;
use crate::application_entry::mutation::{
    WorthQueryDiscoveredOutputStartFailure as StartFailure,
    WorthQueryPerformedDiscoveredApplicationMutation as Performed,
    WorthQueryRecoveredDiscoveredOutputs as Recovered,
    WorthQueryStartedDiscoveredOutputs as Started,
    WorthQueryUnpublishedDiscoveredApplicationMutation as Partial,
};
use worth_query_declaration::facade::application_operation::ApplicationMutationAdmissionBinding;
use worth_query_declaration::facade::application_program::{
    ApplicationActionDeclaration, ApplicationDiscoveredOutputRoot,
};
use worth_query_execution::facade::primary_graph::WorthQueryApplicationDiscoveredOutputConnection;
type Discovery<S, R> =
    <Connection<S, R> as WorthQueryApplicationDiscoveredOutputConnection<S>>::Discovery;
type DiscoveryBinding<S, R> = <Discovery<S, R> as ApplicationQueryIntent<S>>::Binding;
type DiscoveryQuery<S, R> = <DiscoveryBinding<S, R> as ApplicationQueryBinding<S>>::Query;
type DiscoveryValue<S,R> = <<DiscoveryBinding<S,R> as ApplicationQueryBinding<S>>::ResultBinding as ApplicationStructuredValueBinding>::Value;
type RootDemand<S, R> =
    <Connection<S, R> as WorthQueryApplicationDiscoveredOutputConnection<S>>::Demand;
type RootSource<S,R> = <<RootDemand<S,R> as WorthQueryApplicationOutputDemand<S>>::OutputFamily as WorthQueryProducerOutputFamily<S>>::Source;
type RootQuery<S, R> = <RootSource<S, R> as ApplicationQueryBinding<S>>::Query;
type RootValue<S,R> = <<RootSource<S,R> as ApplicationQueryBinding<S>>::ResultBinding as ApplicationStructuredValueBinding>::Value;
pub(super) fn start_preserves_storable_owner_on_both_returns<S,I,P,R>(
    description: WorthQueryOperationDescription<S,NonCloneOriginal<I>>,
    performed: Performed<S,&NonCloneOriginal<I>,P,R>,
    request: &WorthQueryApplicationRequest<'_, '_, '_, S>,
    application: &WorthQueryProgramApplicationRuntime<S,P>,
    phase: &WorthQueryAdvancementPhase<'_>,
    controls: WorthQueryOutputDemandControls,
) -> (
    WorthQueryOperationDescription<S,NonCloneOriginal<I>>,
    Result<Started<S,NonCloneOriginal<I>,P,R>,StartFailure<S,NonCloneOriginal<I>,P,R>>,
)
where
    S: ApplicationSchema + 'static,
    I: ApplicationMutationIntent<S>,
    I::Binding: ApplicationMutationAdmissionBinding<S>,
    <I::Binding as ApplicationMutationAdmissionBinding<S>>::Admission: crate::application_entry::mutation::authorization::WorthQueryBindingAdmission<S,I::Binding>,
    <I::Binding as ApplicationMutationBinding<S>>::Input: Clone + Send + Sync,
    <I::Binding as ApplicationMutationBinding<S>>::ScopeBinding: ApplicationMutationScopeResolution<S,<I::Binding as ApplicationMutationBinding<S>>::PrincipalIdentity>,
    P: ApplicationProgramDefinition<S>, P::Outputs: ApplicationProgramOutputsShape<S>,
    R: ApplicationOutputGraphShape<S> + ApplicationDiscoveredOutputRoot,
    Connection<S,R>: WorthQueryApplicationDiscoveredOutputConnection<S,Source=I::Binding>,
    R::Dependents: crate::application_entry::mutation::program_output_continuation::ProgramOutputContinuationFactory<S,P,RootDemand<S,R>>,
    RootDemand<S,R>: Clone,
    DiscoveryValue<S,R>: WorthQueryApplicationProjection<S,DiscoveryQuery<S,R>> + Clone,
    <DiscoveryBinding<S,R> as ApplicationQueryBinding<S>>::ScopeBinding: ApplicationQueryScopeResolution<S,<DiscoveryBinding<S,R> as ApplicationQueryBinding<S>>::PrincipalIdentity>,
    <RootSource<S,R> as ApplicationQueryBinding<S>>::Input: ApplicationQueryIntent<S,Binding=RootSource<S,R>>,
    <RootSource<S,R> as ApplicationQueryBinding<S>>::ScopeBinding: ApplicationQueryScopeResolution<S,<RootSource<S,R> as ApplicationQueryBinding<S>>::PrincipalIdentity>,
    RootValue<S,R>: WorthQueryApplicationProjection<S,RootQuery<S,R>> + Clone,
{
    let started = performed
        .start_required_outputs_in_advancement(phase, application, request, controls)
        .map(|started| started.into_same_binding_intent::<NonCloneOriginal<I>>())
        .map_err(|refusal| refusal.into_same_binding_intent::<NonCloneOriginal<I>>());
    (description, started)
}

pub(super) fn fresh_partial_promotion_preserves_storable_refusal<S,I,P,R>(
    description: WorthQueryOperationDescription<S,NonCloneOriginal<I>>,
    partial: Partial<S,NonCloneOriginal<I>,P,R>,
    request: &WorthQueryApplicationRequest<'_, '_, '_, S>,
    application: &WorthQueryProgramApplicationRuntime<S,P>,
    installed_action: &ApplicationActionDeclaration,
    phase: &WorthQueryAdvancementPhase<'_>,
    controls: WorthQueryOutputDemandControls,
) -> (
    WorthQueryOperationDescription<S,NonCloneOriginal<I>>,
    Result<Recovered<S,P,R>,(WorthQueryApplicationRecoveryRequestDenial,Partial<S,NonCloneOriginal<I>,P,R>)>,
)
where
    S: ApplicationSchema + 'static,
    I: ApplicationMutationIntent<S>,
    I::Binding: ApplicationMutationAdmissionBinding<S>,
    <I::Binding as ApplicationMutationAdmissionBinding<S>>::Admission: crate::application_entry::mutation::authorization::WorthQueryBindingAdmission<S,I::Binding>,
    <I::Binding as ApplicationMutationBinding<S>>::Input: Clone + Send + Sync,
    <I::Binding as ApplicationMutationBinding<S>>::ScopeBinding: ApplicationMutationScopeResolution<S,<I::Binding as ApplicationMutationBinding<S>>::PrincipalIdentity>,
    P: ApplicationProgramDefinition<S>, P::Outputs: ApplicationProgramOutputsShape<S>,
    R: ApplicationOutputGraphShape<S> + ApplicationDiscoveredOutputRoot,
    Connection<S,R>: WorthQueryApplicationDiscoveredOutputConnection<S,Source=I::Binding>,
    R::Dependents: crate::application_entry::mutation::program_output_continuation::ProgramOutputContinuationFactory<S,P,RootDemand<S,R>>,
    RootDemand<S,R>: Clone,
    DiscoveryValue<S,R>: WorthQueryApplicationProjection<S,DiscoveryQuery<S,R>> + Clone,
    <DiscoveryBinding<S,R> as ApplicationQueryBinding<S>>::ScopeBinding: ApplicationQueryScopeResolution<S,<DiscoveryBinding<S,R> as ApplicationQueryBinding<S>>::PrincipalIdentity>,
    <RootSource<S,R> as ApplicationQueryBinding<S>>::Input: ApplicationQueryIntent<S,Binding=RootSource<S,R>>,
    <RootSource<S,R> as ApplicationQueryBinding<S>>::ScopeBinding: ApplicationQueryScopeResolution<S,<RootSource<S,R> as ApplicationQueryBinding<S>>::PrincipalIdentity>,
    RootValue<S,R>: WorthQueryApplicationProjection<S,RootQuery<S,R>> + Clone,
{
    let promoted = description
        .request_view(request, installed_action)
        .promote_recovered_discovered_outputs_in_advancement::<P, R>(
            phase,
            partial.into_same_binding_intent::<&NonCloneOriginal<I>>(),
            application,
            controls,
        )
        .map_err(|(denial, same_partial)| {
            (
                denial,
                same_partial.into_same_binding_intent::<NonCloneOriginal<I>>(),
            )
        });
    (description, promoted)
}
