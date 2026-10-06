use std::any::TypeId;

use worth_query_declaration::facade::application_operation::{
    ApplicationMutationOutputPostureSet, NoApplicationMutationOutputs,
    WorthQueryApplicationDeclaredOutputRole as DeclaredRole,
    WorthQueryApplicationDeclaredOutputRoleFamily as DeclaredFamily,
    WorthQueryApplicationOutputRole, WorthQueryApplicationOutputRoleFamily,
    WorthQueryAtMostOneOutput, WorthQueryExactlyOneOutput, WorthQueryPreserveOutput,
};
use worth_query_declaration::facade::application_schema::ApplicationEntityMarkerIdentity;

use crate::domain_computation::primary_graph::tests::fixture::{
    Account, IdentityExecutionSchema as Schema,
};

use super::{
    checkpoint::{installed_checkpoint_producer, validate_checkpoint_output_meaning},
    operation_binding_uniqueness::duplicate_operation_binding,
    DeclaredProducerBinding, InstalledProducerEdition, InstalledProducerProvider,
};

#[test]
fn producer_input_reuse_is_portable_installed_meaning_bound_to_the_handler() {
    let base = declared(TypeId::of::<InstalledProducer>());
    let mut opted_in = base.clone();
    opted_in.input_reuse = Some(
        super::super::WorthQueryProducerInputReuseContract::canonical_bitwise(
            super::super::WorthQueryDecisionContextDependencies::NONE,
        ),
    );
    let mut changed_handler = opted_in.clone();
    changed_handler.handler_identity = "another-handler".into();
    let mut changed_dependencies = opted_in.clone();
    changed_dependencies.input_reuse = Some(
        super::super::WorthQueryProducerInputReuseContract::canonical_bitwise(
            super::super::WorthQueryDecisionContextDependencies::KEY,
        ),
    );

    assert!(!base.has_same_meaning_as(&opted_in));
    let edition = |declaration| {
        InstalledProducerEdition::from_declaration(declaration)
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
        for role in &mut checkpoint.roles {
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
    checkpoint.roles[0].entity_name = Account::IDENTIFIER.into();

    assert!(validate_checkpoint_output_meaning(&installed, &checkpoint)
        .unwrap_err()
        .contains("family member. is incomplete"));
}

fn checkpoint(
    roles: Vec<
        crate::domain_computation::primary_graph::application_attempt::WorthQueryCheckpointOutputRole,
    >,
) -> crate::domain_computation::primary_graph::application_output_demand::WorthQueryAcceptedOutputCheckpointIdentity{
    crate::domain_computation::primary_graph::application_output_demand::WorthQueryAcceptedOutputCheckpointIdentity {
        producer: "producer".into(),
        posture: crate::domain_computation::primary_graph::application_output_demand::WorthQueryAcceptedOutputCheckpointPosture::Performed,
        source: [0; 32],
        scope: crate::domain_computation::authorization::WorthQueryOperationScopeEntityBinding::from_entity(
            worth_relational::facade::identity::EntityId::new(
                worth_relational::facade::identity::PartitionId(1), 1, 1,
            ),
        ),
        source_partition: [0; 32],
        producer_dependency: None,
        idempotency_key: [0; 32],
        resources: None,
        roles,
            producer_facts: None,
            producer_fact_wire_version: 0,
    }
}

fn role(
    name: &str,
) -> crate::domain_computation::primary_graph::application_attempt::WorthQueryCheckpointOutputRole {
    crate::domain_computation::primary_graph::application_attempt::WorthQueryCheckpointOutputRole {
        role: name.into(),
        posture:
            crate::domain_computation::primary_graph::WorthQueryApplicationOutputPosture::Preserve,
        entity_name: "account".into(),
        entity: worth_relational::facade::identity::EntityId::new(
            worth_relational::facade::identity::PartitionId(1),
            1,
            1,
        ),
    }
}

fn declared(binding_type: TypeId) -> DeclaredProducerBinding {
    DeclaredProducerBinding {
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
        resource_policy: "bounded".into(),
        reuse_policy: "exact".into(),
        input_reuse: None,
        binding_type,
        source_type: TypeId::of::<()>(),
        operation_binding_type: TypeId::of::<()>(),
        provider_type: TypeId::of::<()>(),
        output_contract_type: TypeId::of::<()>(),
    }
}

struct InstalledProducer;
struct SubstitutedProducer;

struct Required;
impl WorthQueryApplicationOutputRole for Required {
    type Schema = Schema;
    type Contract = NoApplicationMutationOutputs;
    type Entity = Account;
    type Action = WorthQueryPreserveOutput;
    type Cardinality = WorthQueryExactlyOneOutput;
    const NAME: &'static str = "required";
}

struct Optional;
impl WorthQueryApplicationOutputRole for Optional {
    type Schema = Schema;
    type Contract = NoApplicationMutationOutputs;
    type Entity = Account;
    type Action = WorthQueryPreserveOutput;
    type Cardinality = WorthQueryAtMostOneOutput;
    const NAME: &'static str = "optional";
}

struct Member;
impl WorthQueryApplicationOutputRoleFamily for Member {
    type Schema = Schema;
    type Contract = NoApplicationMutationOutputs;
    type Entity = Account;
    const PREFIX: &'static str = "member.";
    const POSTURES: ApplicationMutationOutputPostureSet = ApplicationMutationOutputPostureSet::ALL;
    const MINIMUM: usize = 2;
}
