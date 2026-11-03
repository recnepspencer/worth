pub use worth_query_declaration::facade::application_operation::ApplicationMutationOutputPosture as WorthQueryApplicationOutputPosture;
pub use worth_query_execution::court_doors_for_test::equality_type_id::EqualityTypeId as TypeId;
pub use worth_query_execution::facade::domain_computation::WorthQueryPrimaryGraphInstallationDenialKind as DenialKind;

pub use worth_query_declaration::facade::application_operation::ApplicationMutationOutputPostureSet;
pub use worth_query_declaration::facade::application_operation::NoApplicationMutationOutputs;
pub use worth_query_declaration::facade::application_operation::WorthQueryApplicationDeclaredOutputRole as DeclaredRole;
pub use worth_query_declaration::facade::application_operation::WorthQueryApplicationDeclaredOutputRoleFamily as DeclaredFamily;
pub use worth_query_declaration::facade::application_operation::WorthQueryApplicationOutputRole;
pub use worth_query_declaration::facade::application_operation::WorthQueryApplicationOutputRoleFamily;
pub use worth_query_declaration::facade::application_operation::WorthQueryAtMostOneOutput;
pub use worth_query_declaration::facade::application_operation::WorthQueryExactlyOneOutput;
pub use worth_query_declaration::facade::application_operation::WorthQueryPreserveOutput;
pub use worth_query_declaration::facade::application_schema::ApplicationEntityMarkerIdentity;

pub use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::Account;
pub use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::IdentityExecutionSchema as Schema;

pub use worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_contribution::producer::registry::checkpoint::installed_checkpoint_producer;
pub use worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_contribution::producer::registry::checkpoint::validate_checkpoint_output_meaning;
pub use worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_contribution::producer::registry::operation_binding_uniqueness::duplicate_operation_binding;
pub use worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_contribution::producer::registry::DeclaredProducerBinding;
pub use worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_contribution::producer::registry::semantic_edition::InstalledProducerEdition;
pub use worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_contribution::producer::registry::InstalledProducerProvider;

#[test]
fn every_producer_edition_includes_the_application_build_even_without_input_reuse() {
    use worth_foundational::facade::ApplicationBuildIdentity;
    let first = ApplicationBuildIdentity::parse("11111111-1111-1111-1111-111111111111").unwrap();
    let next = ApplicationBuildIdentity::parse("22222222-2222-2222-2222-222222222222").unwrap();
    for input_reuse in [
        None,
        Some(
            worth_query_execution::facade::application_contribution::WorthQueryProducerInputReuseContract::canonical_bitwise(
                worth_query_execution::facade::application_contribution::WorthQueryDecisionContextDependencies::NONE,
            ),
        ),
    ] {
        let mut declaration = declared(TypeId::of::<InstalledProducer>());
        declaration.input_reuse = input_reuse;
        let installed = InstalledProducerEdition::from_declaration(&declaration, first).unwrap();
        let reopened = InstalledProducerEdition::from_declaration(&declaration, first).unwrap();
        let rebuilt = InstalledProducerEdition::from_declaration(&declaration, next).unwrap();
        assert_eq!(installed, reopened);
        assert_eq!(installed.build_identity(), first);
        assert_eq!(rebuilt.build_identity(), next);
        assert_ne!(installed.digest(), rebuilt.digest());
    }
}

#[test]
fn producer_input_reuse_is_portable_installed_meaning_bound_to_the_handler() {
    let base = declared(TypeId::of::<InstalledProducer>());
    let mut opted_in = base.clone();
    opted_in.input_reuse = Some(
        worth_query_execution::facade::application_contribution::WorthQueryProducerInputReuseContract::canonical_bitwise(
            worth_query_execution::facade::application_contribution::WorthQueryDecisionContextDependencies::NONE,
        ),
    );
    let mut changed_handler = opted_in.clone();
    changed_handler.handler_identity = "another-handler".into();
    let mut changed_dependencies = opted_in.clone();
    changed_dependencies.input_reuse = Some(
        worth_query_execution::facade::application_contribution::WorthQueryProducerInputReuseContract::canonical_bitwise(
            worth_query_execution::facade::application_contribution::WorthQueryDecisionContextDependencies::KEY,
        ),
    );

    assert!(!base.has_same_meaning_as(&opted_in));
    let edition = |declaration| {
        InstalledProducerEdition::from_declaration(
            declaration,
            worth_foundational::facade::ApplicationBuildIdentity::parse(
                "11111111-1111-1111-1111-111111111111",
            )
            .unwrap(),
        )
        .unwrap()
        .digest()
    };
    assert_ne!(edition(&base), edition(&opted_in));
    assert_ne!(edition(&opted_in), edition(&changed_handler));
    assert_ne!(edition(&opted_in), edition(&changed_dependencies));
}

#[test]
fn producer_meaning_rejects_a_distinct_rust_binding_type_reusing_the_identity() {
    let installed = declared(TypeId::of::<InstalledProducer>());
    let substituted = declared(TypeId::of::<SubstitutedProducer>());

    assert_eq!(installed.identity, substituted.identity);
    assert!(!installed.has_same_meaning_as(&substituted));
}

