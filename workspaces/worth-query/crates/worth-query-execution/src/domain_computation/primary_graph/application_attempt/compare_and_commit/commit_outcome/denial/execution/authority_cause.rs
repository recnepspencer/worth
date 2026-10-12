//! The retained request authority exposes execution causes without changing ordinary authorization.

#[cfg(test)]
use crate::domain_computation::authorization::WorthQueryOperationAuthorizationRefusalKind as AuthorizationRefusal;
pub(super) fn authority_execution(
    kind: crate::domain_computation::authorization::WorthQueryOperationAuthorizationDenialKind,
) -> Option<
    Result<
        crate::domain_computation::WorthQueryProviderSessionDenialKind,
        crate::domain_computation::WorthQueryProviderSessionControlStopKind,
    >,
> {
    match kind {
        crate::domain_computation::authorization::WorthQueryOperationAuthorizationDenialKind::Stopped(stop) => Some(stop),
        crate::domain_computation::authorization::WorthQueryOperationAuthorizationDenialKind::Refused(_) => None,
    }
}

#[cfg(test)]
mod tests {
    use super::AuthorizationRefusal;
    use crate::domain_computation::primary_graph::{
        WorthQueryApplicationCommitDenial as Denial,
        WorthQueryApplicationCommitDenialStage as Stage,
        WorthQueryManagedComputationResourceDenial as Resource,
        WorthQueryMemoryLimitLevel as Level,
    };
    use crate::domain_computation::{
        authorization::{
            WorthQueryOperationAuthorizationDenial as Authorization,
            WorthQueryOperationAuthorizationDenialKind as Authority,
        },
        WorthQueryProviderSessionDenialKind as Kind,
    };
    #[test]
    fn authority_evidence_exposes_request_resources_by_value_and_keeps_ordinary_refusal() {
        for denial in [
            Resource::WorkExhausted,
            Resource::MemoryLimit {
                requested: 43,
                admitted: 17,
                level: Level::Policy,
            },
        ] {
            let policy_ancestor = if matches!(
                denial,
                Resource::MemoryLimit {
                    level: Level::Policy,
                    ..
                }
            ) {
                Some(2)
            } else {
                None
            };
            let kind = Kind::ExecutionResource {
                denial,
                partition_identity: None,
                policy_ancestor,
            };
            let authorization =
                Authorization::new(Authority::Stopped(Ok(kind)), "authorization read");
            let outcome = Denial::request_authority_denied(Stage::DecisionReadSet, &authorization);
            assert_eq!(outcome.execution_denial_cause(), Some(Ok(kind)));
            assert_eq!(
                outcome.request_authority(),
                Some(Authority::Stopped(Ok(kind)))
            );
            assert_eq!(outcome.stage(), Stage::DecisionReadSet);
        }
        let ordinary = Authorization::new(
            Authority::Refused(AuthorizationRefusal::ExplicitDenyRuleMatched),
            "authorization rule",
        );
        let outcome = Denial::request_authority_denied(Stage::DecisionReadSet, &ordinary);
        assert_eq!(outcome.execution_denial_cause(), None);
        assert_eq!(
            outcome.request_authority(),
            Some(Authority::Refused(
                AuthorizationRefusal::ExplicitDenyRuleMatched
            ))
        );
    }
}
