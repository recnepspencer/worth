//! Native literal-body execution. Static fixture resources are not NCR create acceptance.
#[path = "operation_body/advanced_commit.rs"]
mod advanced_commit;
#[path = "operation_body/audit_query.rs"]
mod audit_query;
#[path = "operation_body/binding.rs"]
mod binding;
#[path = "operation_body/body_only.rs"]
mod body_only;
#[path = "operation_body/conditional_requirement.rs"]
mod conditional_requirement;
#[path = "operation_body/creation.rs"]
mod creation;
#[path = "operation_body/draft.rs"]
mod draft;
#[path = "operation_body/draft_query.rs"]
mod draft_query;
#[path = "operation_body/genesis.rs"]
mod genesis;
#[path = "operation_body/input.rs"]
mod input;
#[path = "operation_body/input_selection.rs"]
mod input_selection;
#[path = "operation_body/persistence.rs"]
mod persistence;
#[path = "operation_body/program.rs"]
mod program;
#[path = "operation_body/query.rs"]
mod query;
#[path = "operation_body/replacement.rs"]
mod replacement;
#[path = "operation_body/requirement.rs"]
mod requirement;
#[path = "operation_body/requirement_control.rs"]
mod requirement_control;
#[path = "operation_body/schema.rs"]
mod schema;

use crate::document_retention_model::{
    host::host_limits,
    operator_identity::{block_on, operator_identity, request_scope, OperatorAdapter},
    retention_entry::DOCUMENT_IDENTITY,
};
use schema::*;
use worth_query_host::facade::{
    application_entry::{
        WorthQueryApplicationMutationOutcome as Outcome, WorthQueryApplicationRequestExt,
    },
    application_installation::{in_memory_program, WorthQueryProgramApplicationRuntime},
    declaration::authentication::WorthQueryPrincipalMappingStatus,
    primary_graph::{
        WorthQueryApplicationEntityKey, WorthQueryApplicationEntitySeed,
        WorthQueryApplicationPrincipalKey, WorthQueryApplicationRelationSeed,
        WorthQueryOperationBodyDenial,
    },
    runtime::ExecutionAllocationPolicy,
};

fn install<const DESTINATION: u64>() -> WorthQueryProgramApplicationRuntime<
    DocumentRetentionSchema<DESTINATION>,
    program::BodyProgram<DESTINATION>,
> {
    install_with_projection::<DESTINATION>().0
}

fn install_with_projection<const DESTINATION: u64>() -> (
    WorthQueryProgramApplicationRuntime<
        DocumentRetentionSchema<DESTINATION>,
        program::BodyProgram<DESTINATION>,
    >,
    worth_query_host::facade::primary_graph::WorthQueryApplicationInvariantProjectionAuthority<
        DocumentRetentionSchema<DESTINATION>,
    >,
) {
    let schema = program::schema::<DESTINATION>();
    let mut projection = None;
    let app = in_memory_program(
        program::validated::<DESTINATION>(&schema),
        schema,
        (program::Installation::Native,),
        host_limits(),
        |_phase, graph, installed| {
            projection = Some(graph.retain_invariant_projection_authority());
            graph.bind_principal(
                &installed
                    .principal_binding(DocumentPrincipalBinding::reference())
                    .unwrap(),
                WorthQueryApplicationPrincipalKey::new("body-operator").unwrap(),
                1,
                operator_identity(),
                WorthQueryPrincipalMappingStatus::Enabled,
            )?;
            if DESTINATION == 30 {
                graph.bind_entity(
                    WorthQueryApplicationEntitySeed::new(
                        genesis::declaration::Tenant::reference(),
                        WorthQueryApplicationEntityKey::new("static-tenant").unwrap(),
                    )
                    .field(genesis::declaration::TenantIdentity::reference(), 1),
                )?;
                return Ok(());
            }
            let document = WorthQueryApplicationEntitySeed::new(
                Document::reference(),
                WorthQueryApplicationEntityKey::new("body-document").unwrap(),
            )
            .field(
                DocumentIdentityField::reference(),
                DOCUMENT_IDENTITY.to_owned(),
            )
            .field(DocumentRetentionField::reference(), 7)
            .field(DocumentAuditSourceField::reference(), 7);
            let document = if matches!(DESTINATION, 25 | 26 | 29) {
                document.field(DocumentDraftNoteField::reference(), "active".to_owned())
            } else if DESTINATION == 28 {
                document.field(DocumentDraftNoteField::reference(), String::new())
            } else {
                document
            };
            graph.bind_entity(document)?;
            if matches!(DESTINATION, 25..=27 | 29) {
                graph.bind_entity(
                    WorthQueryApplicationEntitySeed::new(
                        Audit::reference(),
                        WorthQueryApplicationEntityKey::new("body-member").unwrap(),
                    )
                    .field(AuditIdentityField::reference(), "member".to_owned())
                    .field(AuditRetentionField::reference(), 7)
                    .field(AuditPriorRetentionField::reference(), 7),
                )?;
                graph.bind_relation(WorthQueryApplicationRelationSeed::new(
                    DocumentMembers::reference(),
                    "body-member-edge",
                    WorthQueryApplicationEntityKey::new("body-document").unwrap(),
                    WorthQueryApplicationEntityKey::new("body-member").unwrap(),
                ))?;
            }
            Ok(())
        },
    )
    .unwrap();
    (app, projection.unwrap())
}

