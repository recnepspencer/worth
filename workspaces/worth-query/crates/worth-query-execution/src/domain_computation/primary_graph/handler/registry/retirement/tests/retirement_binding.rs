//! Declaration-only type parameters for private original-handler slot mechanics.
//! No runtime, installation, session, principal admission or executable authority is made.
use worth_query_declaration::facade::application_schema::fragment::{self, *};
use worth_query_declaration::facade::{
    application_operation::*, application_schema::*, authentication::*,
};
use worth_query_declaration::{
    worth_query_mutation_binding, worth_query_operation, worth_query_principal_binding,
    worth_query_structured_value_binding,
};
mod records;
use records::*;

worth_query_principal_binding!(pub(super) PrincipalBinding in Schema, mapping Mapping {
    identity: External, status: Status, target: Target => Principal, principal_identity: Identity
});
worth_query_structured_value_binding!(pub(super) UnitBinding for () { identity: "worth.query.handler-retirement.unit.v1" });
worth_query_operation!(pub(super) Operation for Schema, input UnitBinding);
fn scope(_: &()) -> u64 {
    1
}
worth_query_mutation_binding!(pub(super) Binding for (), schema Schema,
    identity "worth.query.handler-retirement.binding.v1",
    input UnitBinding, operation Operation, result UnitBinding,
    idempotency u64, identity "worth.query.handler-retirement.key.v1",
    decision (), denial UnitBinding,
    handler identity "worth.query.handler-retirement.handler.v1",
    outputs NoApplicationMutationOutputs,
    principal PrincipalBinding, mapping Mapping, principal_entity Principal,
        principal_identity u64, identity_binding U64ApplicationValueBinding,
    scope Identity,
    field Identity::at("Identity"), value scope,
);
pub(super) struct Schema;
impl Fragment for Schema {
    const NAME: &'static str = "Schema";
    type Bindings = NoBindings;
    type Members = (IdentityRecords, PrincipalBinding, Operation, Binding);
}
impl ApplicationSchema for Schema {
    const OWNER: &'static str = "worth.query.handler-retirement";
    const NAME: &'static str = <Self as Fragment>::NAME;
    fn declaration(
    ) -> Result<ApplicationSchemaDeclaration<Self>, ApplicationSchemaDeclarationDenial> {
        fragment::declare(Self)
    }
}
impl Registration<Schema> for PrincipalBinding {
    fn register(
        builder: ApplicationSchemaDeclarationBuilder<Schema>,
    ) -> Result<ApplicationSchemaDeclarationBuilder<Schema>, ApplicationSchemaDeclarationDenial>
    {
        Ok(builder.principal_binding(Self::reference()))
    }
}
impl Registration<Schema> for Operation {
    fn register(
        builder: ApplicationSchemaDeclarationBuilder<Schema>,
    ) -> Result<ApplicationSchemaDeclarationBuilder<Schema>, ApplicationSchemaDeclarationDenial>
    {
        Ok(builder.operation(
            Self::reference()
                .definition()
                .no_external_effect()
                .no_aftermath()
                .finish(),
        ))
    }
}
impl Registration<Schema> for Binding {
    fn register(
        builder: ApplicationSchemaDeclarationBuilder<Schema>,
    ) -> Result<ApplicationSchemaDeclarationBuilder<Schema>, ApplicationSchemaDeclarationDenial>
    {
        Ok(builder.application_mutation_binding::<Self>())
    }
}
