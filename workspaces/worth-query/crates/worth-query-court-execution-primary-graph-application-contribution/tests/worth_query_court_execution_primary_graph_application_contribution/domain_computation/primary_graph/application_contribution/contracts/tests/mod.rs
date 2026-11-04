use worth_query_execution::court_doors_for_test::equality_type_id::EqualityTypeId as TypeId;

use worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_contribution::producer::registry::DeclaredProducerBinding;
use worth_query_execution::facade::domain_computation::WorthQueryPrimaryGraphInstallationDenialKind as DenialKind;
use worth_query_execution::facade::application_contribution::WorthQueryApplicationContractCatalog;
use worth_query_execution::facade::application_contribution::WorthQueryProducerApplicability;
use worth_query_execution::facade::application_contribution::WorthQueryProducerLifecyclePosture;
use worth_query_declaration::facade::application_operation::ApplicationMutationOutputPostureSet;
use worth_query_declaration::facade::application_operation::NoApplicationMutationOutputs;
use worth_query_declaration::facade::application_operation::WorthQueryApplicationDeclaredOutputRole;
use worth_query_declaration::facade::application_operation::WorthQueryApplicationDeclaredOutputRoleFamily;
use worth_query_declaration::facade::application_operation::WorthQueryApplicationOutputRole;
use worth_query_declaration::facade::application_operation::WorthQueryApplicationOutputRoleFamily;
use worth_query_declaration::facade::application_operation::WorthQueryAtMostOneOutput;
use worth_query_declaration::facade::application_operation::WorthQueryCreateOutput;
use worth_query_declaration::facade::application_operation::WorthQueryExactlyOneOutput;
use worth_query_declaration::facade::application_schema::ApplicationEntityMarkerIdentity;

pub const INITIAL: WorthQueryProducerApplicability =
    WorthQueryProducerApplicability::new("rectangle", WorthQueryProducerLifecyclePosture::Initial);
pub const PRESERVE: WorthQueryProducerApplicability =
    WorthQueryProducerApplicability::new("rectangle", WorthQueryProducerLifecyclePosture::Preserve);

#[test]
fn distinct_family_types_cannot_share_one_declared_identity_at_installation() {
    use worth_query_execution::facade::application_contribution::WorthQueryProducerOutputFamily;
    use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::IdentityExecutionSchema;
    type FirstFamily = CollidingFamily<0>;
    type SecondFamily = CollidingFamily<1>;
    let mut catalog = WorthQueryApplicationContractCatalog::<IdentityExecutionSchema>::default();
    let mut first = producer("initial", "source", &[INITIAL]);
    let mut second = producer("preserve", "source", &[PRESERVE]);
    (*first.output_family_mut_for_test()) = FirstFamily::IDENTITY.into();
    (*second.output_family_mut_for_test()) = SecondFamily::IDENTITY.into();
    (*first.supported_mut_for_test()) = vec![INITIAL, PRESERVE];
    (*second.supported_mut_for_test()) = (*first.supported_for_test()).clone();
    (*first.output_family_type_mut_for_test()) = TypeId::of_for_test::<FirstFamily>();
    (*second.output_family_type_mut_for_test()) = (*first.output_family_type_for_test());
    (*catalog.producers_for_test()).insert("initial".into(), first);
    (*catalog.producers_for_test()).insert("preserve".into(), second);
    catalog
        .validate_for_test()
        .expect("one family may have two lifecycle bindings");
    (*catalog
        .producers_for_test())
        .get_mut("preserve")
        .unwrap()
        .output_family_type = TypeId::of_for_test::<SecondFamily>();
    let denied = catalog
        .validate_for_test()
        .expect_err("different types cannot declare the same family");
    assert_eq!(denied.kind(), DenialKind::ProducerBindingMeaningMismatch);
    assert_eq!(denied.subject(), "family");
}

pub struct CollidingFamily<const N: u8>;
impl<const N: u8>
    worth_query_execution::facade::application_contribution::WorthQueryProducerOutputFamily<
        worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::IdentityExecutionSchema,
    > for CollidingFamily<N>
{
    type Source =
        worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::current_output_source::TestAccountSourceBinding;
    type Entity = worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::Account;
    const IDENTITY: &'static str = "family";
    const SUPPORTED: &'static [WorthQueryProducerApplicability] = &[INITIAL, PRESERVE];
    fn profile_kind(
        _: &worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::application_queries::AccountSummaryResult,
    ) -> &'static str {
        "rectangle"
    }
}

