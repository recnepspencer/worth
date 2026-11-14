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
    demand_identity(DeclaredPreImageDemand::new([locus], maximum_encoded_bytes))
}

fn demand_identity(demand: DeclaredPreImageDemand<Schema>) -> ApplicationSchemaIdentity {
    let inverse = DeclaredRecordedInverse::new(
        "restore",
        DeclaredLoweringCorrespondenceRef::new("exact-inverse"),
        DeclaredAftermathPostcondition::ExactPriorTruth,
        demand,
    );
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

use crate::application_operation::*;
struct Outputs;
impl ApplicationMutationOutputContract<Schema> for Outputs {
    const ROLES: &'static [ApplicationMutationOutputRoleDescriptor] = &[];
}
struct Target<A, C>(std::marker::PhantomData<(A, C)>);
impl<
        A: WorthQueryApplicationOutputAction + 'static,
        C: WorthQueryApplicationOutputCardinality + 'static,
    > WorthQueryApplicationOutputRole for Target<A, C>
{
    type Schema = Schema;
    type Contract = Outputs;
    type Entity = Entity;
    type Action = A;
    type Cardinality = C;
    const NAME: &'static str = "target";
}
fn field() -> DeclaredPreImageLocus<Schema> {
    DeclaredPreImageLocus::from_field(
        fragment::FieldSelection::<Fixture, Entity, Aspect, u64>::at("Field"),
    )
}

#[test]
fn changed_fields_require_one_preserved_output_and_a_bounded_same_entity_vocabulary() {
    use crate::application_aftermath::DeclaredPreImageDemandDenial as Denial;
    let kept = Target::<WorthQueryPreserveOutput, WorthQueryExactlyOneOutput>::DESCRIPTOR;
    assert_eq!(
        DeclaredPreImageDemand::changed_fields_for_output(kept, [field()], 64).validate(),
        Ok(())
    );
    for role in [
        Target::<WorthQueryCreateOutput, WorthQueryExactlyOneOutput>::DESCRIPTOR,
        Target::<WorthQueryRetireOutput, WorthQueryExactlyOneOutput>::DESCRIPTOR,
        Target::<WorthQueryPreserveOutput, WorthQueryAtMostOneOutput>::DESCRIPTOR,
    ] {
        assert_eq!(
            DeclaredPreImageDemand::changed_fields_for_output(role, [field()], 64).validate(),
            Err(Denial::InvalidOutputFieldCapture)
        );
    }
    let foreign = DeclaredPreImageLocus::<Schema>::from_field(fragment::FieldSelection::<
        Fixture,
        OtherEntity,
        Aspect,
        u64,
    >::at("Field"));
    for fields in [vec![], vec![foreign]] {
        assert_eq!(
            DeclaredPreImageDemand::changed_fields_for_output(kept, fields, 64).validate(),
            Err(Denial::InvalidOutputFieldCapture)
        );
    }
    assert_eq!(
        DeclaredPreImageDemand::changed_fields_for_output(kept, [field()], 0).validate(),
        Err(Denial::ZeroByteBound)
    );
    assert_eq!(
        DeclaredPreImageDemand::changed_fields_for_output(kept, [field(), field()], 64).validate(),
        Err(Denial::DuplicateLocus)
    );
}

#[test]
fn changed_field_selection_has_a_distinct_canonical_contract_without_changing_strict_meaning() {
    let strict = identity("Entity", "Aspect", "Field", 64);
    let changed = demand_identity(DeclaredPreImageDemand::changed_fields_for_output(
        Target::<WorthQueryPreserveOutput, WorthQueryExactlyOneOutput>::DESCRIPTOR,
        [field()],
        64,
    ));
    assert_ne!(strict, changed);
    assert_eq!(
        strict,
        demand_identity(DeclaredPreImageDemand::new([field()], 64))
    );
    assert_ne!(
        changed,
        demand_identity(DeclaredPreImageDemand::changed_fields_for_output(
            Target::<WorthQueryPreserveOutput, WorthQueryExactlyOneOutput>::DESCRIPTOR,
            [field()],
            65
        ))
    );
}

#[test]
fn complete_fields_require_one_retired_output_and_have_distinct_canonical_meaning() {
    use crate::application_aftermath::DeclaredPreImageDemandDenial as Denial;
    let retired = Target::<WorthQueryRetireOutput, WorthQueryExactlyOneOutput>::DESCRIPTOR;
    assert_eq!(
        DeclaredPreImageDemand::retired_fields_for_output(retired, [field()], 64).validate(),
        Ok(())
    );
    for role in [
        Target::<WorthQueryPreserveOutput, WorthQueryExactlyOneOutput>::DESCRIPTOR,
        Target::<WorthQueryCreateOutput, WorthQueryExactlyOneOutput>::DESCRIPTOR,
        Target::<WorthQueryRetireOutput, WorthQueryAtMostOneOutput>::DESCRIPTOR,
    ] {
        assert_eq!(
            DeclaredPreImageDemand::retired_fields_for_output(role, [field()], 64).validate(),
            Err(Denial::InvalidOutputFieldCapture)
        );
    }
    assert_eq!(
        DeclaredPreImageDemand::retired_fields_for_output(retired, [], 64).validate(),
        Err(Denial::InvalidOutputFieldCapture)
    );
    assert_eq!(
        DeclaredPreImageDemand::retired_fields_for_output(retired, [field()], 0).validate(),
        Err(Denial::ZeroByteBound)
    );
    assert_eq!(
        DeclaredPreImageDemand::retired_fields_for_output(retired, [field(), field()], 64)
            .validate(),
        Err(Denial::DuplicateLocus)
    );
    let complete = demand_identity(DeclaredPreImageDemand::retired_fields_for_output(
        retired,
        [field()],
        64,
    ));
    assert_ne!(
        complete,
        demand_identity(DeclaredPreImageDemand::new([field()], 64))
    );
    assert_ne!(
        complete,
        demand_identity(DeclaredPreImageDemand::changed_fields_for_output(
            Target::<WorthQueryPreserveOutput, WorthQueryExactlyOneOutput>::DESCRIPTOR,
            [field()],
            64
        ))
    );
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
