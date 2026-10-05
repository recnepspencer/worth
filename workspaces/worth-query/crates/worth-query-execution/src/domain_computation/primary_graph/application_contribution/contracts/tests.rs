use std::any::TypeId;

use super::{DeclaredProducerBinding, DenialKind, WorthQueryApplicationContractCatalog};
use crate::domain_computation::primary_graph::application_contribution::{
    WorthQueryProducerApplicability, WorthQueryProducerLifecyclePosture,
};
use worth_query_declaration::facade::{
    application_operation::{
        ApplicationMutationOutputPostureSet, NoApplicationMutationOutputs,
        WorthQueryApplicationDeclaredOutputRole, WorthQueryApplicationDeclaredOutputRoleFamily,
        WorthQueryApplicationOutputRole, WorthQueryApplicationOutputRoleFamily,
        WorthQueryAtMostOneOutput, WorthQueryCreateOutput, WorthQueryExactlyOneOutput,
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

/// A producer's output role is a fixed exactly-one role marker, so a
/// producer that names a family member fails to compile. The erased
/// declaration is still refused, member or bare prefix alike.
#[test]
fn a_producer_role_naming_a_family_member_is_denied() {
    for output_role in ["created.primary", "created."] {
        let mut catalog = WorthQueryApplicationContractCatalog::<TestSchema>::default();
        catalog
            .producers
            .insert("initial".to_owned(), family_producer(output_role));

        let denial = catalog.validate().unwrap_err();
        assert_eq!(denial.kind(), DenialKind::ProducerBindingMeaningMismatch);
        assert_eq!(denial.subject(), "initial");
    }
}

#[test]
fn producer_output_role_must_be_bound_by_every_commit() {
    let mut catalog = WorthQueryApplicationContractCatalog::<TestSchema>::default();
    let mut binding = producer("initial", "create-source", &[INITIAL]);
    binding.output_role_descriptors =
        vec![<OptionalOutput as WorthQueryApplicationDeclaredOutputRole>::DESCRIPTOR];
    catalog.producers.insert("initial".to_owned(), binding);

    let denial = catalog.validate().unwrap_err();
    assert_eq!(denial.kind(), DenialKind::ProducerBindingMeaningMismatch);
    assert_eq!(denial.subject(), "initial");
}

fn family_producer(output_role: &str) -> DeclaredProducerBinding {
    let mut binding = producer("initial", "create-source", &[INITIAL]);
    binding.output_role_descriptors.clear();
    binding.output_role_families =
        vec![<Created as WorthQueryApplicationDeclaredOutputRoleFamily>::DESCRIPTOR];
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
        output_role_descriptors: vec![
            <Output as WorthQueryApplicationDeclaredOutputRole>::DESCRIPTOR,
        ],
        output_role_families: Vec::new(),
        output_role: "output".to_owned(),
        operation: "operation".to_owned(),
        handler_identity: "handler".to_owned(),
        provider_identity: format!("{identity}-provider"),
        applicability: supported.to_vec(),
        supported: supported.to_vec(),
        required_invariants: Vec::new(),
        resource_policy: "bounded".to_owned(),
        reuse_policy: "exact-source".to_owned(),
        input_reuse: None,
        binding_type: TypeId::of::<()>(),
        source_type: TypeId::of::<()>(),
        provider_type: TypeId::of::<()>(),
        operation_binding_type: TypeId::of::<()>(),
        output_contract_type: TypeId::of::<()>(),
    }
}

struct Output;
impl WorthQueryApplicationOutputRole for Output {
    type Schema = TestSchema;
    type Contract = NoApplicationMutationOutputs;
    type Entity = TestEntity;
    type Action = WorthQueryCreateOutput;
    type Cardinality = WorthQueryExactlyOneOutput;
    const NAME: &'static str = "output";
}

struct OptionalOutput;
impl WorthQueryApplicationOutputRole for OptionalOutput {
    type Schema = TestSchema;
    type Contract = NoApplicationMutationOutputs;
    type Entity = TestEntity;
    type Action = WorthQueryCreateOutput;
    type Cardinality = WorthQueryAtMostOneOutput;
    const NAME: &'static str = "output";
}

struct Created;
impl WorthQueryApplicationOutputRoleFamily for Created {
    type Schema = TestSchema;
    type Contract = NoApplicationMutationOutputs;
    type Entity = TestEntity;
    const PREFIX: &'static str = "created.";
    const POSTURES: ApplicationMutationOutputPostureSet =
        ApplicationMutationOutputPostureSet::CREATE;
    const MINIMUM: usize = 0;
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
