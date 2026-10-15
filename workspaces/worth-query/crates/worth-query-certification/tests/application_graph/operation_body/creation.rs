//! Body-authored state and created audit effects share one native candidate.
use super::*;
use worth_query_host::facade::application_entry::WorthQueryApplicationProgramMutationPreparation as Preparation;
use worth_query_host::facade::primary_graph::WorthQueryApplicationUncommitted;

fn read_audit<const DESTINATION: u64>(
    app: &WorthQueryProgramApplicationRuntime<
        DocumentRetentionSchema<DESTINATION>,
        program::BodyProgram<DESTINATION>,
    >,
) -> (u64, u64) {
    let scope = request_scope();
    let principal = authenticate_operator(app.runtime().installed_schema(), &scope);
    let result = app
        .request(&principal, &scope)
        .query(audit_query::AuditRead {
            identity: DOCUMENT_IDENTITY.to_owned(),
        })
        .execute()
        .unwrap();
    let row = &result.rows()[0];
    assert_eq!(row.identity, DOCUMENT_IDENTITY);
    (row.retention_days, row.prior_retention_days)
}

fn move_audit_source<const DESTINATION: u64>(
    app: &WorthQueryProgramApplicationRuntime<
        DocumentRetentionSchema<DESTINATION>,
        program::BodyProgram<DESTINATION>,
    >,
    key: u64,
) {
    let scope = request_scope();
    let principal = authenticate_operator(app.runtime().installed_schema(), &scope);
    let outcome = app
        .request(&principal, &scope)
        .mutate(binding::SourceIntent(SetRetentionInput {
            identity: DOCUMENT_IDENTITY.to_owned(),
            retention_days: 99,
            ..Default::default()
        }))
        .without_source()
        .idempotency(&key)
        .execute_in_program(app, ExecutionAllocationPolicy::SystemAllocation)
        .unwrap();
    assert!(matches!(outcome, Outcome::Committed { .. }));
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
    assert_eq!((read_retention(&first), read_audit(&first)), (20, (20, 7)));
    assert_eq!(
        (read_retention(&second), read_audit(&second)),
        (21, (21, 7))
    );
    assert!(matches!(invoke(&first, 200), Outcome::AlreadyCommitted(_)));
    assert!(matches!(
        invoke(&first, 201),
        Outcome::DomainDenied(WorthQueryOperationBodyDenial::SourceMismatch { .. })
    ));
    assert_eq!((read_retention(&first), read_audit(&first)), (20, (20, 7)));
}

#[test]
fn stale_prepared_creation_cannot_publish_an_extra_audit() {
    let app = install::<20>();
    // This body includes DocumentMembers -> the created audit. A stale native
    // publication must discard the row, its fields and its relation together.
    assert!(program::schema::<20>().erased().members().iter().any(|member| matches!(member,
        worth_query_host::facade::declaration::application_schema::ApplicationSchemaMember::OperationBody { body, .. }
            if body.creation().is_some_and(|creation| creation.links().len() == 1))));
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
    assert_eq!((read_retention(&app), read_audit(&app)), (20, (20, 7)));
}

#[test]
fn only_the_created_field_source_dependency_stales_the_prepared_candidate() {
    let with_source = install::<23>();
    let scope = request_scope();
    let principal = authenticate_operator(with_source.runtime().installed_schema(), &scope);
    let key = 204_u64;
    let mut request = with_source
        .request(&principal, &scope)
        .mutate(binding::BodyIntent(SetRetentionInput {
            identity: DOCUMENT_IDENTITY.to_owned(),
            retention_days: 99,
            ..Default::default()
        }))
        .without_source()
        .idempotency(&key);
    let candidate = match request
        .prepare_in_program(&with_source, ExecutionAllocationPolicy::SystemAllocation)
        .unwrap()
    {
        Preparation::Prepared(candidate) => candidate,
        Preparation::Settled(_) => panic!("fresh source-backed body must prepare"),
    };
    move_audit_source(&with_source, 205);
    assert_eq!(read_retention(&with_source), 7);
    match candidate.commit(ExecutionAllocationPolicy::SystemAllocation) {
        Outcome::Commit(WorthQueryApplicationUncommitted::Stale(stale)) => {
            assert!(stale.stale_fact_count() > 0)
        }
        other => panic!("changed initializer source must stale: {other:?}"),
    }
    assert!(matches!(
        invoke(&with_source, key),
        Outcome::Committed { .. }
    ));
    assert_eq!(
        (read_retention(&with_source), read_audit(&with_source)),
        (23, (23, 8))
    );

    let without_source = install::<22>();
    let principal = authenticate_operator(without_source.runtime().installed_schema(), &scope);
    let key = 206_u64;
    let mut request = without_source
        .request(&principal, &scope)
        .mutate(binding::BodyIntent(SetRetentionInput {
            identity: DOCUMENT_IDENTITY.to_owned(),
            retention_days: 99,
            ..Default::default()
        }))
        .without_source()
        .idempotency(&key);
    let candidate = match request
        .prepare_in_program(&without_source, ExecutionAllocationPolicy::SystemAllocation)
        .unwrap()
    {
        Preparation::Prepared(candidate) => candidate,
        Preparation::Settled(_) => panic!("unrelated body must prepare"),
    };
    move_audit_source(&without_source, 207);
    assert!(matches!(
        candidate.commit(ExecutionAllocationPolicy::SystemAllocation),
        Outcome::Committed { .. }
    ));
    assert_eq!(read_retention(&without_source), 22);
}
