use std::any::TypeId;

use super::{operation_binding_uniqueness::duplicate_operation_binding, DeclaredProducerBinding};

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

fn declared(binding_type: TypeId) -> DeclaredProducerBinding {
    DeclaredProducerBinding {
        owner: "owner".into(),
        identity: "producer".into(),
        source_selector: "source".into(),
        output_family: "family".into(),
        output_family_type: TypeId::of::<()>(),
        output_roles: vec!["output".into()],
        output_role_families: Vec::new(),
        output_role: "output".into(),
        operation: "operation".into(),
        provider_identity: "provider".into(),
        applicability: Vec::new(),
        supported: Vec::new(),
        required_invariants: Vec::new(),
        resource_policy: "bounded".into(),
        reuse_policy: "exact".into(),
        binding_type,
        source_type: TypeId::of::<()>(),
        operation_binding_type: TypeId::of::<()>(),
        provider_type: TypeId::of::<()>(),
    }
}

struct InstalledProducer;
struct SubstitutedProducer;