#[test]
fn producer_meaning_rejects_a_provider_version_change() {
    let installed = declared(TypeId::of::<InstalledProducer>());
    let mut changed = installed.clone();
    changed.provider_identity = "provider.v2".into();

    assert!(!installed.has_same_meaning_as(&changed));
}

#[test]
fn one_mutation_binding_cannot_own_two_producer_lineage_slots() {
    let first = declared(TypeId::of::<InstalledProducer>());
    let mut second = declared(TypeId::of::<SubstitutedProducer>());
    second.identity = "producer.two".into();
    let declarations = std::collections::BTreeMap::from([
        (first.identity.clone(), first),
        (second.identity.clone(), second),
    ]);

    assert_eq!(
        duplicate_operation_binding(&declarations),
        Some(("producer", "producer.two"))
    );
}

#[test]
fn checkpoint_role_must_match_installed_producer_meaning() {
    let installed = declared(TypeId::of::<InstalledProducer>());
    let checkpoint = checkpoint(vec![role("unexpected")]);

    assert!(validate_checkpoint_output_meaning(&installed, &checkpoint)
        .unwrap_err()
        .contains("differs from installed producer"));
}

#[test]
fn checkpoint_producer_must_be_installed() {
    let entries =
        std::collections::BTreeMap::<String, std::sync::Arc<InstalledProducerProvider<()>>>::new();
    let denial = match installed_checkpoint_producer(&entries, "missing") {
        Err(denial) => denial,
        Ok(_) => panic!("an absent producer cannot be readmitted"),
    };
    assert!(denial.contains("producer missing is not installed"));
}

#[test]
fn checkpoint_cannot_omit_an_exact_installed_role() {
    let mut installed = declared(TypeId::of::<InstalledProducer>());
    installed.output_role_descriptors = vec![<Required as DeclaredRole>::DESCRIPTOR];

    assert!(
        validate_checkpoint_output_meaning(&installed, &checkpoint(Vec::new()))
            .unwrap_err()
            .contains("omits installed role required")
    );
}

#[test]
fn checkpoint_may_omit_only_an_at_most_one_installed_role() {
    let mut installed = declared(TypeId::of::<InstalledProducer>());
    installed.output_role_descriptors = vec![
        <Required as DeclaredRole>::DESCRIPTOR,
        <Optional as DeclaredRole>::DESCRIPTOR,
    ];
    let checkpoint_of = |names: &[&str]| {
        let mut checkpoint = checkpoint(names.iter().map(|name| role(name)).collect());
        for role in std::sync::Arc::get_mut(&mut checkpoint.roles).expect("unique fixture roles") {
            role.entity_name = Account::IDENTIFIER.into();
        }
        checkpoint
    };

    assert_eq!(
        validate_checkpoint_output_meaning(&installed, &checkpoint_of(&["required"])),
        Ok(())
    );
    assert_eq!(
        validate_checkpoint_output_meaning(&installed, &checkpoint_of(&["required", "optional"])),
        Ok(())
    );
    assert!(
        validate_checkpoint_output_meaning(&installed, &checkpoint_of(&["optional"]))
            .unwrap_err()
            .contains("omits installed role required")
    );
}

#[test]
fn checkpoint_cannot_underfill_an_installed_role_family() {
    let mut installed = declared(TypeId::of::<InstalledProducer>());
    installed.output_role_families = vec![<Member as DeclaredFamily>::DESCRIPTOR];
    let mut checkpoint = checkpoint(vec![role("member.one")]);
    std::sync::Arc::get_mut(&mut checkpoint.roles).expect("unique fixture roles")[0].entity_name =
        Account::IDENTIFIER.into();

    assert!(validate_checkpoint_output_meaning(&installed, &checkpoint)
        .unwrap_err()
        .contains("family member. is incomplete"));
}

pub fn checkpoint(
    roles: Vec<
        worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_attempt::effect_program::output_correspondence::WorthQueryCheckpointOutputRole,
    >,
) -> worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_output_demand::registry::accepted_checkpoint_identity::WorthQueryAcceptedOutputCheckpointIdentity{
    worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_output_demand::registry::accepted_checkpoint_identity::WorthQueryAcceptedOutputCheckpointIdentity {
        implementation: worth_relational::facade::mvcc::RequiredOutputProducingImplementation::from_checkpoint([0x11; 16], [0x22; 32]).unwrap(),
        producer: ("producer".to_owned()).into(),
        posture: worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_output_demand::registry::accepted_checkpoint_identity::WorthQueryAcceptedOutputCheckpointPosture::Performed,
        source: [0; 32],
        scope: worth_query_execution::facade::domain_computation::WorthQueryOperationScopeEntityBinding::from_entity(
            worth_relational::facade::identity::EntityId::new(
                worth_relational::facade::identity::PartitionId(1), 1, 1,
            ),
        ),
        source_partition: [0; 32],
        producer_dependency: None,
        idempotency_key: [0; 32],

        roles: roles.into(),
            producer_facts: None,
            producer_fact_wire_version: 0,
 metadata: None,
}
}

