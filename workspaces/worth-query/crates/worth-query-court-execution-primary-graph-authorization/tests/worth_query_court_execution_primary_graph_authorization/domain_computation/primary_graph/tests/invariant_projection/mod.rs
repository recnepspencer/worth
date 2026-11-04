pub use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::world_installation::installed_authorization_world;
pub use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::schema_types::live_scope;
pub use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::Account;
pub use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::AccountOwner;
pub use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::AccountStatus;
pub use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::Principal;
pub use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::PrincipalIdentityField;
pub use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::TouchAccountOperation;
pub use worth_query_execution::facade::domain_computation::WorthQueryOperationProjectionDenialKind;
pub use worth_query_execution::facade::domain_computation::WorthQueryPrincipalResolutionMode;
pub use std::time::Duration;

#[test]
fn installed_invariant_projection_reads_one_pinned_typed_graph_snapshot() {
    let host_request = worth_query_testkit_execution_primary_graph_application_attempt_2::test_request_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let world = installed_authorization_world(
        true,
        worth_query_admission::facade::authenticated_principal::WorthQueryAdmissionClock::system(),
    );
    let snapshot = world
        .application
        .with_host_advancement(|phase| world.invariant.snapshot(&phase))
        .expect("installed host call opens")
        .expect("invariant snapshot");
    let accounts = snapshot
        .entities(execution, Account::reference())
        .expect("inventory request remains live");
    let principals = snapshot
        .entities(execution, Principal::reference())
        .expect("inventory request remains live");
    let owners = snapshot
        .relations(execution, AccountOwner::reference())
        .expect("inventory request remains live");
    let mut statuses = accounts
        .iter()
        .map(|account| {
            snapshot
                .field(execution, account, AccountStatus::reference())
                .expect("request remains live")
                .expect("installed account status must project")
        })
        .collect::<Vec<String>>();
    statuses.sort();

    assert!(!snapshot.version().is_zero());
    assert_eq!(accounts.len(), 2);
    assert_eq!(principals.len(), 1);
    assert_eq!(owners.len(), 2);
    assert_eq!(statuses, ["open", "unrelated"]);
    assert!(owners
        .iter()
        .all(|owner| principals.contains(owner.from()) && accounts.contains(owner.to())));
}

#[test]
fn independently_installed_graphs_mint_distinct_projection_identities() {
    let host_request = worth_query_testkit_execution_primary_graph_application_attempt_2::test_request_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let first = installed_authorization_world(
        true,
        worth_query_admission::facade::authenticated_principal::WorthQueryAdmissionClock::system(),
    );
    let second = installed_authorization_world(
        true,
        worth_query_admission::facade::authenticated_principal::WorthQueryAdmissionClock::system(),
    );
    let first_snapshot = first
        .application
        .with_host_advancement(|phase| first.invariant.snapshot(&phase))
        .expect("installed host call opens")
        .expect("first snapshot");
    let second_snapshot = second
        .application
        .with_host_advancement(|phase| second.invariant.snapshot(&phase))
        .expect("installed host call opens")
        .expect("second snapshot");
    let first_account = first_snapshot
        .entities(execution, Account::reference())
        .expect("inventory request remains live")
        .remove(0);
    let second_account = second_snapshot
        .entities(execution, Account::reference())
        .expect("inventory request remains live")
        .remove(0);

    assert_ne!(first_account, second_account);
    assert_eq!(
        second_snapshot.field(execution, &first_account, AccountStatus::reference()),
        Ok(None),
        "a foreign projection identity must not be reinterpreted in this graph"
    );
}

#[test]
fn invariant_entity_identity_exposes_the_resolved_instance() {
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
                        let open = reader
                            .resolve_entity(AccountStatus::reference(), "open".to_string())
                            .unwrap();
                        let unrelated = reader
                            .resolve_entity(AccountStatus::reference(), "unrelated".to_string())
                            .unwrap();
                        (open.entity_id(), unrelated.entity_id())
                    },
                    worth_execution::ExecutionAllocationPolicy::SystemAllocation,
                )
                .expect("invariant projection")
        })
        .expect("the fixture read owns its installed host call");

    assert_ne!(completed.output().0, completed.output().1);
}

#[test]
fn locked_projection_uses_indexes_and_directional_adjacency_without_graph_scans() {
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
                        let owned = reader
                            .relations_from(AccountOwner::reference(), &principal)
                            .unwrap();
                        let open = reader
                            .resolve_entity(AccountStatus::reference(), "open".to_string())
                            .unwrap();
                        let owners = reader
                            .relations_to(AccountOwner::reference(), &open)
                            .unwrap();
                        let mut statuses = owned
                            .iter()
                            .map(|relation| {
                                reader
                                    .field(relation.to(), AccountStatus::reference())
                                    .unwrap()
                            })
                            .collect::<Vec<String>>();
                        statuses.sort();
                        (owned.len(), owners.len(), statuses)
                    },
                    worth_execution::ExecutionAllocationPolicy::SystemAllocation,
                )
                .expect("invariant projection")
        })
        .expect("the fixture read owns its installed host call");

    assert_eq!(
        completed.output(),
        &(2, 1, vec!["open".to_string(), "unrelated".to_string()])
    );
    assert_eq!(completed.work().equality_lookups(), 2);
    assert_eq!(completed.work().index_candidates_examined(), 2);
    assert_eq!(completed.work().adjacency_lists_read(), 2);
    assert_eq!(completed.work().adjacency_edges_inspected(), 3);
    assert_eq!(completed.work().endpoint_records_read(), 3);
    assert_eq!(completed.work().field_reads(), 2);
    let (_, snapshot, _) = completed.into_parts();
    assert!(!snapshot.version().is_zero());
}

