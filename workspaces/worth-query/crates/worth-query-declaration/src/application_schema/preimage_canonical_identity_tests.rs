use crate::application_aftermath::{
    DeclaredAftermathPostcondition, DeclaredApplicationAftermathContract,
    DeclaredCorrectionMechanism, DeclaredLoweringCorrespondenceRef, DeclaredPreImageDemand,
    DeclaredPreImageLocus, DeclaredRecordedInverse,
};

use super::{
    ApplicationFieldRef, ApplicationOperationMarkerIdentity, ApplicationOperationRef,
    ApplicationSchema, ApplicationSchemaDeclaration, ApplicationSchemaDeclarationBuilder,
    ApplicationSchemaDeclarationDenial, ApplicationSchemaIdentity,
};

struct Schema;
struct Operation;
struct Entity;
struct Aspect;
struct Field;
impl crate::application_schema::DeclaredApplicationFieldValue for Field {
    type Value = u64;
    type Binding = crate::application_schema::U64ApplicationValueBinding;
    const PRESENCE: crate::application_schema::ApplicationFieldPresence =
        crate::application_schema::ApplicationFieldPresence::Required;
}
crate::worth_query_structured_value_binding!(
    OperationInputBinding for () {
        identity: "worth.rust.unit"
    }
);

impl ApplicationOperationMarkerIdentity<Schema> for Operation {
    type InputBinding = OperationInputBinding;
    const IDENTIFIER: &'static str = "Operation";
}

impl ApplicationSchema for Schema {
    const OWNER: &'static str = "owner";
    const NAME: &'static str = "Schema";
    const MAJOR: u32 = 1;
    const MINOR: u32 = 0;

    fn declaration(
    ) -> Result<ApplicationSchemaDeclaration<Self>, ApplicationSchemaDeclarationDenial> {
        ApplicationSchemaDeclarationBuilder::for_schema().build()
    }
}

#[test]
fn every_exact_preimage_axis_and_bound_changes_declared_schema_identity() {
    let baseline = identity("Entity", "Aspect", "Field", 64);
    for changed in [
        identity("OtherEntity", "Aspect", "Field", 64),
        identity("Entity", "OtherAspect", "Field", 64),
        identity("Entity", "Aspect", "OtherField", 64),
        identity("Entity", "Aspect", "Field", 65),
    ] {
        assert_ne!(baseline, changed);
    }
}

fn identity(
    entity: &'static str,
    aspect: &'static str,
    field: &'static str,
    maximum_encoded_bytes: usize,
) -> ApplicationSchemaIdentity {
    let field = ApplicationFieldRef::<Schema, Entity, Aspect, Field, u64>::from_schema_identifiers(
        entity, aspect, field,
    );
    let inverse = DeclaredRecordedInverse::new(
        "restore",
        DeclaredLoweringCorrespondenceRef::new("exact-inverse").unwrap(),
        DeclaredAftermathPostcondition::ExactPriorTruth,
        DeclaredPreImageDemand::new(
            [DeclaredPreImageLocus::from_field(field)],
            maximum_encoded_bytes,
        )
        .unwrap(),
    )
    .unwrap();
    let definition = ApplicationOperationRef::<Schema, Operation, ()>::from_declaration()
        .definition()
        .no_external_effect()
        .aftermath(DeclaredApplicationAftermathContract::runtime_alone(
            DeclaredCorrectionMechanism::RecordedInverse(inverse),
        ))
        .finish();
    ApplicationSchemaDeclarationBuilder::<Schema>::for_schema()
        .operation(definition)
        .build()
        .unwrap()
        .identity()
        .clone()
}
