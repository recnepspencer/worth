use std::any::TypeId;

use super::DeclaredProducerBinding;

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

fn declared(binding_type: TypeId) -> DeclaredProducerBinding {
    DeclaredProducerBinding {
        owner: "owner".into(),
        identity: "producer".into(),
        source_selector: "source".into(),
        output_family: "family".into(),
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
