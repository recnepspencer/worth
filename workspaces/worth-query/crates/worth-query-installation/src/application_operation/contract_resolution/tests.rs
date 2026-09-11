use worth_foundational::facade::{BoundaryProtocolIdentity, BoundaryProtocolVersion};
use worth_query_declaration::facade::application_aftermath::DeclaredApplicationAftermathContract;
use worth_query_declaration::facade::application_schema::{
    ApplicationExternalEffectProtocol, ApplicationOperationMarkerIdentity, ApplicationOperationRef,
    ApplicationSchema, ApplicationSchemaDeclaration, ApplicationSchemaDeclarationBuilder,
    ApplicationSchemaMember, WorthQueryExternalEffectCorrelationFamily,
};
use worth_query_declaration::facade::portable_identity::WorthQueryPortableTypeIdentity;

use super::{
    operation_aftermath, operation_external_effect, WorthQueryOperationContractCardinalityDenial,
};

struct Schema;
struct ResolutionOperation;
worth_query_declaration::worth_query_structured_value_binding!(
    ResolutionInputBinding for () { identity: "worth.rust.unit" }
);

impl ApplicationOperationMarkerIdentity<Schema> for ResolutionOperation {
    type InputBinding = ResolutionInputBinding;
    const IDENTIFIER: &'static str = "ResolutionOperation";
}

impl ApplicationSchema for Schema {
    const OWNER: &'static str = "worth-query-installation-tests";
    const NAME: &'static str = "ContractResolutionFixture";
    const MAJOR: u32 = 1;
    const MINOR: u32 = 0;

    fn declaration() -> Result<
        ApplicationSchemaDeclaration<Self>,
        worth_query_declaration::facade::application_schema::ApplicationSchemaDeclarationDenial,
    > {
        ApplicationSchemaDeclarationBuilder::<Self>::for_schema().build()
    }
}

#[test]
fn external_effect_resolution_rejects_ambiguity_instead_of_selecting_first() {
    let members = [
        external_effect("Operation", "EffectA"),
        external_effect("Operation", "EffectB"),
    ];
    assert_eq!(
        operation_external_effect(&members, "Operation"),
        Err(WorthQueryOperationContractCardinalityDenial::AmbiguousExternalEffect)
    );
}

#[test]
fn aftermath_resolution_rejects_ambiguity_instead_of_selecting_first() {
    let members = [aftermath("Operation"), aftermath("Operation")];
    assert_eq!(
        operation_aftermath(&members, "Operation"),
        Err(WorthQueryOperationContractCardinalityDenial::AmbiguousAftermath)
    );
}

#[test]
fn unrelated_operation_contracts_do_not_create_false_ambiguity() {
    let members = [
        external_effect("Operation", "EffectA"),
        external_effect("OtherOperation", "EffectB"),
        aftermath("Operation"),
        aftermath("OtherOperation"),
    ];
    assert!(operation_external_effect(&members, "Operation").is_ok());
    assert!(matches!(
        operation_aftermath(&members, "Operation"),
        Ok(Some(_))
    ));
}

fn external_effect(operation: &str, effect: &str) -> ApplicationSchemaMember {
    ApplicationSchemaMember::OperationExternalEffect {
        operation: operation.to_owned(),
        effect: effect.to_owned(),
        rust_payload_type: WorthQueryPortableTypeIdentity::declared("Payload"),
        protocol: ApplicationExternalEffectProtocol::new(
            BoundaryProtocolIdentity::new("test.external-payload"),
            BoundaryProtocolVersion::new(1),
        ),
        maximum_payload_bytes: 64,
        correlation_family: WorthQueryExternalEffectCorrelationFamily::new("external-family")
            .unwrap(),
    }
}

fn aftermath(operation: &'static str) -> ApplicationSchemaMember {
    let definition = ApplicationOperationRef::<Schema, ResolutionOperation, ()>::from_declaration()
        .definition()
        .no_external_effect()
        .aftermath(DeclaredApplicationAftermathContract::not_correctable())
        .finish();
    let declaration = ApplicationSchemaDeclarationBuilder::<Schema>::for_schema()
        .operation(definition)
        .build()
        .expect("the matching operation builder associates the aftermath");
    let mut member = declaration
        .erased()
        .members()
        .iter()
        .find(|member| matches!(member, ApplicationSchemaMember::OperationAftermath { .. }))
        .expect("the matching operation emits its portable aftermath member")
        .clone();
    let ApplicationSchemaMember::OperationAftermath {
        operation: installed,
        ..
    } = &mut member
    else {
        unreachable!()
    };
    *installed = operation.to_owned();
    member
}
