use super::fixture::{
    installed_authorization_world, live_scope, Account, AccountOwner, AccountStatus, Principal,
    PrincipalIdentityField, TouchAccountOperation,
};
use crate::domain_computation::primary_graph::{
    WorthQueryOperationProjectionDenialKind, WorthQueryPrincipalResolutionMode,
};
use std::time::Duration;

#[test]
fn installed_invariant_projection_reads_one_pinned_typed_graph_snapshot() {
    let world = installed_authorization_world(true);
    let snapshot = world.invariant.snapshot().expect("invariant snapshot");
    let accounts = snapshot.entities(Account::reference());
    let principals = snapshot.entities(Principal::reference());
    let owners = snapshot.relations(AccountOwner::reference());
    let mut statuses = accounts
        .iter()
        .map(|account| {
            snapshot
                .field(account, AccountStatus::reference())
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
    let first = installed_authorization_world(true);
    let second = installed_authorization_world(true);
    let first_snapshot = first.invariant.snapshot().expect("first snapshot");
    let second_snapshot = second.invariant.snapshot().expect("second snapshot");
    let first_account = first_snapshot.entities(Account::reference()).remove(0);
    let second_account = second_snapshot.entities(Account::reference()).remove(0);

    assert_ne!(first_account, second_account);
    assert_eq!(
        second_snapshot.field(&first_account, AccountStatus::reference()),
        None,
        "a foreign projection identity must not be reinterpreted in this graph"
    );
}

#[test]
fn locked_projection_uses_indexes_and_directional_adjacency_without_graph_scans() {
    let world = installed_authorization_world(true);
    let completed = world
        .invariant
        .project(|reader| {
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
        })
        .expect("invariant projection");

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
    assert_eq!(completed.work().reconstructive_scans(), 0);
    let (_, snapshot, _) = completed.into_parts();
    assert!(!snapshot.version().is_zero());
}

#[test]
fn optional_locked_resolution_distinguishes_absence_from_ambiguity_and_counts_work() {
    let world = installed_authorization_world(true);
    let completed = world
        .invariant
        .project(|reader| {
            reader
                .resolve_optional_entity(AccountStatus::reference(), "missing".to_string())
                .unwrap()
                .is_none()
        })
        .expect("optional invariant projection");

    assert_eq!(completed.output(), &true);
    assert_eq!(completed.work().equality_lookups(), 1);
    assert_eq!(completed.work().index_candidates_examined(), 0);
    assert_eq!(completed.work().reconstructive_scans(), 0);
}

#[test]
fn panicking_projection_releases_its_snapshot_without_poisoning_the_graph() {
    let world = installed_authorization_world(true);
    let baseline = world.invariant.active_snapshot_count();
    let panic = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        let _ = world
            .invariant
            .project::<()>(|_| panic!("hostile projector"));
    }));

    assert!(panic.is_err());
    assert_eq!(world.invariant.active_snapshot_count(), baseline);
    let completed = world
        .invariant
        .project(|reader| reader.version())
        .expect("projection after hostile projector");
    assert!(!completed.output().is_zero());
}

#[test]
fn admitted_projection_supplies_its_exact_root_without_an_equality_lookup() {
    let world = installed_authorization_world(true);
    let request = live_scope();
    let external = world.authenticate("alice", Duration::from_secs(60), &request);
    let principal = world
        .application
        .select_product_branch(world.application.product_runtime().default_branch())
        .expect("the selected product branch remains admitted")
        .resolve_authenticated_principal(
            &world.binding,
            external,
            &request,
            WorthQueryPrincipalResolutionMode::Ordinary,
        )
        .unwrap();
    let scope = world
        .application
        .select_product_branch(world.application.product_runtime().default_branch())
        .expect("the selected product branch remains admitted")
        .resolve_entity(
            AccountStatus::reference(),
            "open".to_string(),
            &request,
            WorthQueryPrincipalResolutionMode::Ordinary,
        )
        .unwrap();
    let operation = world
        .application
        .installed_schema()
        .installed_operation(TouchAccountOperation::reference())
        .unwrap();
    let admission = world
        .selected_product()
        .authorize_operation(&principal, &scope, &operation, Default::default(), &request)
        .unwrap();

    let completed = world
        .invariant
        .project_admitted_operation(&admission, |reader, root| {
            reader.field(root, AccountStatus::reference())
        })
        .unwrap();

    assert_eq!(completed.output(), &Some("open".to_string()));
    assert_eq!(completed.work().equality_lookups(), 0);
    assert_eq!(completed.work().index_candidates_examined(), 0);
    assert_eq!(completed.work().field_reads(), 1);
}

#[test]
fn admitted_projection_budget_exhaustion_mints_no_snapshot_authority() {
    let world = installed_authorization_world(true);
    let request = live_scope();
    let external = world.authenticate("alice", Duration::from_secs(60), &request);
    let principal = world
        .application
        .select_product_branch(world.application.product_runtime().default_branch())
        .expect("the selected product branch remains admitted")
        .resolve_authenticated_principal(
            &world.binding,
            external,
            &request,
            WorthQueryPrincipalResolutionMode::Ordinary,
        )
        .unwrap();
    let scope = world
        .application
        .select_product_branch(world.application.product_runtime().default_branch())
        .expect("the selected product branch remains admitted")
        .resolve_entity(
            AccountStatus::reference(),
            "open".to_string(),
            &request,
            WorthQueryPrincipalResolutionMode::Ordinary,
        )
        .unwrap();
    let operation = world
        .application
        .installed_schema()
        .installed_operation(TouchAccountOperation::reference())
        .unwrap();
    assert_eq!(operation.contracts().projection_work_budget(), 32);
    let admission = world
        .selected_product()
        .authorize_operation(&principal, &scope, &operation, Default::default(), &request)
        .unwrap();
    let baseline = world.invariant.active_snapshot_count();

    let denial = world
        .invariant
        .project_admitted_operation(&admission, |reader, _| {
            let account = reader
                .resolve_entity(AccountStatus::reference(), "open".to_string())
                .unwrap();
            for _ in 0..31 {
                let _ = reader.field(&account, AccountStatus::reference());
            }
        })
        .err()
        .expect("provider work beyond the installed limit must deny");

    assert_eq!(
        denial.kind(),
        WorthQueryOperationProjectionDenialKind::WorkBudgetExceeded
    );
    assert_eq!(world.invariant.active_snapshot_count(), baseline);
}
