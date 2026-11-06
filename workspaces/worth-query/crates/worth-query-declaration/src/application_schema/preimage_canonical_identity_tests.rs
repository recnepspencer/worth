use crate::application_aftermath::{
    DeclaredAftermathPostcondition, DeclaredApplicationAftermathContract,
    DeclaredCorrectionMechanism, DeclaredLoweringCorrespondenceRef, DeclaredPreImageDemand,
    DeclaredPreImageLocus, DeclaredRecordedInverse,
};

use super::{
    ApplicationOperationMarkerIdentity, ApplicationOperationRef, ApplicationSchema,
    ApplicationSchemaDeclaration, ApplicationSchemaDeclarationBuilder,
    ApplicationSchemaDeclarationDenial, ApplicationSchemaIdentity,
};

struct Schema;
struct Operation;
struct Entity;
struct Aspect;
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
    use super::fragment::FieldSelection;
    let locus = match (entity, aspect) {
        ("Entity", "Aspect") => DeclaredPreImageLocus::<Schema>::from_field(FieldSelection::<
            Fixture,
            Entity,
            Aspect,
            u64,
        >::at(field)),
        ("OtherEntity", "Aspect") => {
            DeclaredPreImageLocus::<Schema>::from_field(FieldSelection::<
                Fixture,
                OtherEntity,
                Aspect,
                u64,
            >::at(field))
        }
        ("Entity", "OtherAspect") => {
            DeclaredPreImageLocus::<Schema>::from_field(FieldSelection::<
                Fixture,
                Entity,
                OtherAspect,
                u64,
            >::at(field))
        }
        _ => panic!("unknown fixture locus"),
    };
    let inverse = DeclaredRecordedInverse::new(
        "restore",
        DeclaredLoweringCorrespondenceRef::new("exact-inverse").unwrap(),
        DeclaredAftermathPostcondition::ExactPriorTruth,
        DeclaredPreImageDemand::new([locus], maximum_encoded_bytes).unwrap(),
    )
    .unwrap();
    let definition = ApplicationOperationRef::<Schema, Operation, ()>::from_declaration()
        .definition()
        .no_external_effect()
        .aftermath(DeclaredApplicationAftermathContract::runtime_alone(
            DeclaredCorrectionMechanism::RecordedInverse(inverse),
        ))
        .finish();
    ApplicationSchemaDeclarationBuilder::<Schema>::from_fragment(Fixture)
        .unwrap()
        .operation(definition)
        .build()
        .unwrap()
        .identity()
        .clone()
}

use super::fragment;
struct Fixture;
impl fragment::Fragment for Fixture {
    const NAME: &'static str = "fixture";
    type Bindings = Bindings;
    type Members = (Entity, OtherEntity);
}
struct OtherEntity;
struct OtherAspect;
struct Bindings;
impl fragment::BindingSet for Bindings {
    type Work<'a> = super::ValueDecodeAdmission<'a>;
    fn with_admission<T>(
        work: &mut Self::Work<'_>,
        run: impl FnOnce(&mut super::ValueDecodeAdmission<'_>) -> T,
    ) -> T {
        run(work)
    }
}
impl fragment::Scalar<u64> for Bindings {
    type Binding = super::U64ApplicationValueBinding;
}
impl fragment::Entity for Entity {
    const NAME: &'static str = "Entity";
    type Aspects = (Aspect, OtherAspect);
    type Relations = ();
}
impl fragment::Entity for OtherEntity {
    const NAME: &'static str = "OtherEntity";
    type Aspects = (Aspect,);
    type Relations = ();
}
impl fragment::Aspect for Aspect {
    const NAME: &'static str = "Aspect";
    type Row = Pair;
}
impl fragment::Aspect for OtherAspect {
    const NAME: &'static str = "OtherAspect";
    type Row = Pair;
}
struct Pair(u64, u64);
impl fragment::Record<Pair> for Bindings {
    fn record() -> impl fragment::Shape<Pair, Self> {
        fragment::field("Field", |r: &Pair| r.0)
            .and("OtherField", |r: &Pair| r.1)
            .build(|a, b, _: &mut super::ValueDecodeAdmission<'_>| {
                Ok::<_, std::convert::Infallible>(Pair(a, b))
            })
    }
}
