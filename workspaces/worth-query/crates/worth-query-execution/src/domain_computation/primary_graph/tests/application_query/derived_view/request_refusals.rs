//! Refusal precedes pair preparation; every request releases its transient memory.
use super::*;
use crate::domain_computation::primary_graph::tests::fixture::{
    PublicAccountMembershipQuery, PublicScopedAccountSummaryQuery,
};
use std::time::Instant;
use worth_execution::{
    CancellationSource, CancellationToken, ExecutionRequest, SerialMemoryBudget, SerialRequest,
};

#[test]
fn zero_memory_cancellation_and_expired_deadline_refuse_before_any_read() {
    let _owner = crate::domain_computation::primary_graph::tests::fixture::isolated_request_owner();
    let world = installed_authorization_world(true);
    let scope = live_scope();
    let selected = world.selected_product();
    let external = world.authenticate("alice", Duration::from_secs(60), &scope);
    let principal = selected
        .resolve_authenticated_principal(
            &world.binding,
            &external,
            &scope,
            WorthQueryPrincipalResolutionMode::Ordinary,
        )
        .unwrap();
    let root = selected
        .resolve_entity(
            AccountStatus::reference(),
            "open".to_owned(),
            &scope,
            WorthQueryPrincipalResolutionMode::Ordinary,
        )
        .unwrap();
    let access = WorthQueryApplicationQueryAccessContext::new(&principal, &root);
    let membership_query = world
        .application
        .installed_schema()
        .certification_query(PublicAccountMembershipQuery::reference())
        .unwrap();
    let query = world
        .application
        .installed_schema()
        .certification_query(PublicScopedAccountSummaryQuery::reference())
        .unwrap();
    let membership = world
        .application
        .execute_application_query_one_shot(
            selected
                .retain_selection()
                .unwrap()
                .admit_application_query(
                    &membership_query,
                    &access,
                    ApplicationQueryParameterSet::new(),
                    current_controls(&scope),
                )
                .unwrap(),
        )
        .unwrap();
    let cancelled = CancellationSource::new();
    cancelled.cancel();
    for workers in [None, Some(1), Some(2), Some(4)] {
        let requests = [
            SerialRequest::from_memory(SerialMemoryBudget::new(0), CancellationToken::new(), None),
            SerialRequest::from_memory(SerialMemoryBudget::new(1 << 20), cancelled.token(), None),
            SerialRequest::from_memory(
                SerialMemoryBudget::new(1 << 20),
                CancellationToken::new(),
                Some(Instant::now()),
            ),
        ];
        for (index, serial) in requests.into_iter().enumerate() {
            let lease = workers.map(|workers| {
                crate::domain_computation::primary_graph::application_contribution::test_authority()
                    .request_lease(worth_execution::LeaseRequest {
                        policy:
                            crate::domain_computation::primary_graph::application_contribution::test_policy(
                                std::num::NonZeroUsize::new(workers).unwrap(),
                                serial.memory().limit(),
                            ),
                        cancellation: if index == 1 {
                            cancelled.token()
                        } else {
                            CancellationToken::new()
                        },
                        deadline: (index == 2).then(Instant::now),
                    })
                    .unwrap()
            });
            let request = lease.as_ref().map_or_else(
                || ExecutionRequest::serial(&serial),
                ExecutionRequest::leased,
            );
            let view = world
                .application
                .open_managed_derived_collection_pair(
                    &ApplicationDerivedViewDefinition::new(
                        "request-refusal",
                        PublicAccountMembershipQuery::reference(),
                        ApplicationDerivedViewLimits::bounded(8, 32768),
                    ),
                    &membership_query,
                    &query,
                    &query,
                    selected.product(),
                )
                .unwrap();
            let (witness, _scope) =
            crate::domain_computation::primary_graph::application_query::DispatchWitness::install(
                None,
            );
            let mut prepared = 0;
            let result = world
                .application
                .reconstruct_managed_derived_collection_pair(
                    request,
                    &view,
                    selected.product(),
                    &membership,
                    |row| {
                        row.members
                            .iter()
                            .map(|root| (*root, row.tag.clone()))
                            .collect()
                    },
                    |_: &String| {
                        prepared += 1;
                        let first = selected
                            .retain_selection()
                            .unwrap()
                            .admit_application_query(
                                &query,
                                &access,
                                ApplicationQueryParameterSet::new(),
                                current_controls(&scope),
                            )
                            .unwrap();
                        let second = selected
                            .retain_selection()
                            .unwrap()
                            .admit_application_query(
                                &query,
                                &access,
                                ApplicationQueryParameterSet::new(),
                                current_controls(&scope),
                            )
                            .unwrap();
                        Ok(WorthQueryDerivedPairReadPlans::new(first, second))
                    },
                    |row| row.status().to_owned(),
                    |row| row.status().to_owned(),
                    |_, second| SceneLabel(second.label().to_owned()),
                );
            let denial = result.expect_err("request must be refused");
            match index {
                0 => assert!(matches!(
                    denial,
                    WorthQueryManagedDerivedViewDenial::PolicyMemoryExhausted(
                        worth_execution::MemoryLimitDenial {
                            admitted: 0,
                            level: worth_execution::MemoryLimitLevel::Policy { ancestor: 0 },
                            ..
                        }
                    )
                )),
                1 => assert_eq!(denial, WorthQueryManagedDerivedViewDenial::Cancelled),
                2 => assert_eq!(denial, WorthQueryManagedDerivedViewDenial::DeadlineElapsed),
                _ => unreachable!(),
            }
            assert_eq!(prepared, 0);
            assert!(witness.observations().0.is_empty());
            if let Some(lease) = &lease {
                drop(lease.reserve_memory(serial.memory().limit()).unwrap());
            } else {
                drop(serial.memory().reserve(serial.memory().limit()).unwrap());
            }
        }
    }
}
