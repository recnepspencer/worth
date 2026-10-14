//! Body-authored state and created audit effects share one native candidate.
use super::*;
use worth_query_host::facade::application_entry::WorthQueryApplicationProgramMutationPreparation as Preparation;
use worth_query_host::facade::primary_graph::WorthQueryApplicationUncommitted;

fn read_audit<const DESTINATION: u64>(
    app: &WorthQueryProgramApplicationRuntime<
        DocumentRetentionSchema<DESTINATION>,
        program::BodyProgram<DESTINATION>,
    >,
) -> u64 {
    let scope = request_scope();
    let principal = authenticate_operator(app.runtime().installed_schema(), &scope);
    app.request(&principal, &scope)
        .query(audit_query::AuditRead {
            identity: DOCUMENT_IDENTITY.to_owned(),
        })
        .execute()
        .unwrap()
        .rows()[0]
        .retention_days
}

#[test]
fn authored_creation_publishes_state_and_audit_in_one_candidate() {
    let first_schema = program::schema::<20>();
    let second_schema = program::schema::<21>();
    assert_ne!(
        program::validated::<20>(&first_schema)
            .normalized_manifest()
            .records(),
        program::validated::<21>(&second_schema)
            .normalized_manifest()
            .records(),
    );
    let first = install::<20>();
    let second = install::<21>();
    assert!(matches!(invoke(&first, 200), Outcome::Committed { .. }));
    assert!(matches!(invoke(&second, 210), Outcome::Committed { .. }));
    assert_eq!((read_retention(&first), read_audit(&first)), (20, 20));
    assert_eq!((read_retention(&second), read_audit(&second)), (21, 21));
    assert!(matches!(invoke(&first, 200), Outcome::AlreadyCommitted(_)));
    assert!(matches!(
        invoke(&first, 201),
        Outcome::DomainDenied(WorthQueryOperationBodyDenial::SourceMismatch { .. })
    ));
    assert_eq!((read_retention(&first), read_audit(&first)), (20, 20));
}

#[test]
fn stale_prepared_creation_cannot_publish_an_extra_audit() {
    let app = install::<20>();
    let scope = request_scope();
    let principal = authenticate_operator(app.runtime().installed_schema(), &scope);
    let key = 202_u64;
    let mut request = app
        .request(&principal, &scope)
        .mutate(binding::BodyIntent(SetRetentionInput {
            identity: DOCUMENT_IDENTITY.to_owned(),
            retention_days: 99,
            ..Default::default()
        }))
        .without_source()
        .idempotency(&key);
    let candidate = match request
        .prepare_in_program(&app, ExecutionAllocationPolicy::SystemAllocation)
        .unwrap()
    {
        Preparation::Prepared(candidate) => candidate,
        Preparation::Settled(_) => panic!("fresh body must prepare a candidate"),
    };
    assert!(matches!(invoke(&app, 203), Outcome::Committed { .. }));
    match candidate.commit(ExecutionAllocationPolicy::SystemAllocation) {
        Outcome::Commit(WorthQueryApplicationUncommitted::Stale(stale)) => {
            assert!(stale.stale_fact_count() > 0)
        }
        other => panic!("losing state-and-audit candidate must be stale: {other:?}"),
    }
    assert_eq!((read_retention(&app), read_audit(&app)), (20, 20));
}
