//! Map the authorization owner's refusal at ordinary Source consumers.
use {crate::domain_computation::authorization::SourceReadSupportCurrentnessStop as Support, crate::domain_computation::vocabulary::authorization::denial::WorthQueryOperationAuthorizationDenialKind as Authorization, crate::domain_computation::vocabulary::authorization::denial::WorthQueryOperationAuthorizationRefusalKind as Refusal, crate::domain_computation::primary_graph::vocabulary::application_attempt::source_currentness::WorthQuerySourceCurrentnessFailure as Currentness, crate::domain_computation::primary_graph::vocabulary::invariant_projection::work_budget::map_admission_stop, crate::domain_computation::primary_graph::vocabulary::invariant_projection::verification::ConsumedOutputVerificationStop as Stop};
pub(in crate::domain_computation::primary_graph) fn map_source_authorization_currentness(
    stop: Support,
) -> Stop {
    match stop {
        Support::PrincipalStale => {
            Stop::SourceQueryAuthorization(Authorization::Refused(Refusal::StalePrincipal))
        }
        Support::AuthorizationStale => {
            Stop::SourceQueryAuthorization(Authorization::Refused(Refusal::StaleAuthorization))
        }
        Support::Admission(stop) => map_admission_stop(stop),
        Support::Execution(cause) => Stop::Execution(cause),
        Support::PrincipalUnavailable => Stop::Unavailable,
        Support::PrincipalWorkExhausted => Stop::WorkExhausted,
    }
}

/// Keep private physical currentness vocabulary at its actual graph owner.
/// The neutral authorization carrier retains only the issuer's closed six facts.
pub(in crate::domain_computation) fn validate_source_principal_facts(
    facts: &[crate::domain_computation::primary_graph::WorthQueryApplicationObservedFact],
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    runtime: &worth_relational::facade::runtime::RelationalRuntime,
    snapshot: &worth_relational::facade::snapshots::SnapshotHandle,
) -> Result<(), Support> {
    use crate::domain_computation::primary_graph::vocabulary::application_attempt::movement::Movement;
    for fact in facts {
        let current = fact
            .source_currentness_in(runtime, execution, snapshot)
            .map_err(Support::Admission)?
            .map_err(|stop| match stop {
                Currentness::Execution(cause) => Support::Execution(cause),
                Currentness::WorkBudgetExceeded => Support::PrincipalWorkExhausted,
                Currentness::Unavailable => Support::PrincipalUnavailable,
                // The principal issuer emits no managed/output facts. Refuse an
                // unsupported issuer family; never treat it as current support.
                Currentness::ManagedIndex(_) | Currentness::OutputObservation(_) => {
                    Support::PrincipalUnavailable
                }
            })?;
        if current.movement() != Movement::Unmoved {
            return Err(Support::PrincipalStale);
        }
    }
    Ok(())
}
