//! Real installed field contracts enter the producer edition through ordinary sealing.
pub use crate::domain_computation::primary_graph::application_contribution::producer::registry::tests::*;
pub use worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_contribution::producer::output_equivalence_contract::seal;
pub use worth_query_execution::facade::application_contribution::WorthQueryProducerOutputEquivalence;
pub use worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_contribution::producer::registry::semantic_edition::InstalledProducerEdition;
pub use schema::ApplicationSchema;
pub use worth_query_declaration::facade::application_schema as schema;
pub use worth_query_installation::facade as domain;

pub trait Fixture: ApplicationSchema {}
worth_query_declaration::worth_query_entity!(pub Output for S: Fixture);
worth_query_declaration::worth_query_aspect!(Values for S: Fixture, Output;
    identity = AspectIdentity(0x0071_9810), revision = AspectContractRevision(1));
worth_query_declaration::worth_query_field!(Value for S: Fixture, Output, Values:
    u64 => schema::U64ApplicationValueBinding, read_write, no_equality);
worth_query_declaration::worth_query_field!(Sibling for S: Fixture, Output, Values:
    u64 => schema::U64ApplicationValueBinding, read_write, no_equality);

macro_rules! fixture {
    ($S:ident, $aspect:path, $mode:ident) => {
        worth_query_declaration::worth_query_application_schema!(schema $S {
            owner: $S, version: (1,0), members: |schema| {
                schema.entity(Output::reference())
                    .optional_aspect(Output::reference(), <$aspect>::reference())
                    .$mode(Output::reference(), Value::reference())
            }
        });
        impl Fixture for $S {}
    };
}
fixture!(Struct, Values, field);
fixture!(Scalar, Values, scalar_aspect_field);
fixture!(RevisionTwo, crate::domain_computation::primary_graph::application_contribution::producer::registry::tests::output_equivalence::changed::Values, field);
worth_query_declaration::worth_query_application_schema!(schema Missing {
    owner: Missing, version: (1,0), members: |schema| {
        schema.entity(Output::reference())
            .optional_aspect(Output::reference(), Values::reference())
            .field(Output::reference(), Sibling::reference())
    }
});
impl Fixture for Missing {}
pub struct Role<S>(std::marker::PhantomData<S>);
impl<S: Fixture> WorthQueryApplicationOutputRole for Role<S> {
    type Schema = S;
    type Contract = NoApplicationMutationOutputs;
    type Entity = Output;
    type Action = WorthQueryPreserveOutput;
    type Cardinality = WorthQueryExactlyOneOutput;
    const NAME: &'static str = "number";
}
pub fn installed<S: Fixture>() -> domain::WorthQueryInstalledApplicationSchema<S> {
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
pub fn declaration<S: Fixture>() -> DeclaredProducerBinding {
    let mut declaration = declared(TypeId::of_for_test::<InstalledProducer>());
    let field =
        WorthQueryProducerOutputEquivalence::<S, Role<S>>::exact_u64(Value::reference::<S>())
            .into_field_for_test();
    (*declaration.output_equivalence_mut_for_test()) = Some(field.clone());
    (*declaration.installed_output_equivalence_mut_for_test()) = Some(
        seal(
            declaration.identity_for_test(),
            declaration.output_role_for_test(),
            &field,
            &installed::<S>(),
        )
        .unwrap(),
    );
    declaration
}
pub fn edition(declaration: &DeclaredProducerBinding) -> [u8; 32] {
    InstalledProducerEdition::from_declaration_for_test(
        declaration,
        worth_foundational::facade::ApplicationBuildIdentity::parse(
            "11111111-1111-1111-1111-111111111111",
        )
        .unwrap(),
    )
    .unwrap()
    .digest_for_test()
}

#[test]
fn installed_output_equivalence_binds_field_shape_revision_role_and_opt_in() {
    let structure = declaration::<Struct>();
    assert_eq!(edition(&structure), edition(&declaration::<Struct>()));
    for distinct in [
        declared(TypeId::of_for_test::<InstalledProducer>()),
        declaration::<Scalar>(),
        declaration::<RevisionTwo>(),
    ] {
        assert!(!structure.has_same_meaning_as_for_test(&distinct));
        assert_ne!(edition(&structure), edition(&distinct));
    }
    let field = (*structure.output_equivalence_for_test()).as_ref().unwrap();
    let different_role = seal("producer", "another-role", field, &installed::<Struct>()).unwrap();
    assert_ne!(
        (*structure
            .installed_output_equivalence_into_for_test()
            .unwrap()
            .meaning_for_test()),
        (*different_role.meaning_for_test())
    );
}

#[test]
fn missing_output_field_refuses_instead_of_substituting_its_sibling() {
    let field = WorthQueryProducerOutputEquivalence::<Missing, Role<Missing>>::exact_u64(
        Value::reference::<Missing>(),
    )
    .into_field_for_test();
    let denied = seal("producer", "number", &field, &installed::<Missing>()).unwrap_err();
    assert_eq!(denied.kind(), worth_query_execution::facade::domain_computation::WorthQueryPrimaryGraphInstallationDenialKind::ProducerBindingMeaningMismatch);
    assert!(denied
        .subject()
        .contains("absent from the completed declaration"));
}

#[test]
fn edition_refuses_unsealed_or_substituted_output_equivalence() {
    let valid = declaration::<Struct>();
    let mut unsealed = valid.clone();
    (*unsealed.installed_output_equivalence_mut_for_test()) = None;
    let mut substituted = valid.clone();
    (*substituted.output_equivalence_mut_for_test()) = Some(
        WorthQueryProducerOutputEquivalence::<Struct, Role<Struct>>::exact_u64(
            Sibling::reference::<Struct>(),
        )
        .into_field_for_test(),
    );
    assert!(!valid.has_same_meaning_as_for_test(&substituted));
    for invalid in [unsealed, substituted] {
        assert!(InstalledProducerEdition::from_declaration_for_test(
            &invalid,
            worth_foundational::facade::ApplicationBuildIdentity::parse(
                "11111111-1111-1111-1111-111111111111",
            )
            .unwrap(),
        )
        .is_err());
    }
}

#[path = "association/mod.rs"]
pub mod association;
#[path = "changed/mod.rs"]
pub mod changed;
