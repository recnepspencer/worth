pub(crate) use worth_query_testkit_certification_operation::facade::certification::native_operation_bodies::operation_body::*;

#[test]
fn authored_body_literals_drive_actual_writes_and_program_revisions() {
    use worth_query_host::facade::declaration::application_schema::ApplicationSchemaMember;
    let first_schema = program::schema::<8>();
    let second_schema = program::schema::<9>();
    let without_body = |members: &[ApplicationSchemaMember]| {
        members
            .iter()
            .filter(|member| !matches!(member, ApplicationSchemaMember::OperationBody { .. }))
            .cloned()
            .collect::<Vec<_>>()
    };
    assert_eq!(
        without_body(first_schema.erased().members()),
        without_body(second_schema.erased().members())
    );
    let first_program = program::validated::<8>(&first_schema).normalized_manifest();
    let second_program = program::validated::<9>(&second_schema).normalized_manifest();
    let without_body_record = |records: &[String]| {
        records
            .iter()
            .filter(|record| !record.starts_with("action-body-v1|"))
            .cloned()
            .collect::<Vec<_>>()
    };
    assert_eq!(
        without_body_record(first_program.records()),
        without_body_record(second_program.records())
    );
    let first = install::<8>();
    let second = install::<9>();
    assert_ne!(
        first.installed_program().revision(),
        second.installed_program().revision()
    );
    assert!(matches!(invoke(&first, 80), Outcome::Committed { .. }));
    assert!(matches!(invoke(&second, 90), Outcome::Committed { .. }));
    assert_eq!(read_retention(&first), 8);
    assert_eq!(read_retention(&second), 9);
    // The unchanged caller supplied 99, never either authored destination.
}

#[test]
fn authored_body_replacement_is_denied_before_any_handler_is_installed() {
    use worth_query_host::facade::{
        application_installation::WorthQueryInMemoryApplicationDenial,
        primary_graph::WorthQueryPrimaryGraphInstallationDenialKind,
    };
    let schema = program::schema::<8>();
    let result = in_memory_program(
        program::validated::<8>(&schema),
        schema,
        (program::Installation::Replacement,),
        ApplicationBuildIdentity::parse("11111111-1111-1111-1111-111111111111").unwrap(),
        host_limits(),
        |_phase, _graph, _installed| Ok(()),
    );
    match result {
        Err(WorthQueryInMemoryApplicationDenial::Contributions(denial)) => assert_eq!(
            denial.kind(),
            WorthQueryPrimaryGraphInstallationDenialKind::OperationBodyReplacement
        ),
        Err(other) => panic!("expected native replacement refusal: {other:?}"),
        Ok(_) => panic!("host must not override authored values with a replacement handler"),
    }
}

#[test]
fn authored_body_replay_recovers_original_commit_after_source_guard_changes() {
    let app = install::<8>();
    assert!(matches!(invoke(&app, 85), Outcome::Committed { .. }));
    assert!(matches!(invoke(&app, 85), Outcome::AlreadyCommitted(_)));
    assert_eq!(read_retention(&app), 8);
}

#[test]
fn authored_body_source_guard_denies_without_another_effect() {
    let app = install::<8>();
    assert!(matches!(invoke(&app, 81), Outcome::Committed { .. }));
    assert!(matches!(
        invoke(&app, 82),
        Outcome::DomainDenied(WorthQueryOperationBodyDenial::SourceMismatch { .. })
    ));
    assert_eq!(read_retention(&app), 8);
}

#[test]
fn authored_body_captured_dependency_stales_before_publication() {
    use worth_query_host::facade::application_entry::WorthQueryApplicationProgramMutationPreparation as Preparation;
    use worth_query_host::facade::primary_graph::WorthQueryApplicationUncommitted;
    let app = install::<8>();
    let scope = request_scope();
    let principal = authenticate_operator(app.runtime().installed_schema(), &scope);
    let key = 83_u64;
    let mut request = app
        .request(&principal, &scope)
        .advanced()
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
        Preparation::Settled(_) => {
            panic!("fresh body decision must produce an unpublished candidate")
        }
    };
    assert!(matches!(invoke(&app, 84), Outcome::Committed { .. }));
    match candidate.commit(ExecutionAllocationPolicy::SystemAllocation) {
        Outcome::Commit(WorthQueryApplicationUncommitted::Stale(stale)) => {
            assert!(stale.stale_fact_count() > 0)
        }
        other => panic!("body-read dependency must reject the losing candidate: {other:?}"),
    }
    assert_eq!(read_retention(&app), 8);
}

#[test]
fn authored_body_unbound_program_cannot_acquire_body_by_label() {
    use worth_query_host::facade::application_installation::WorthQueryInMemoryApplicationDenial;
    use worth_query_host::facade::domain::WorthQueryProgramSupportDenial;
    let old_schema = program::schema::<8>();
    use worth_query_host::facade::declaration::application_program::ApplicationProgramAuthoring;
    let old_program =
        ApplicationProgramAuthoring::<DocumentRetentionSchema<8>, program::BodyProgram<8>>::begin()
            .validated_program()
            .unwrap();
    let result = in_memory_program(
        old_program,
        old_schema,
        (program::Installation::Native,),
        ApplicationBuildIdentity::parse("11111111-1111-1111-1111-111111111111").unwrap(),
        host_limits(),
        |_phase, _graph, _installed| panic!("mismatching body must deny before initialization"),
    );
    match result {
        Err(WorthQueryInMemoryApplicationDenial::Program(denial)) => assert!(matches!(
            denial.support_denial(),
            Some(WorthQueryProgramSupportDenial::UnsupportedAction { .. })
        )),
        Err(other) => panic!("expected exact native program support refusal: {other:?}"),
        Ok(_) => panic!("same binding label must not replace the old program body"),
    }
}

#[test]
fn authored_body_missing_executor_is_denied_at_installation() {
    use worth_query_host::facade::{
        application_installation::WorthQueryInMemoryApplicationDenial,
        primary_graph::WorthQueryPrimaryGraphInstallationDenialKind,
    };
    let schema = program::schema::<8>();
    let result = in_memory_program(
        program::validated::<8>(&schema),
        schema,
        (program::Installation::Missing,),
        ApplicationBuildIdentity::parse("11111111-1111-1111-1111-111111111111").unwrap(),
        host_limits(),
        |_phase, _graph, _installed| Ok(()),
    );
    match result {
        Err(WorthQueryInMemoryApplicationDenial::Contributions(denial)) => assert_eq!(
            denial.kind(),
            WorthQueryPrimaryGraphInstallationDenialKind::MissingMutationHandler
        ),
        Err(other) => panic!("expected native missing-body-executor refusal: {other:?}"),
        Ok(_) => panic!("a body declaration cannot substitute for installed execution"),
    }
}

pub(crate) mod advanced_commit;

pub(crate) mod body_only;

pub(crate) mod conditional_requirement;

pub(crate) mod creation;

pub(crate) mod creation_link;

pub(crate) mod draft;

pub(crate) mod genesis;

pub(crate) mod input_expectation;

pub(crate) mod input_selection;

pub(crate) mod optional_input;

pub(crate) mod output;

pub(crate) mod persistence;

pub(crate) mod principal;

pub(crate) mod requirement;

pub(crate) mod retirement;
