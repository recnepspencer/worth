use super::*;
use worth_query_declaration::facade::application_schema::{
    ApplicationEffectMarkerIdentity, ApplicationSchemaAuthoringDenialKind, OperationEmits,
};

struct ImpostorOperation;
struct ImpostorEffect;

impl ApplicationOperationMarkerIdentity<TestSchema> for ImpostorOperation {
    type InputBinding = TestInputBinding;
    const IDENTIFIER: &'static str = "TestOperation";
}

impl ApplicationEffectMarkerIdentity<TestSchema> for ImpostorEffect {
    type PayloadBinding = TestPayloadBinding;
    const IDENTIFIER: &'static str = "TestEffect";
}

impl OperationEmits<TestOperation<TestSchema>> for ImpostorEffect {}
impl OperationRequiresAbility<ImpostorOperation> for TestAbility {}
impl OperationCreates<ImpostorOperation> for FixtureEntity<TestSchema> {}
impl OperationReads<ImpostorOperation> for FixturePrincipalIdentityField<TestSchema> {}
impl OperationExpectsFact<ImpostorOperation> for FixturePrincipalIdentityField<TestSchema> {}

#[test]
fn foreign_builder_cannot_replace_the_owner_declared_operation_marker() {
    let impostor_declaration = test_schema_members_for::<TestSchema, ImpostorOperation>(None)
        .build()
        .unwrap();

    let denial = installed_index()
        .bind_application_schema(impostor_declaration)
        .unwrap_err();
    assert_eq!(
        denial.kind(),
        WorthQueryInstalledApplicationSchemaDenialKind::SchemaMeaningChanged
    );
}

#[test]
fn copied_operation_name_and_input_identity_cannot_resolve_installed_membership() {
    let schema = installed_index()
        .bind_application_schema(TestSchema::declaration().unwrap())
        .unwrap();
    let impostor =
        ApplicationOperationRef::<TestSchema, ImpostorOperation, TestInput>::from_declaration();

    let denial = match schema.installed_operation(impostor) {
        Ok(_) => panic!("copied operation marker must not resolve installed membership"),
        Err(denial) => denial,
    };
    assert_eq!(
        denial.kind(),
        crate::facade::WorthQueryApplicationOperationInstallationDenialKind::OperationMeaningChanged
    );
}

#[test]
fn copied_effect_name_and_payload_identity_cannot_enter_installed_authoring() {
    let schema = installed_index()
        .bind_application_schema(TestSchema::declaration().unwrap())
        .unwrap();
    let operation =
        ApplicationOperationRef::<TestSchema, TestOperation<TestSchema>, TestInput>::
            from_declaration();
    let impostor =
        ApplicationEffectRef::<TestSchema, ImpostorEffect, TestPayload>::from_declaration();

    let denial = schema
        .effects(operation)
        .emit(impostor, TestPayload)
        .build()
        .unwrap_err();
    assert_eq!(
        denial.kind(),
        ApplicationSchemaAuthoringDenialKind::EffectProvenanceMismatch
    );
}
