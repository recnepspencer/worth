//! Authored output correspondence uses the actual co-committed entity identities.
pub use super::*;
pub use worth_query_host::facade::declaration::application_operation::*;
#[path = "output/portable.rs"]
pub mod portable;
#[path = "output/related.rs"]
pub mod related;

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
pub fn create(
    app: &WorthQueryProgramApplicationRuntime<
        DocumentRetentionSchema<31>,
        program::BodyProgram<31>,
    >,
    key: u64,
) -> Outcome<WorthQueryOperationBodyDenial, ()> {
    let scope = request_scope();
    let principal = authenticate_operator(app.runtime().installed_schema(), &scope);
    app.request(&principal, &scope)
        .advanced()
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
