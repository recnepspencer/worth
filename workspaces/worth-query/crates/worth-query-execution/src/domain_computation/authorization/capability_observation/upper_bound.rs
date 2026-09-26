use super::*;

pub(in crate::domain_computation::authorization) fn observe_upper_bound_policy(
    _permit: super::super::delegation_admission::WorthQueryCapabilityObservationPermit,
    session_identity: crate::domain_computation::provider_session::WorthQueryGraphWorkSessionIdentity,
    relational: &worth_relational::facade::runtime::RelationalRuntime,
    snapshot: worth_relational::facade::snapshots::SnapshotHandle,
    bridge: &BridgeAuthorizationRuntime,
    installed: &WorthQueryInstalledCapabilityPlan,
    request: &WorthQueryRetainedCapabilityRequest,
    sample: &WorthQueryRuntimeTimeSample,
    exact_grant: worth_relational::facade::identity::EntityId,
) -> Result<WorthQueryObservedCapabilityDecision, WorthQueryOperationAuthorizationDenial> {
    let upper_bound = installed
        .upper_bound()
        .as_ref()
        .ok_or_else(|| invalid_policy(installed.contract().name()))?;
    projection_validation::validate_projection_shape(
        installed,
        request,
        upper_bound.path_count,
        false,
    )?;
    let paths = path_preparation::prepare_upper_bound_policy_paths(
        installed,
        request,
        sample,
        exact_grant,
    )?;
    let evidence = observe_exact_policy(relational, snapshot.clone(), installed, request, paths)?;
    let dependency_identity = *evidence.observation_identity().bytes();
    let observation = bridge_observation::lower_upper_bound_observation(
        installed,
        request,
        &evidence,
        dependency_identity,
    )?;
    let bridge_evidence = evaluate(bridge, installed, upper_bound, &evidence, observation)?;
    let observed_grant = extract_exact_grant(installed, &evidence)?;
    if observed_grant != exact_grant {
        return Err(WorthQueryOperationAuthorizationDenial::new(
            WorthQueryOperationAuthorizationDenialKind::InconsistentDecision,
            installed.contract().name(),
        ));
    }
    let expiry = expiry::observe_grant_expiry(relational, &snapshot, installed, observed_grant)?;
    Ok(WorthQueryObservedCapabilityDecision::new(
        WorthQueryAuthorizationDecisionFact::from_capability_observation(
            WorthQueryAuthorizationDecisionPermit::new(),
            session_identity,
            relational,
            evidence,
            bridge_evidence,
        )
        .map_err(|()| {
            WorthQueryOperationAuthorizationDenial::new(
                WorthQueryOperationAuthorizationDenialKind::InconsistentDecision,
                installed.contract().name(),
            )
        })?
        .with_preparatory_relational_work(expiry::observation_work()),
        observed_grant,
        std::sync::Arc::clone(installed.capability_authority_identity()),
        request.clone(),
        sample.clone(),
        expiry,
    ))
}

fn evaluate(
    bridge: &BridgeAuthorizationRuntime,
    installed: &WorthQueryInstalledCapabilityPlan,
    upper_bound: &super::super::capability_registry::WorthQueryCapabilityUpperBoundBindings,
    evidence: &worth_relational::facade::authorization::RelationalAuthorizationObservationEvidence,
    observation: worth_runtime_bridge::facade::BridgeAuthorizationObservation,
) -> Result<
    worth_runtime_bridge::facade::BridgeAuthorizationDecisionEvidence,
    WorthQueryOperationAuthorizationDenial,
> {
    let bridge_evidence = bridge.evaluate(observation).map_err(|_| {
        WorthQueryOperationAuthorizationDenial::new(
            WorthQueryOperationAuthorizationDenialKind::BridgeEvaluationRejected,
            installed.contract().name(),
        )
    })?;
    if bridge_evidence.dependency_identity() != evidence.observation_identity().bytes()
        || !bridge.retains(&bridge_evidence)
    {
        return Err(WorthQueryOperationAuthorizationDenial::new(
            WorthQueryOperationAuthorizationDenialKind::InconsistentDecision,
            installed.contract().name(),
        ));
    }
    if !bridge_evidence.is_allowed() {
        let causes = decision_denial::decision_denial_causes(
            &upper_bound.decision_rules,
            &bridge_evidence,
            None,
        )?;
        return Err(WorthQueryOperationAuthorizationDenial::from_ordered_causes(
            causes,
            installed.contract().name(),
        ));
    }
    Ok(bridge_evidence)
}
