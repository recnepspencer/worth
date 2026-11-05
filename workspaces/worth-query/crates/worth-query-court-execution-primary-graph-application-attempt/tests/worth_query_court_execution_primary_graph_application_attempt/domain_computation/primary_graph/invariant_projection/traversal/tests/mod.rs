use std::cell::Cell;

use worth_query_execution::facade::domain_computation::WorthQueryInvariantProjectionTraversalDenialKind;
use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::world_installation::installed_authorization_world;
use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::Account;
use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::AccountOwner;
use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::Principal;
use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::PrincipalIdentityField;
use worth_execution::CancellationSource;
use worth_execution::ExecutionRequest;
use worth_execution::SerialMemoryBudget;
use worth_execution::SerialRequest;

#[test]
fn partial_adjacency_reservations_are_not_endpoint_acquisitions() {
    let world = installed_authorization_world(
        true,
        worth_query_admission::facade::authenticated_principal::WorthQueryAdmissionClock::system(),
    );
    let (denial, stopped) = read_partial_adjacency(&world);
    assert_eq!(
        stopped,
        Some(WorthQueryInvariantProjectionTraversalDenialKind::WorkBudgetExceeded)
    );
    let work = denial
        .projection_work()
        .expect("the refused projection retains its observations");
    assert!(
        work.adjacency_edges_inspected() > 0,
        "the native scan actually began"
    );
    assert_eq!(
        work.endpoint_records_read(),
        0,
        "no endpoint lookup ran after the partial scan"
    );

    let completed = world
        .application
        .with_host_advancement(|phase| {
            world
                .invariant
                .project(
                    &phase,
                    |reader| {
                        let principal = reader
                            .resolve_entity(PrincipalIdentityField::reference(), 1_u64)
                            .unwrap();
                        reader
                            .relations_from(AccountOwner::reference(), &principal)
                            .unwrap()
                            .len()
                    },
                    worth_execution::ExecutionAllocationPolicy::SystemAllocation,
                )
                .unwrap()
        })
        .unwrap();
    assert!(*completed.output() > 0);
    assert_eq!(
        completed.work().endpoint_records_read(),
        *completed.output(),
        "the lawful twin acquires every returned endpoint"
    );
}

pub fn read_partial_adjacency(
    world: &worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::authorization_world_installation::AuthorizationWorld,
) -> (
    worth_query_execution::facade::domain_computation::WorthQueryInvariantProjectionDenial,
    Option<WorthQueryInvariantProjectionTraversalDenialKind>,
) {
    let host = worth_query_testkit_execution_primary_graph_application_attempt_2::test_request_policy::serial_request();
    let execution = ExecutionRequest::serial(&host);
    let selected = world
        .application
        .select_product_branch_for_test(
            &execution,
            world
                .application
                .product_runtime()
                .default_branch_for_test(),
        )
        .unwrap();
    let product = selected.product().publication_binding_for_test();
    let stopped = Cell::new(None);
    let denial = world
        .invariant
        .project_with_work_limit_for_test(
            &execution,
            Some(5),
            product
                .observation_for_test()
                .basis()
                .relational_basis()
                .clone(),
            |reader| {
                let principal = reader
                    .resolve_entity(PrincipalIdentityField::reference(), 1_u64)
                    .unwrap();
                let error = reader
                    .relations_from(AccountOwner::reference(), &principal)
                    .unwrap_err();
                stopped.set(Some(error.kind()));
            },
            None,
            worth_execution::ExecutionAllocationPolicy::SystemAllocation,
        )
        .err()
        .expect("the partial native read cannot seal a snapshot");
    (denial, stopped.get())
}

#[test]
fn acquired_endpoint_metadata_counts_even_when_its_kind_is_rejected() {
    let world = installed_authorization_world(
        true,
        worth_query_admission::facade::authenticated_principal::WorthQueryAdmissionClock::system(),
    );
    let completed = world
        .application
        .with_host_advancement(|phase| {
            world
                .invariant
                .project(
                    &phase,
                    |reader| {
                        let principal = reader
                            .resolve_entity(PrincipalIdentityField::reference(), 1_u64)
                            .unwrap();
                        let principal_kind = (*reader.layout_for_test())
                            .entity_kind_for_test(Principal::reference().name())
                            .unwrap();
                        let account_kind = (*reader.layout_for_test())
                            .entity_kind_for_test(Account::reference().name())
                            .unwrap();
                        let before = (*reader.work_for_test()).endpoint_records_read();
                        reader
                            .endpoint_for_test::<Principal>(
                                *reader.execution_for_test(),
                                principal.entity_id(),
                                principal_kind,
                                "Principal",
                            )
                            .unwrap();
                        let first = (*reader.work_for_test()).endpoint_records_read();
                        let rejected = reader
                            .endpoint_for_test::<Principal>(
                                *reader.execution_for_test(),
                                principal.entity_id(),
                                account_kind,
                                "Principal",
                            )
                            .unwrap_err();
                        let second = (*reader.work_for_test()).endpoint_records_read();
                        (first - before, second - first, rejected.kind())
                    },
                    worth_execution::ExecutionAllocationPolicy::SystemAllocation,
                )
                .unwrap()
        })
        .unwrap();
    let &(accepted, rejected, kind) = completed.output();
    assert!(accepted > 0);
    assert_eq!(
        accepted, rejected,
        "both native callbacks acquired the same metadata"
    );
    assert_eq!(
        kind,
        WorthQueryInvariantProjectionTraversalDenialKind::EndpointUnavailable
    );
}

#[test]
fn cancellation_before_endpoint_acquisition_records_no_endpoint_read() {
    let world = installed_authorization_world(
        true,
        worth_query_admission::facade::authenticated_principal::WorthQueryAdmissionClock::system(),
    );
    let cancellation = CancellationSource::new();
    let host =
        SerialRequest::from_memory(SerialMemoryBudget::new(None), cancellation.token(), None);
    let execution = ExecutionRequest::serial(&host);
    let selected = world
        .application
        .select_product_branch_for_test(
            &execution,
            world
                .application
                .product_runtime()
                .default_branch_for_test(),
        )
        .unwrap();
    let product = selected.product().publication_binding_for_test();
    let observed = Cell::new(None);
    let denied = world.invariant.project_with_work_limit_for_test(
        &execution,
        None,
        product
            .observation_for_test()
            .basis()
            .relational_basis()
            .clone(),
        |reader| {
            let principal = reader
                .resolve_entity(PrincipalIdentityField::reference(), 1_u64)
                .unwrap();
            let kind = (*reader.layout_for_test())
                .entity_kind_for_test(Principal::reference().name())
                .unwrap();
            let before = (*reader.work_for_test()).endpoint_records_read();
            cancellation.cancel();
            let error = reader
                .endpoint_for_test::<Principal>(
                    *reader.execution_for_test(),
                    principal.entity_id(),
                    kind,
                    "Principal",
                )
                .unwrap_err();
            observed.set(Some((
                before,
                (*reader.work_for_test()).endpoint_records_read(),
                error.kind(),
            )));
        },
        None,
        worth_execution::ExecutionAllocationPolicy::SystemAllocation,
    );
    assert!(
        denied.is_err(),
        "the original cancelled request cannot seal output"
    );
    let (before, after, kind) = observed
        .get()
        .expect("the projection reached its controlled stop");
    assert_eq!(before, after);
    assert!(matches!(
        kind,
        WorthQueryInvariantProjectionTraversalDenialKind::ReadExecution(_)
    ));
}