fn invoke<const DESTINATION: u64>(
    app: &WorthQueryProgramApplicationRuntime<
        DocumentRetentionSchema<DESTINATION>,
        program::BodyProgram<DESTINATION>,
    >,
    key: u64,
) -> Outcome<WorthQueryOperationBodyDenial, ()> {
    invoke_input(
        app,
        key,
        SetRetentionInput {
            identity: DOCUMENT_IDENTITY.to_owned(),
            retention_days: 99,
            ..Default::default()
        },
    )
}

fn invoke_input<const DESTINATION: u64>(
    app: &WorthQueryProgramApplicationRuntime<
        DocumentRetentionSchema<DESTINATION>,
        program::BodyProgram<DESTINATION>,
    >,
    key: u64,
    input: SetRetentionInput,
) -> Outcome<WorthQueryOperationBodyDenial, ()> {
    let scope = request_scope();
    let principal = authenticate_operator(app.runtime().installed_schema(), &scope);
    app.request(&principal, &scope)
        .mutate(binding::BodyIntent(input))
        .without_source()
        .idempotency(&key)
        .execute_in_program(app, ExecutionAllocationPolicy::SystemAllocation)
        .unwrap()
}

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

fn authenticate_operator<Schema: BodySchemaBinding>(
    schema: &worth_query_host::facade::domain::WorthQueryInstalledApplicationSchema<Schema>,
    scope: &worth_query_host::facade::admission::authenticated_principal::WorthQueryRequestScope,
) -> worth_query_host::facade::admission::authenticated_principal::WorthQueryAuthenticatedExternalPrincipal<Schema>{
    use worth_query_host::facade::admission::authenticated_principal::*;
    let adapter = admit_authentication_adapter(
        schema,
        WorthQueryAuthenticationAdapterAdmission::new(
            WorthQueryAuthenticationAudience::new("document-retention-host").unwrap(),
            WorthQueryAuthenticationMethod::new("certification").unwrap(),
        ),
        OperatorAdapter,
    )
    .unwrap();
    block_on(adapter.authenticate((), scope)).unwrap()
}

fn read_retention<const DESTINATION: u64>(
    app: &WorthQueryProgramApplicationRuntime<
        DocumentRetentionSchema<DESTINATION>,
        program::BodyProgram<DESTINATION>,
    >,
) -> u64 {
    let scope = request_scope();
    let principal = authenticate_operator(app.runtime().installed_schema(), &scope);
    app.request(&principal, &scope)
        .query(query::DocumentRetentionRead {
            identity: DOCUMENT_IDENTITY.to_owned(),
        })
        .execute()
        .unwrap()
        .rows()[0]
        .retention_days
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
