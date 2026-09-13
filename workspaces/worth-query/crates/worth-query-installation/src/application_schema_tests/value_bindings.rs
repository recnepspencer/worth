use std::any::TypeId;

use worth_foundational::facade::{AspectValue, ScalarAspectType};
use worth_query_declaration::facade::application_schema::{
    ApplicationSchema, ApplicationValueEncodeDenial, U64ApplicationValueBinding,
};

use super::{installed_index, TestSchema};

#[test]
fn installed_schema_exposes_exact_indexed_field_binding_recipes() {
    let installed = installed_index()
        .bind_application_schema(TestSchema::declaration().unwrap())
        .unwrap();
    let bindings = installed.value_bindings();

    let principal_identity = bindings
        .field("TestEntity", "IdentityAspect", "PrincipalIdentityField")
        .expect("the declared principal identity field must have one installed binding");
    assert_eq!(bindings.len(), 3);
    assert_eq!(principal_identity.schema(), &installed.binding_identity());
    assert_eq!(principal_identity.identity().as_str(), "worth.rust.u64");
    assert_eq!(principal_identity.scalar_family(), ScalarAspectType::UInt64);
    assert_eq!(principal_identity.value_type(), TypeId::of::<u64>());
    assert_eq!(
        principal_identity.binding_type(),
        TypeId::of::<U64ApplicationValueBinding>()
    );
    assert!(principal_identity.is_readable());
    assert!(principal_identity.is_identity());
    assert!(!principal_identity.is_signed_aggregate());

    let encoded = principal_identity.encode(&42_u64).unwrap();
    assert_eq!(encoded, AspectValue::UInt64(42));

    let denial = principal_identity
        .encode(&"wrong type")
        .expect_err("an unrelated Rust value type must fail at the binding boundary");
    assert!(matches!(
        denial,
        ApplicationValueEncodeDenial::Validation(_)
    ));
}
