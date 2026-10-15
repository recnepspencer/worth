//! The retained request authority exposes execution causes without changing ordinary authorization.

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
    use crate::domain_computation::primary_graph::tests::application_attempt::preimage_evidence::{
        retained_status_program, RetentionMutationBreadth,
    };
    use crate::domain_computation::primary_graph::tests::application_attempt::{
        authenticated_principal, idempotency, resolved_account,
    };
    use crate::domain_computation::primary_graph::tests::fixture::{
        installed_authorization_world, live_scope,
    };
    use crate::domain_computation::primary_graph::{
        WorthQueryApplicationCommitDenialStage as Stage,
        WorthQueryApplicationCommitOutcome as Outcome,
        WorthQueryManagedComputationResourceDenial as Resource,
    };
    use crate::domain_computation::{
        authorization::WorthQueryOperationAuthorizationDenialKind as Authority,
        WorthQueryProviderSessionControlStopKind as Control,
        WorthQueryProviderSessionDenialKind as Kind,
    };
    type Evidence = (Stage, Option<Result<Kind, Control>>, Option<Authority>);
    fn commit_at(ceiling: u64) -> (bool, Option<Evidence>, Vec<u64>) {
        let world = installed_authorization_world(true);
        let scope = live_scope();
        let program = world
            .application
            .with_host_advancement(|phase| {
                let host = phase.test_host_request();
                let principal = authenticated_principal(&host, &world, &scope);
                let account = resolved_account(&host, &world, "open", &scope);
                retained_status_program(
                    &host,
                    &world,
                    &principal,
                    &account,
                    &scope,
                    "authority-read-ceiling",
                    RetentionMutationBreadth::Narrow,
                )
            })
            .unwrap();
        world.application.with_application_advancement(&scope, |phase| {
            let observer = crate::test_read_stops::observe_consults_for_test(|_| {});
            let execution = phase.borrowed_request_for_test();
            let (outcome, _) = execution.run(worth_execution::ExecutionWorkCeiling::new(ceiling), |_| {
                world.application.compare_and_commit_application_in_advancement(&phase, program, idempotency(202, 203), worth_execution::ExecutionAllocationPolicy::SystemAllocation)
            }).unwrap();
            let charges = crate::test_read_stops::charged_prefixes_for_test(&observer).into_iter().map(|(_, required)| required).collect();
            match outcome {
                Outcome::Committed(_) => (true, None, charges),
                Outcome::Denied(denial) => (false, Some((denial.stage(), denial.execution_denial_cause(), denial.authorization_denial_kind())), charges),
                other => panic!("the real commit door has a committed or pre-effect denied answer: {other:?}"),
            }
        }).unwrap()
    }
    #[test]
    fn authority_evidence_exposes_request_resources_by_value_through_the_commit_door() {
        let (committed, evidence, charges) = commit_at(64_000_000);
        assert!(committed);
        assert_eq!(evidence, None);
        let expected = Kind::ExecutionResource {
            denial: Resource::WorkExhausted,
            partition_identity: None,
            policy_ancestor: None,
        };
        let mut reached = 0;
        let unique: std::collections::BTreeSet<_> = charges
            .into_iter()
            .filter(|required| *required > 0)
            .collect();
        for required in unique {
            let (committed, evidence, _) = commit_at(required - 1);
            assert!(!committed, "charge {required}");
            if let Some((Stage::DecisionReadSet, _, Some(Authority::Stopped(_)))) = evidence {
                assert_eq!(
                    evidence,
                    Some((
                        Stage::DecisionReadSet,
                        Some(Ok(expected)),
                        Some(Authority::Stopped(Ok(expected)))
                    )),
                    "charge {required}"
                );
                reached += 1;
            }
        }
        assert!(
            reached > 0,
            "the real authority revalidation read must be reached by its own charge"
        );
    }
}
