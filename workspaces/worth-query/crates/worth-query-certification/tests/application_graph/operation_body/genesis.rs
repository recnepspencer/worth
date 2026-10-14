//! Business entities start at the ordinary entry, never in bootstrap.
#[path = "genesis/binding.rs"]
pub(super) mod binding;
#[path = "genesis/declaration.rs"]
pub(super) mod declaration;
#[path = "genesis/portable.rs"]
mod portable;
use super::*;
use binding::{CreateInput, CreateIntent};

type Application =
    WorthQueryProgramApplicationRuntime<DocumentRetentionSchema<30>, program::BodyProgram<30>>;
fn input(tenant: u64) -> CreateInput {
    CreateInput {
        tenant,
        identity: DOCUMENT_IDENTITY.to_owned(),
        retention_days: 7,
    }
}
fn create(
    app: &Application,
    key: u64,
    input: CreateInput,
) -> Result<
    Outcome<WorthQueryOperationBodyDenial, ()>,
    worth_query_host::facade::application_entry::WorthQueryApplicationRequestMutationDenial,
> {
    let scope = request_scope();
    let principal = authenticate_operator(app.runtime().installed_schema(), &scope);
    app.request(&principal, &scope)
        .mutate(CreateIntent(input))
        .without_source()
        .idempotency(&key)
        .execute_in_program(app, ExecutionAllocationPolicy::SystemAllocation)
}

#[test]
fn ordinary_genesis_replay_and_edit_need_no_scope_write_or_business_bootstrap() {
    let app = install::<30>();
    let schema = program::schema::<30>();
    use worth_query_host::facade::declaration::application_schema::ApplicationSchemaMember;
    let body = schema
        .erased()
        .members()
        .iter()
        .find_map(|member| match member {
            ApplicationSchemaMember::OperationBody { operation, body }
                if operation == "CreateDocument" =>
            {
                Some(body)
            }
            _ => None,
        })
        .unwrap();
    assert_eq!(body.scope_entity(), Some("Tenant"));
    assert!(body.scope_fields().is_empty());
    assert!(body.is_creation_only());
    assert!(matches!(
        create(&app, 301, input(1)).unwrap(),
        Outcome::Committed { .. }
    ));
    assert_eq!(read_retention(&app), 7);
    assert!(matches!(
        create(&app, 301, input(1)).unwrap(),
        Outcome::AlreadyCommitted { .. }
    ));
    // The ordinary identity-scoped query rejects ambiguity, so replay must leave
    // exactly one selectable entity, not another equal-looking row.
    assert_eq!(read_retention(&app), 7);
    assert!(matches!(invoke(&app, 302), Outcome::Committed { .. }));
    assert_eq!(read_retention(&app), 30);
}

#[test]
fn missing_tenant_is_a_typed_refusal_before_creation() {
    let app = install::<30>();
    let denied = create(&app, 303, input(999)).unwrap_err();
    assert!(matches!(denied,
        worth_query_host::facade::application_entry::WorthQueryApplicationRequestMutationDenial::ScopeResolution(_)
    ));
    // A lawful retry under the same logical key must create the requested entity,
    // not recover an accidental effect from the refused attempt.
    assert!(matches!(
        create(&app, 303, input(1)).unwrap(),
        Outcome::Committed { .. }
    ));
    assert_eq!(read_retention(&app), 7);
}
