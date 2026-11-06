use super::*;

pub(in crate::domain_capabilities::certification::reports) fn admitted_ready<P, T>(
    requested: WorthQueryRequestedDomainCapabilityContribution<P, T>,
) -> Result<
    crate::domain_capabilities::WorthQueryMaterializationReadyDomainCapabilityContribution<P, T>,
    IdentityDerivationDenial,
>
where
    P: crate::domain_capabilities::payloads::WorthQueryDomainCapabilityPayload,
    T: crate::domain_capabilities::targets::WorthQueryDomainCapabilityTargetBinding,
{
    let eligible = success(evaluate_requested_domain_capability_contribution(
        requested,
    )?);
    let admitted = success(admit_eligible_domain_capability_contribution(eligible)?);
    let target = admitted.payload().target().clone();
    Ok(success(
        prepare_admitted_domain_capability_contribution_for_materialization(admitted, target)?,
    ))
}

pub(in crate::domain_capabilities::certification::reports) fn success<T>(
    outcome: crate::domain_capabilities::WorthQueryDomainCapabilityTransitionOutcome<T>,
) -> T {
    match outcome {
        TransitionOutcome::Success(value) => value,
        _ => panic!("expected success"),
    }
}