pub fn role(
    name: &str,
) -> worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_attempt::effect_program::output_correspondence::WorthQueryCheckpointOutputRole{
    worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_attempt::effect_program::output_correspondence::WorthQueryCheckpointOutputRole {
        role: name.into(),
        posture:
            WorthQueryApplicationOutputPosture::Preserve,
        entity_name: "account".into(),
        entity: worth_relational::facade::identity::EntityId::new(
            worth_relational::facade::identity::PartitionId(1),
            1,
            1,
        ),
    }
}

pub fn declared(binding_type: TypeId) -> DeclaredProducerBinding {
    DeclaredProducerBinding {
        ownership: worth_foundational::facade::ProducerPublicationOwnership::Independent,
        owned_scope_entity: None,
        owner: "owner".into(),
        identity: "producer".into(),
        source_selector: "source".into(),
        output_family: "family".into(),
        output_family_type: TypeId::of::<()>(),
        output_role_descriptors: Vec::new(),
        output_role_families: Vec::new(),
        output_role: "output".into(),
        operation: "operation".into(),
        handler_identity: "handler".into(),
        provider_identity: "provider".into(),
        applicability: Vec::new(),
        supported: Vec::new(),
        required_invariants: Vec::new(),

        reuse_policy: "exact".into(),
        input_reuse: None,
        output_equivalence: None,
        installed_output_equivalence: None,
        binding_type,
        source_type: TypeId::of::<()>(),
        operation_binding_type: TypeId::of::<()>(),
        operation_type: TypeId::of::<()>(),
        provider_type: TypeId::of::<()>(),
        output_contract_type: TypeId::of::<()>(),
    }
}

pub struct InstalledProducer;
pub struct SubstitutedProducer;

pub struct Required;
impl WorthQueryApplicationOutputRole for Required {
    type Schema = Schema;
    type Contract = NoApplicationMutationOutputs;
    type Entity = Account;
    type Action = WorthQueryPreserveOutput;
    type Cardinality = WorthQueryExactlyOneOutput;
    const NAME: &'static str = "required";
}

pub struct Optional;
impl WorthQueryApplicationOutputRole for Optional {
    type Schema = Schema;
    type Contract = NoApplicationMutationOutputs;
    type Entity = Account;
    type Action = WorthQueryPreserveOutput;
    type Cardinality = WorthQueryAtMostOneOutput;
    const NAME: &'static str = "optional";
}

pub struct Member;
impl WorthQueryApplicationOutputRoleFamily for Member {
    type Schema = Schema;
    type Contract = NoApplicationMutationOutputs;
    type Entity = Account;
    const PREFIX: &'static str = "member.";
    const POSTURES: ApplicationMutationOutputPostureSet = ApplicationMutationOutputPostureSet::ALL;
    const MINIMUM: usize = 2;
}

#[test]
fn cascade_ownership_is_installed_identity_and_family_wide_meaning() {
    use worth_foundational::facade::ApplicationBuildIdentity;
    use worth_foundational::facade::ProducerPublicationOwnership;
    let independent = declared(TypeId::of::<InstalledProducer>());
    let mut owned = independent.clone();
    owned.ownership = ProducerPublicationOwnership::SourceScopeCascade;
    owned.owned_scope_entity = Some("Account".into());
    assert!(!independent.has_same_meaning_as(&owned));
    let build = ApplicationBuildIdentity::parse("11111111-1111-1111-1111-111111111111").unwrap();
    let edition = |declaration| {
        InstalledProducerEdition::from_declaration(declaration, build)
            .unwrap()
            .digest()
    };
    assert_ne!(edition(&independent), edition(&owned));
    let mut other_kind = owned.clone();
    other_kind.owned_scope_entity = Some("Principal".into());
    assert!(!owned.has_same_meaning_as(&other_kind));
    assert_ne!(edition(&owned), edition(&other_kind));
    let mut preserve = independent.clone();
    preserve.identity = "preserve".into();
    let mixed = std::collections::BTreeMap::from([
        ("initial".into(), owned.clone()),
        ("preserve".into(), preserve),
    ]);
    let denial = worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_contribution::producer::registry::scope_ownership::require_family_agreement(&mixed).unwrap_err();
    assert_eq!(denial.kind(), DenialKind::ProducerFamilyOwnershipMismatch);
    let mut preserve = owned.clone();
    preserve.identity = "preserve".into();
    assert!(
        worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_contribution::producer::registry::scope_ownership::require_family_agreement(&std::collections::BTreeMap::from([
            ("initial".into(), owned),
            ("preserve".into(), preserve),
        ]))
        .is_ok()
    );
}

#[path = "canonical_inventory/mod.rs"]
pub mod canonical_inventory;
#[path = "output_equivalence/mod.rs"]
pub mod output_equivalence;
