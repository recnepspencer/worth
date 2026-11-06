//! Real installed field contracts enter the producer edition through ordinary sealing.
use super::*;
use crate::domain_computation::primary_graph::application_contribution::producer::{
    output_equivalence_contract::{seal, WorthQueryProducerOutputEquivalence},
    registry::InstalledProducerEdition,
};
use schema::ApplicationSchema;
use worth_query_declaration::facade::application_schema as schema;
use worth_query_installation::facade as domain;

use schema::fragment::{self, RecordSelection};
#[path = "output_equivalence/records.rs"]
mod records;
use records::*;
trait Fixture: ApplicationSchema {
    type Entity: fragment::Entity;
    type Field: RecordSelection<
        Entity = Self::Entity,
        Value = u64,
        Binding = schema::U64ApplicationValueBinding,
    >;
    fn field() -> Self::Field;
}
struct Struct;
impl schema::ApplicationSchema for Struct {
    const OWNER: &'static str = "Struct";
    const NAME: &'static str = "Struct";
    const MAJOR: u32 = 1;
    const MINOR: u32 = 0;
    fn declaration() -> Result<
        schema::ApplicationSchemaDeclaration<Self>,
        schema::ApplicationSchemaDeclarationDenial,
    > {
        schema::ApplicationSchemaDeclarationBuilder::from_fragment(Model::<0>)?.build()
    }
}
impl Fixture for Struct {
    type Entity = Output<0>;
    type Field = fragment::FieldSelection<Model<0>, Output<0>, Values<0>, u64>;
    fn field() -> Self::Field {
        Self::Field::at("Value")
    }
}
struct Scalar;
impl schema::ApplicationSchema for Scalar {
    const OWNER: &'static str = "Scalar";
    const NAME: &'static str = "Scalar";
    const MAJOR: u32 = 1;
    const MINOR: u32 = 0;
    fn declaration() -> Result<
        schema::ApplicationSchemaDeclaration<Self>,
        schema::ApplicationSchemaDeclarationDenial,
    > {
        schema::ApplicationSchemaDeclarationBuilder::from_fragment(Model::<1>)?.build()
    }
}
impl Fixture for Scalar {
    type Entity = Output<1>;
    type Field = fragment::FieldSelection<Model<1>, Output<1>, Values<1>, u64>;
    fn field() -> Self::Field {
        Self::Field::at("value")
    }
}
struct ShapeChanged;
impl schema::ApplicationSchema for ShapeChanged {
    const OWNER: &'static str = "ShapeChanged";
    const NAME: &'static str = "ShapeChanged";
    const MAJOR: u32 = 1;
    const MINOR: u32 = 0;
    fn declaration() -> Result<
        schema::ApplicationSchemaDeclaration<Self>,
        schema::ApplicationSchemaDeclarationDenial,
    > {
        schema::ApplicationSchemaDeclarationBuilder::from_fragment(Model::<2>)?.build()
    }
}
impl Fixture for ShapeChanged {
    type Entity = Output<2>;
    type Field = fragment::FieldSelection<Model<2>, Output<2>, Values<2>, u64>;
    fn field() -> Self::Field {
        Self::Field::at("Value")
    }
}
struct Missing;
impl schema::ApplicationSchema for Missing {
    const OWNER: &'static str = "Missing";
    const NAME: &'static str = "Missing";
    const MAJOR: u32 = 1;
    const MINOR: u32 = 0;
    fn declaration() -> Result<
        schema::ApplicationSchemaDeclaration<Self>,
        schema::ApplicationSchemaDeclarationDenial,
    > {
        schema::ApplicationSchemaDeclarationBuilder::from_fragment(Model::<3>)?.build()
    }
}
impl Fixture for Missing {
    type Entity = Output<3>;
    type Field = fragment::FieldSelection<Model<3>, Output<3>, Values<3>, u64>;
    fn field() -> Self::Field {
        Self::Field::at("Sibling")
    }
}
struct Role<S>(std::marker::PhantomData<S>);
impl<S: Fixture> WorthQueryApplicationOutputRole for Role<S> {
    type Schema = S;
    type Contract = NoApplicationMutationOutputs;
    type Entity = S::Entity;
    type Action = WorthQueryPreserveOutput;
    type Cardinality = WorthQueryExactlyOneOutput;
    const NAME: &'static str = "number";
}
fn installed<S: Fixture>() -> domain::WorthQueryInstalledApplicationSchema<S> {
    let declaration = S::declaration().unwrap();
    let admitted = domain::WorthQueryInstallationAdmissionProfile::new("equivalence", "field")
        .admit(
            domain::WorthQueryPortableDomainPackage::new(
                domain::WorthQueryPortableDomainIdentity::new(S::OWNER, 1, 0),
            )
            .application_schema(declaration.clone())
            .validate()
            .unwrap(),
        )
        .unwrap();
    domain::WorthQueryInstalledPackageIndex::build(
        domain::WorthQueryInstallationRuntimeIdentity::fresh(),
        domain::WorthQueryInstallationGeneration::initial(),
        [admitted],
    )
    .unwrap()
    .bind_application_schema(declaration)
    .unwrap()
}
fn declaration<S: Fixture>() -> DeclaredProducerBinding {
    let mut declaration = declared(TypeId::of::<InstalledProducer>());
    let field = WorthQueryProducerOutputEquivalence::<S, Role<S>>::exact_u64(S::field())
        .unwrap()
        .into_field();
    declaration.output_equivalence = Some(field.clone());
    declaration.installed_output_equivalence = Some(
        seal(
            &declaration.identity,
            &declaration.output_role,
            &field,
            &installed::<S>(),
        )
        .unwrap(),
    );
    declaration
}
fn edition(declaration: &DeclaredProducerBinding) -> [u8; 32] {
    InstalledProducerEdition::from_declaration(
        declaration,
        worth_foundational::facade::ApplicationBuildIdentity::parse(
            "11111111-1111-1111-1111-111111111111",
        )
        .unwrap(),
    )
    .unwrap()
    .digest()
}

