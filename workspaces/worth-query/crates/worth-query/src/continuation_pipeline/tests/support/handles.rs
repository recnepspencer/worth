use crate::application::{
    WorthQueryConfig, WorthQueryDeclarationBridgeContinuationMode,
    WorthQueryDeclarationBridgeContinuationRequest, WorthQueryDeclarationBridgeTruthContext,
    WorthQueryDeclarationEnvelope, WorthQueryDeclarationFamilyMarker, WorthQueryDeclarationInput,
};
use crate::binding_pipeline::{
    WorthQueryBindingSourceKind, WorthQueryBindingSpecificity,
    WorthQueryContinuationBindingRequest, WorthQueryEnvelopeContextCandidate,
    WorthQueryResolveContinuationFromTargetRequest,
};

use super::{
    ContinuationDomain, ContinuationWorld, DriftedContinuationWorld, HistoricalFamily, Input,
    PreviewFamily, ReadmissionDrift, RuntimeFamily,
};

pub(crate) fn admitted_handle(
    request_execution: &worth_execution::ExecutionRequest<'_, '_>,
    world: &'static str,
) -> crate::application::WorthQueryInstalledDomainDeclarationContext<
    ContinuationDomain,
    ContinuationWorld,
> {
    configured_handle(
        request_execution,
        world,
        WorthQueryConfig::runtime_backed_default(),
    )
}

pub(crate) fn admitted_workspace(
    request_execution: &worth_execution::ExecutionRequest<'_, '_>,
    world: &'static str,
) -> (
    crate::runtime::WorthQueryWorkspace,
    crate::application::WorthQueryInstalledDomainDeclarationContext<
        ContinuationDomain,
        ContinuationWorld,
    >,
) {
    installed_workspace(request_execution, ContinuationWorld(world))
}

pub(crate) fn configured_handle(
    request_execution: &worth_execution::ExecutionRequest<'_, '_>,
    world: &'static str,
    _config: WorthQueryConfig,
) -> crate::application::WorthQueryInstalledDomainDeclarationContext<
    ContinuationDomain,
    ContinuationWorld,
> {
    installed_context(request_execution, ContinuationWorld(world))
}

pub(crate) fn drifted_readmission_handle_in(
    workspace: &crate::runtime::WorthQueryWorkspace,
    world: &'static str,
    drift: ReadmissionDrift,
) -> crate::application::WorthQueryInstalledDomainDeclarationContext<
    ContinuationDomain,
    DriftedContinuationWorld,
> {
    context_in_workspace(
        workspace,
        DriftedContinuationWorld {
            label: world,
            drift,
        },
    )
}

pub(crate) fn continuation_handle_in(
    workspace: &crate::runtime::WorthQueryWorkspace,
    world: &'static str,
) -> crate::application::WorthQueryInstalledDomainDeclarationContext<
    ContinuationDomain,
    ContinuationWorld,
> {
    context_in_workspace(workspace, ContinuationWorld(world))
}

fn installed_context<C>(
    request_execution: &worth_execution::ExecutionRequest<'_, '_>,
    context: C,
) -> crate::application::WorthQueryInstalledDomainDeclarationContext<ContinuationDomain, C>
where
    C: crate::application::WorthQueryDomainOperatingContext<ContinuationDomain>,
{
    let (_, context) = installed_workspace(request_execution, context);
    context
}

fn installed_workspace<C>(
    request_execution: &worth_execution::ExecutionRequest<'_, '_>,
    context: C,
) -> (
    crate::runtime::WorthQueryWorkspace,
    crate::application::WorthQueryInstalledDomainDeclarationContext<ContinuationDomain, C>,
)
where
    C: crate::application::WorthQueryDomainOperatingContext<ContinuationDomain>,
{
    crate::application::domain_test_support::installed_declaration_workspace(
        request_execution,
        ContinuationDomain,
        context,
        declaration_families(),
    )
}

