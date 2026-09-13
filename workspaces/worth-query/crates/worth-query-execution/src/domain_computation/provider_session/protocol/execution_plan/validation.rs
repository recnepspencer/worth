use super::*;

pub(super) fn validate_common_authority(
    observation: WorthQueryProviderPlanAuthorityObservation<'_>,
    counters: &WorthQueryProviderSessionProtocolCounters,
) -> Result<(), WorthQueryProviderSessionFailure> {
    if !observation.operation.is_current_installation_generation() {
        return Err(failure(
            WorthQueryProviderSessionDenialKind::ForeignOperationAttempt,
            "provider plan operation belongs to a stale installation generation",
            counters,
        ));
    }
    if !observation.resource_authority_matches
        || observation.session_identity != observation.evidence_session_identity
        || observation.session_attempt_identity != observation.evidence_attempt_identity
    {
        return Err(failure(
            WorthQueryProviderSessionDenialKind::ForeignOperationAttempt,
            "provider plan inputs do not belong to the exact operation attempt",
            counters,
        ));
    }
    let intent = observation.bridge.managed_intent();
    if intent.operation_binding_identity() != observation.operation.binding_identity()
        || intent.resource_attempt_identity() != observation.session_attempt_identity
    {
        return Err(failure(
            WorthQueryProviderSessionDenialKind::ForeignExecutionBasis,
            "provider plan bridge basis belongs to a different managed intent",
            counters,
        ));
    }
    if !observation
        .operation
        .admits_provider_plan_graph(observation.stage_identity, observation.graph)
    {
        return Err(failure(
            WorthQueryProviderSessionDenialKind::ForeignGraphAuthority,
            "provider plan graph authority is not installed for this operation scope",
            counters,
        ));
    }
    Ok(())
}

pub(super) fn retain_session_provider(
    graph: &WorthQueryInstalledGraphParticipationAuthority,
    counters: &WorthQueryProviderSessionProtocolCounters,
) -> Result<Arc<WorthQueryGraphProviderAnchor>, WorthQueryProviderSessionFailure> {
    let provider = graph
        .retain_provider_anchor::<WorthQueryGraphProviderAnchor>()
        .ok_or_else(|| {
            failure(
                WorthQueryProviderSessionDenialKind::ProviderIdentityMismatch,
                "installed graph authority does not retain the Query provider anchor",
                counters,
            )
        })?;
    if provider.provider_identity() != graph.provider_identity() {
        return Err(failure(
            WorthQueryProviderSessionDenialKind::ProviderIdentityMismatch,
            "installed graph provider identity differs from the retained provider",
            counters,
        ));
    }
    if !provider.supports_session_protocol() {
        return Err(failure(
            WorthQueryProviderSessionDenialKind::SessionProtocolUnsupported,
            "installed graph provider does not implement the sealed session protocol",
            counters,
        ));
    }
    Ok(provider)
}

pub(super) fn undeclared_scope(
    counters: &WorthQueryProviderSessionProtocolCounters,
) -> WorthQueryProviderSessionFailure {
    failure(
        WorthQueryProviderSessionDenialKind::UndeclaredOperationScope,
        "provider plan scope is absent from the installed read/touch closure",
        counters,
    )
}

pub(super) fn failure(
    kind: WorthQueryProviderSessionDenialKind,
    detail: &'static str,
    counters: &WorthQueryProviderSessionProtocolCounters,
) -> WorthQueryProviderSessionFailure {
    WorthQueryProviderSessionFailure::new(
        kind,
        WorthQueryProviderSessionProtocolStage::PlanAdmission,
        detail,
        *counters,
    )
}