#[test]
fn optional_locked_resolution_distinguishes_absence_from_ambiguity_and_counts_work() {
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
                        reader
                            .resolve_optional_entity(
                                AccountStatus::reference(),
                                "missing".to_string(),
                            )
                            .unwrap()
                            .is_none()
                    },
                    worth_execution::ExecutionAllocationPolicy::SystemAllocation,
                )
                .expect("optional invariant projection")
        })
        .expect("the fixture read owns its installed host call");

    assert_eq!(completed.output(), &true);
    assert_eq!(completed.work().equality_lookups(), 1);
    assert_eq!(completed.work().index_candidates_examined(), 0);
}

#[test]
fn panicking_projection_releases_its_snapshot_without_poisoning_the_graph() {
    let host_request = worth_query_testkit_execution_primary_graph_application_attempt_2::test_request_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let world = installed_authorization_world(
        true,
        worth_query_admission::facade::authenticated_principal::WorthQueryAdmissionClock::system(),
    );
    let baseline = world.invariant.active_snapshot_count(request_execution);
    let panic = world
        .application
        .with_host_advancement(|phase| {
            std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                let _ = world.invariant.project::<()>(
                    &phase,
                    |_| panic!("hostile projector"),
                    worth_execution::ExecutionAllocationPolicy::SystemAllocation,
                );
            }))
        })
        .expect("the fixture read owns its installed host call");

    assert!(panic.is_err());
    assert_eq!(
        world
            .invariant
            .active_snapshot_count_for_test(request_execution),
        baseline
    );
    let completed = world
        .application
        .with_host_advancement(|phase| {
            world
                .invariant
                .project(
                    &phase,
                    |reader| reader.version(),
                    worth_execution::ExecutionAllocationPolicy::SystemAllocation,
                )
                .expect("projection after hostile projector")
        })
        .expect("the fixture read owns its installed host call");
    assert!(!completed.output().is_zero());
}

#[test]
fn admitted_projection_supplies_its_exact_root_without_an_equality_lookup() {
    let world = installed_authorization_world(
        true,
        worth_query_admission::facade::authenticated_principal::WorthQueryAdmissionClock::system(),
    );
    let request = live_scope();
    let external = world.authenticate("alice", Duration::from_secs(60), &request);
    let principal = world
        .application
        .with_host_advancement(|phase| {
            let host_custody = phase.test_host_request_for_test();
            world
                .application
                .select_product_branch_for_test(
                    &host_custody.borrowed_request_for_test(),
                    world
                        .application
                        .product_runtime()
                        .default_branch_for_test(),
                )
                .expect("the selected product branch remains admitted")
                .resolve_authenticated_principal(
                    &phase,
                    &world.binding,
                    &external,
                    &request,
                    WorthQueryPrincipalResolutionMode::Ordinary,
                )
                .unwrap()
        })
        .expect("the fixture read owns its installed host call");
    let scope = world
        .application
        .with_host_advancement(|phase| {
            let host_custody = phase.test_host_request_for_test();
            world
                .application
                .select_product_branch_for_test(
                    &host_custody.borrowed_request_for_test(),
                    world
                        .application
                        .product_runtime()
                        .default_branch_for_test(),
                )
                .expect("the selected product branch remains admitted")
                .resolve_entity(
                    &phase,
                    AccountStatus::reference(),
                    "open".to_string(),
                    &request,
                    WorthQueryPrincipalResolutionMode::Ordinary,
                )
                .unwrap()
        })
        .expect("the fixture read owns its installed host call");
    let operation = world
        .application
        .installed_schema()
        .installed_operation(TouchAccountOperation::reference())
        .unwrap();
    let admission = world
        .application
        .with_host_advancement(|phase| {
            world
                .selected_product(&(phase).test_host_request_for_test())
                .authorize_operation(
                    &phase,
                    &principal,
                    &scope,
                    &operation,
                    Default::default(),
                    &request,
                )
                .unwrap()
        })
        .expect("the fixture read owns its installed host call");

    let completed = world
        .application
        .with_host_advancement(|phase| {
            world
                .invariant
                .project_admitted_operation(
                    &phase,
                    &admission,
                    |reader, root| reader.field(root, AccountStatus::reference()),
                    worth_execution::ExecutionAllocationPolicy::SystemAllocation,
                )
                .unwrap()
        })
        .expect("the fixture read owns its installed host call");

    assert_eq!(completed.output(), &Some("open".to_string()));
    assert_eq!(completed.work().equality_lookups(), 0);
    assert_eq!(completed.work().index_candidates_examined(), 0);
    assert_eq!(completed.work().field_reads(), 1);
}

#[path = "admitted_projection_budget_exhaustion_mints_no_snapshot_authorit/mod.rs"]
pub mod admitted_projection_budget_exhaustion_mints_no_snapshot_authorit;
#[path = "counted_read_stops/mod.rs"]
pub mod counted_read_stops;
