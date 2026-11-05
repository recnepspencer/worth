//! Declaration-only type parameters for private original-handler slot mechanics.
//! No runtime, installation, session, principal admission or executable authority is made.
use worth_foundational::facade::{AspectContractRevision, AspectIdentity};
use worth_query_declaration::facade::{
    application_operation::*, application_schema::*, authentication::*,
};
use worth_query_declaration::{
    worth_query_application_schema, worth_query_aspect, worth_query_entity, worth_query_field,
    worth_query_mutation_binding, worth_query_operation, worth_query_principal_binding,
    worth_query_relation, worth_query_structured_value_binding,
};

worth_query_entity!(pub(super) Mapping for Schema);
worth_query_entity!(pub(super) Principal for Schema);
worth_query_aspect!(pub(super) MappingFacts for Schema, Mapping; identity = AspectIdentity(0x7719_1011), revision = AspectContractRevision(1));
worth_query_aspect!(pub(super) PrincipalFacts for Schema, Principal; identity = AspectIdentity(0x7719_1012), revision = AspectContractRevision(1));
worth_query_field!(pub(super) External for Schema, Mapping, MappingFacts: WorthQueryExternalPrincipalIdentity => WorthQueryExternalPrincipalIdentityBinding, read_only, equality);
worth_query_field!(pub(super) Status for Schema, Mapping, MappingFacts: WorthQueryPrincipalMappingStatus => WorthQueryPrincipalMappingStatusBinding, read_write, equality);
worth_query_field!(pub(super) Identity for Schema, Principal, PrincipalFacts: u64 => U64ApplicationValueBinding, read_only, equality);
worth_query_relation!(pub(super) Target in Schema, Mapping => Principal; integrity = same_context_unbounded_retain_dangling);
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
    scope Principal, PrincipalFacts, Identity, u64, ReadOnly, NoApplicationUnit,
    field Identity::reference(), value scope,
);
worth_query_application_schema! { pub(super) schema Schema {
    owner: "worth.query.handler-retirement", version: (1, 0),
    members: |schema| {
        schema.entity(Mapping::reference()).entity(Principal::reference())
            .aspect(Mapping::reference(), MappingFacts::reference())
            .aspect(Principal::reference(), PrincipalFacts::reference())
            .field(Mapping::reference(), External::reference())
            .field(Mapping::reference(), Status::reference())
            .field(Principal::reference(), Identity::reference())
            .relation(Target::reference(), Mapping::reference(), Principal::reference())
            .principal_binding(PrincipalBinding::reference())
            .operation(Operation::reference().definition().no_external_effect().no_aftermath().finish())
            .application_mutation_binding::<Binding>()
    }
} }
