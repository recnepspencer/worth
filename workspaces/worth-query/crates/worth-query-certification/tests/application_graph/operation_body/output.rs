//! Authored output correspondence uses the actual co-committed entity identities.
use super::*;
use worth_query_host::facade::declaration::application_operation::*;
#[path = "output/portable.rs"]
mod portable;

pub struct ScopeOutputs<const MODE: u64>;
pub struct ScopeOutput<const MODE: u64>;
impl<const MODE: u64> WorthQueryApplicationOutputRole for ScopeOutput<MODE> {
    type Schema = DocumentRetentionSchema<MODE>;
    type Contract = ScopeOutputs<MODE>;
    type Entity = Document;
    type Action = WorthQueryPreserveOutput;
    type Cardinality = WorthQueryAtMostOneOutput;
    const NAME: &'static str = "document";
}
impl<const MODE: u64> ApplicationMutationOutputContract<DocumentRetentionSchema<MODE>>
    for ScopeOutputs<MODE>
{
    const ROLES: &'static [ApplicationMutationOutputRoleDescriptor] =
        &[<ScopeOutput<MODE> as WorthQueryApplicationDeclaredOutputRole>::DESCRIPTOR];
}
pub struct CreatedOutputs<const MODE: u64>;
pub struct CreatedOutput<const MODE: u64>;
pub struct AbsentOutput<const MODE: u64>;
impl<const MODE: u64> WorthQueryApplicationOutputRole for CreatedOutput<MODE> {
    type Schema = DocumentRetentionSchema<MODE>;
    type Contract = CreatedOutputs<MODE>;
    type Entity = Document;
    type Action = WorthQueryCreateOutput;
    type Cardinality = WorthQueryAtMostOneOutput;
    const NAME: &'static str = "created-document";
}
impl<const MODE: u64> WorthQueryApplicationOutputRole for AbsentOutput<MODE> {
    type Schema = DocumentRetentionSchema<MODE>;
    type Contract = CreatedOutputs<MODE>;
    type Entity = Document;
    type Action = WorthQueryPreserveOutput;
    type Cardinality = WorthQueryAtMostOneOutput;
    const NAME: &'static str = "unused";
}
impl<const MODE: u64> ApplicationMutationOutputContract<DocumentRetentionSchema<MODE>>
    for CreatedOutputs<MODE>
{
    const ROLES: &'static [ApplicationMutationOutputRoleDescriptor] = &[
        <CreatedOutput<MODE> as WorthQueryApplicationDeclaredOutputRole>::DESCRIPTOR,
        <AbsentOutput<MODE> as WorthQueryApplicationDeclaredOutputRole>::DESCRIPTOR,
    ];
}
fn create(
    app: &WorthQueryProgramApplicationRuntime<
        DocumentRetentionSchema<31>,
        program::BodyProgram<31>,
    >,
    key: u64,
) -> Outcome<WorthQueryOperationBodyDenial, ()> {
    let scope = request_scope();
    let principal = authenticate_operator(app.runtime().installed_schema(), &scope);
    app.request(&principal, &scope)
        .mutate(genesis::binding::CreateIntent(
            genesis::binding::CreateInput {
                tenant: 1,
                identity: DOCUMENT_IDENTITY.to_owned(),
                retention_days: 7,
                note: None,
            },
        ))
        .without_source()
        .idempotency(&key)
        .execute_in_program(app, ExecutionAllocationPolicy::SystemAllocation)
        .unwrap()
}

#[test]
fn created_and_scope_outputs_name_the_same_real_row_and_replay_keeps_correspondence() {
    let app = install::<31>();
    let Outcome::Committed { receipt, .. } = create(&app, 310) else {
        panic!("ordinary genesis")
    };
    let created = receipt
        .outputs_of::<CreatedOutputs<31>>()
        .unwrap()
        .entity::<CreatedOutput<31>>()
        .unwrap()
        .expect("authored Create output is present")
        .entity_id();
    assert!(receipt.outputs_of::<ScopeOutputs<31>>().is_err());
    assert!(receipt
        .outputs_of::<CreatedOutputs<31>>()
        .unwrap()
        .entity::<AbsentOutput<31>>()
        .unwrap()
        .is_none());
    let Outcome::AlreadyCommitted(replayed) = create(&app, 310) else {
        panic!("replay")
    };
    assert_eq!(
        replayed
            .outputs_of::<CreatedOutputs<31>>()
            .unwrap()
            .entity::<CreatedOutput<31>>()
            .unwrap()
            .expect("replayed Create output is present")
            .entity_id(),
        created
    );
    let Outcome::Committed { receipt, .. } = invoke(&app, 311) else {
        panic!("output-only preserve")
    };
    assert_eq!(
        receipt
            .outputs_of::<ScopeOutputs<31>>()
            .unwrap()
            .entity::<ScopeOutput<31>>()
            .unwrap()
            .expect("authored Preserve output is present")
            .entity_id(),
        created
    );
    assert_eq!(
        read_retention(&app),
        7,
        "preservation must not fabricate a business write"
    );
    let body = program::schema::<31>().erased().members().iter().find_map(|member| match member {
        worth_query_host::facade::declaration::application_schema::ApplicationSchemaMember::OperationBody { operation, body } if operation == "SetRetention" => Some(body.clone()), _ => None,
    }).unwrap();
    assert!(body.scope_fields().is_empty());
    assert!(body.creation().is_none());
    assert_eq!(body.outputs().len(), 1);
}
