use super::*;
use worth_query_declaration::facade::application_query::ApplicationQueryBinding;

pub struct ResultOnlyInput;
worth_query_structured_value_binding!(pub ResultOnlyInputBinding for ResultOnlyInput {
    identity: "worth.query.tests.result-only-input.v1"
});
worth_query_query_binding!(
    pub ResultOnlyQueryBinding for ResultOnlyInput, schema IdentitySchema,
    identity "worth.query.tests.result-only-binding.v1",
    input ResultOnlyInputBinding,
    query PrincipalQuery,
    parameters PrincipalQueryParametersBinding => |_| {
        worth_query_declaration::facade::application_query::ApplicationQueryParameterSet::new()
    },
    result PrincipalQueryResultBinding,
    principal IdentityBinding, mapping ExternalMapping, principal_entity Principal,
        principal_identity u64, identity_binding U64ApplicationValueBinding,
    scope Principal, PrincipalIdentity, PrincipalIdentityField, u64,
        ReadOnly, worth_query_declaration::facade::application_schema::NoApplicationUnit,
    principal_field PrincipalIdentityField::reference(),
    limits results 8
);

#[test]
fn result_only_macro_installs_without_a_mechanical_work_number() {
    let schema = installed_index()
        .bind_application_schema(IdentitySchema::declaration().unwrap())
        .unwrap();
    let ordinary = schema
        .installed_query_binding::<ResultOnlyQueryBinding>()
        .unwrap();
    let explicit = schema
        .installed_query_binding::<PrincipalQueryBinding>()
        .unwrap();
    assert_eq!(ResultOnlyQueryBinding::LIMITS.maximum_work(), None);
    assert_eq!(ordinary.limits().maximum_work(), None);
    assert_eq!(ordinary.limits().maximum_results().get(), 8);
    assert_eq!(
        ordinary.query().identity(),
        explicit.query().identity(),
        "operational binding caps do not redefine the query's semantic identity"
    );
}