#[test]
fn installed_output_equivalence_binds_field_shape_revision_role_and_opt_in() {
    let structure = declaration::<Struct>();
    assert_eq!(edition(&structure), edition(&declaration::<Struct>()));
    for distinct in [
        declared(TypeId::of::<InstalledProducer>()),
        declaration::<Scalar>(),
        declaration::<ShapeChanged>(),
    ] {
        assert!(!structure.has_same_meaning_as(&distinct));
        assert_ne!(edition(&structure), edition(&distinct));
    }
    let field = structure.output_equivalence.as_ref().unwrap();
    let different_role = seal("producer", "another-role", field, &installed::<Struct>()).unwrap();
    assert_ne!(
        structure.installed_output_equivalence.unwrap().meaning,
        different_role.meaning
    );
}

#[test]
fn missing_output_field_refuses_instead_of_substituting_its_sibling() {
    let field =
        WorthQueryProducerOutputEquivalence::<Struct, Role<Struct>>::exact_u64(Struct::field())
            .unwrap()
            .into_field();
    let denied = seal("producer", "number", &field, &installed::<Missing>()).unwrap_err();
    assert_eq!(denied.kind(), crate::domain_computation::primary_graph::WorthQueryPrimaryGraphInstallationDenialKind::ProducerBindingMeaningMismatch);
    assert!(denied
        .subject()
        .contains("absent from the completed declaration"));
}

#[test]
fn edition_refuses_unsealed_or_substituted_output_equivalence() {
    let valid = declaration::<Struct>();
    let mut unsealed = valid.clone();
    unsealed.installed_output_equivalence = None;
    let mut substituted = valid.clone();
    let mut forged = valid.output_equivalence.clone().unwrap();
    forged.locus =
        schema::ApplicationFieldBindingLocus::new("Output", Struct::field().aspect(), "Sibling");
    substituted.output_equivalence = Some(forged);
    assert!(!valid.has_same_meaning_as(&substituted));
    for invalid in [unsealed, substituted] {
        assert!(InstalledProducerEdition::from_declaration(
            &invalid,
            worth_foundational::facade::ApplicationBuildIdentity::parse(
                "11111111-1111-1111-1111-111111111111",
            )
            .unwrap(),
        )
        .is_err());
    }
}

mod association;