#[test]
fn output_family_meaning_cannot_change_with_source_binding() {
    let mut catalog = WorthQueryApplicationContractCatalog::<TestSchema>::default();
    (*catalog.producers_for_test()).insert(
        "initial".to_owned(),
        producer("initial", "create-source", &[INITIAL]),
    );
    (*catalog.producers_for_test()).insert(
        "preserve".to_owned(),
        producer("preserve", "edit-source", &[PRESERVE]),
    );

    let denial = catalog.validate_for_test().unwrap_err();
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
        (*catalog
            .producers_for_test())
            .insert("initial".to_owned(), family_producer(output_role));

        let denial = catalog.validate_for_test().unwrap_err();
        assert_eq!(denial.kind(), DenialKind::ProducerBindingMeaningMismatch);
        assert_eq!(denial.subject(), "initial");
    }
}

#[test]
fn producer_output_role_must_be_bound_by_every_commit() {
    let mut catalog = WorthQueryApplicationContractCatalog::<TestSchema>::default();
    let mut binding = producer("initial", "create-source", &[INITIAL]);
    (*binding.output_role_descriptors_mut_for_test()) =
        vec![<OptionalOutput as WorthQueryApplicationDeclaredOutputRole>::DESCRIPTOR];
    (*catalog.producers_for_test()).insert("initial".to_owned(), binding);

    let denial = catalog.validate_for_test().unwrap_err();
    assert_eq!(denial.kind(), DenialKind::ProducerBindingMeaningMismatch);
    assert_eq!(denial.subject(), "initial");
}

pub fn family_producer(output_role: &str) -> DeclaredProducerBinding {
    let mut binding = producer("initial", "create-source", &[INITIAL]);
    (*binding.output_role_descriptors_for_test()).clear();
    (*binding.output_role_families_mut_for_test()) =
        vec![<Created as WorthQueryApplicationDeclaredOutputRoleFamily>::DESCRIPTOR];
    (*binding.output_role_mut_for_test()) = output_role.to_owned();
    binding
}

pub fn producer(
    identity: &str,
    source: &str,
    supported: &[WorthQueryProducerApplicability],
) -> DeclaredProducerBinding {
    DeclaredProducerBinding {
        ownership: worth_foundational::facade::ProducerPublicationOwnership::Independent,
        owned_scope_entity: None,
        owner: "owner".to_owned(),
        identity: identity.to_owned(),
        source_selector: source.to_owned(),
        output_family: "family".to_owned(),
        output_family_type: TypeId::of_for_test::<()>(),
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

        reuse_policy: "exact-source".to_owned(),
        input_reuse: None,
        output_equivalence: None,
        installed_output_equivalence: None,
        binding_type: TypeId::of_for_test::<()>(),
        source_type: TypeId::of_for_test::<()>(),
        provider_type: TypeId::of_for_test::<()>(),
        operation_binding_type: TypeId::of_for_test::<()>(),
        operation_type: TypeId::of_for_test::<()>(),
        output_contract_type: TypeId::of_for_test::<()>(),
    }
}

pub struct Output;
impl WorthQueryApplicationOutputRole for Output {
    type Schema = TestSchema;
    type Contract = NoApplicationMutationOutputs;
    type Entity = TestEntity;
    type Action = WorthQueryCreateOutput;
    type Cardinality = WorthQueryExactlyOneOutput;
    const NAME: &'static str = "output";
}

pub struct OptionalOutput;
impl WorthQueryApplicationOutputRole for OptionalOutput {
    type Schema = TestSchema;
    type Contract = NoApplicationMutationOutputs;
    type Entity = TestEntity;
    type Action = WorthQueryCreateOutput;
    type Cardinality = WorthQueryAtMostOneOutput;
    const NAME: &'static str = "output";
}

pub struct Created;
impl WorthQueryApplicationOutputRoleFamily for Created {
    type Schema = TestSchema;
    type Contract = NoApplicationMutationOutputs;
    type Entity = TestEntity;
    const PREFIX: &'static str = "created.";
    const POSTURES: ApplicationMutationOutputPostureSet =
        ApplicationMutationOutputPostureSet::CREATE;
    const MINIMUM: usize = 0;
}

pub struct TestSchema;
pub struct TestEntity;

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

#[path = "package_composition/mod.rs"]
pub mod package_composition;
