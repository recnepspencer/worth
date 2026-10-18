//! A body creates rows and their graph linkage atomically through native owners.
use super::*;
#[path = "creation_link/portable.rs"]
mod portable;
#[path = "creation_link/query.rs"]
pub(super) mod query;

#[test]
fn ordinary_created_to_scope_link_is_traversable_and_exact_replay_keeps_it() {
    let app = install::<30>();
    let scope = request_scope();
    let principal = authenticate_operator(app.runtime().installed_schema(), &scope);
    let command = genesis::binding::CreateIntent(genesis::binding::CreateInput {
        tenant: 1,
        identity: DOCUMENT_IDENTITY.to_owned(),
        retention_days: 7,
        note: None,
    });
    let first = app
        .request(&principal, &scope)
        .advanced()
        .mutate(command.clone())
        .without_source()
        .idempotency(&320)
        .execute_in_program(&app, ExecutionAllocationPolicy::SystemAllocation)
        .unwrap();
    let Outcome::Committed { receipt, .. } = first else {
        panic!("linked genesis")
    };
    assert_eq!(
        app.request(&principal, &scope)
            .query(query::ReadTenant(DOCUMENT_IDENTITY.to_owned()))
            .execute()
            .unwrap()
            .rows(),
        &[query::TenantRow(1)]
    );
    let replay = app
        .request(&principal, &scope)
        .advanced()
        .mutate(command)
        .without_source()
        .idempotency(&320)
        .execute_in_program(&app, ExecutionAllocationPolicy::SystemAllocation)
        .unwrap();
    let Outcome::AlreadyCommitted(replayed) = replay else {
        panic!("replay")
    };
    assert_eq!(replayed.outcome_identity(), receipt.outcome_identity());
    assert_eq!(
        app.request(&principal, &scope)
            .query(query::ReadTenant(DOCUMENT_IDENTITY.to_owned()))
            .execute()
            .unwrap()
            .rows(),
        &[query::TenantRow(1)]
    );
}