fn context_in_workspace<C>(
    workspace: &crate::runtime::WorthQueryWorkspace,
    context: C,
) -> crate::application::WorthQueryInstalledDomainDeclarationContext<ContinuationDomain, C>
where
    C: crate::application::WorthQueryDomainOperatingContext<ContinuationDomain>,
{
    workspace
        .domain(ContinuationDomain)
        .expect("continuation test domain should be installed")
        .declarations_in(workspace, context)
        .expect("continuation test context should admit")
}

fn declaration_families(
) -> [crate::domain_installation::WorthQueryDomainDeclarationFamilyDefinition; 3] {
    [
        crate::application::domain_test_support::family::<ContinuationDomain, RuntimeFamily>(),
        crate::application::domain_test_support::family::<ContinuationDomain, HistoricalFamily>(),
        crate::application::domain_test_support::family::<ContinuationDomain, PreviewFamily>(),
    ]
}

pub(crate) fn runtime_route_request() -> WorthQueryDeclarationBridgeContinuationRequest {
    WorthQueryDeclarationBridgeContinuationRequest::new(
        WorthQueryDeclarationBridgeContinuationMode::RuntimeRoute,
        WorthQueryDeclarationBridgeTruthContext::Current,
    )
}

pub(crate) fn historical_truth_view_request() -> WorthQueryDeclarationBridgeContinuationRequest {
    WorthQueryDeclarationBridgeContinuationRequest::new(
        WorthQueryDeclarationBridgeContinuationMode::TruthView,
        WorthQueryDeclarationBridgeTruthContext::Historical,
    )
}

pub(crate) fn preview_session_request() -> WorthQueryDeclarationBridgeContinuationRequest {
    WorthQueryDeclarationBridgeContinuationRequest::new(
        WorthQueryDeclarationBridgeContinuationMode::PreviewSession,
        WorthQueryDeclarationBridgeTruthContext::Preview,
    )
}

pub(crate) fn envelope<I>(
    handle: &crate::application::WorthQueryInstalledDomainDeclarationContext<
        ContinuationDomain,
        ContinuationWorld,
    >,
    id: &'static str,
) -> WorthQueryDeclarationEnvelope<ContinuationDomain, Input<I>>
where
    Input<I>: WorthQueryDeclarationInput<ContinuationDomain, Family = I>,
    I: WorthQueryDeclarationFamilyMarker<ContinuationDomain>,
{
    let progressed = handle
        .declare_review_and_progress(Input::<I>::new(id))
        .unwrap_or_else(|_| panic!("expected progressed continuation declaration"));
    handle
        .envelope_routes_from_progressed(progressed)
        .unwrap_or_else(|_| panic!("expected envelope"))
}

pub(crate) fn target_request<I>(
    handle: &crate::application::WorthQueryInstalledDomainDeclarationContext<
        ContinuationDomain,
        ContinuationWorld,
    >,
    id: &'static str,
    bridge_request: WorthQueryDeclarationBridgeContinuationRequest,
) -> WorthQueryResolveContinuationFromTargetRequest<ContinuationDomain, Input<I>>
where
    Input<I>: WorthQueryDeclarationInput<ContinuationDomain, Family = I>,
    I: WorthQueryDeclarationFamilyMarker<ContinuationDomain>,
{
    WorthQueryResolveContinuationFromTargetRequest::new(
        envelope::<I>(handle, id),
        I::aspect_contract(),
    )
    .with_bridge_request(bridge_request)
}

pub(crate) fn context_request(
    handle: &crate::application::WorthQueryInstalledDomainDeclarationContext<
        ContinuationDomain,
        ContinuationWorld,
    >,
    id: &'static str,
) -> WorthQueryContinuationBindingRequest<ContinuationDomain, Input<RuntimeFamily>> {
    WorthQueryContinuationBindingRequest::new(
        vec![WorthQueryEnvelopeContextCandidate::new(
            "current envelope",
            WorthQueryBindingSourceKind::CurrentEnvelope,
            WorthQueryBindingSpecificity::TypedCurrentArtifact,
            envelope::<RuntimeFamily>(handle, id),
        )],
        RuntimeFamily::aspect_contract(),
        vec![WorthQueryBindingSourceKind::CurrentEnvelope],
    )
    .with_bridge_request(runtime_route_request())
}
