use std::any::TypeId;

use super::{DeclaredProducerBinding, DenialKind, WorthQueryApplicationContractCatalog};
use crate::domain_computation::primary_graph::application_contribution::{
    WorthQueryProducerApplicability, WorthQueryProducerLifecyclePosture,
};
use worth_query_declaration::facade::{
    application_operation::{
        ApplicationMutationOutputPostureSet, ApplicationMutationOutputRoleFamilyDescriptor,
    },
    application_schema::ApplicationEntityMarkerIdentity,
};

const INITIAL: WorthQueryProducerApplicability =
    WorthQueryProducerApplicability::new("rectangle", WorthQueryProducerLifecyclePosture::Initial);
const PRESERVE: WorthQueryProducerApplicability =
    WorthQueryProducerApplicability::new("rectangle", WorthQueryProducerLifecyclePosture::Preserve);

#[test]
fn output_family_meaning_cannot_change_with_source_binding() {
    let mut catalog = WorthQueryApplicationContractCatalog::<TestSchema>::default();
    catalog.producers.insert(
        "initial".to_owned(),
        producer("initial", "create-source", &[INITIAL]),
    );
    catalog.producers.insert(
        "preserve".to_owned(),
        producer("preserve", "edit-source", &[PRESERVE]),
    );

    let denial = catalog.validate().unwrap_err();
    assert_eq!(denial.kind(), DenialKind::ProducerBindingMeaningMismatch);
    assert_eq!(denial.subject(), "family");
}

#[test]
fn concrete_producer_role_may_be_declared_by_an_output_role_family() {
    let mut catalog = WorthQueryApplicationContractCatalog::<TestSchema>::default();
    catalog
        .producers
        .insert("initial".to_owned(), family_producer("created.primary"));

    assert!(catalog.validate().is_ok());
}

#[test]
fn output_role_family_prefix_without_a_member_is_denied() {
    let mut catalog = WorthQueryApplicationContractCatalog::<TestSchema>::default();
    catalog
        .producers
        .insert("initial".to_owned(), family_producer("created."));

    let denial = catalog.validate().unwrap_err();
    assert_eq!(denial.kind(), DenialKind::ProducerBindingMeaningMismatch);
    assert_eq!(denial.subject(), "initial");
}

fn family_producer(output_role: &str) -> DeclaredProducerBinding {
    let mut binding = producer("initial", "create-source", &[INITIAL]);
    binding.output_roles.clear();
    binding.output_role_families =
        vec![ApplicationMutationOutputRoleFamilyDescriptor::for_entity::<
            TestSchema,
            TestEntity,
        >(
            "created.", ApplicationMutationOutputPostureSet::CREATE, 0
        )];
    binding.output_role = output_role.to_owned();
    binding
}

fn producer(
    identity: &str,
    source: &str,
    supported: &[WorthQueryProducerApplicability],
) -> DeclaredProducerBinding {
    DeclaredProducerBinding {
        owner: "owner".to_owned(),
        identity: identity.to_owned(),
        source_selector: source.to_owned(),
        output_family: "family".to_owned(),
        output_family_type: TypeId::of::<()>(),
        output_roles: vec!["output".to_owned()],
        output_role_families: Vec::new(),
        output_role: "output".to_owned(),
        operation: "operation".to_owned(),
        provider_identity: format!("{identity}-provider"),
        applicability: supported.to_vec(),
        supported: supported.to_vec(),
        required_invariants: Vec::new(),
        resource_policy: "bounded".to_owned(),
        reuse_policy: "exact-source".to_owned(),
        binding_type: TypeId::of::<()>(),
        source_type: TypeId::of::<()>(),
        provider_type: TypeId::of::<()>(),
        operation_binding_type: TypeId::of::<()>(),
    }
}

struct TestSchema;
struct TestEntity;

impl ApplicationEntityMarkerIdentity<TestSchema> for TestEntity {
    const IDENTIFIER: &'static str = "TestEntity";
}

impl worth_query_installation::facade::ApplicationSchema for TestSchema {
    const OWNER: &'static str = "owner";
    const NAME: &'static str = "schema";
    const MAJOR: u32 = 1;
    const MINOR: u32 = 0;

    fn declaration() -> Result<
        worth_query_declaration::facade::application_schema::ApplicationSchemaDeclaration<Self>,
        worth_query_declaration::facade::application_schema::ApplicationSchemaDeclarationDenial,
    > {
        unreachable!()
    }
}
