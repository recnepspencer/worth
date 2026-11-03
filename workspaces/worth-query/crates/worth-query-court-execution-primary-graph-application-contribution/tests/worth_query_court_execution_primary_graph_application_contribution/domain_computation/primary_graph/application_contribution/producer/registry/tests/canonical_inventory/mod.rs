//! Producer declarations order visits; equality alone deduplicates bindings.
use worth_query_execution::domain_computation::primary_graph::application_contribution::producer::registry::output_family_inventory::declared_output_family_bindings;
use crate::domain_computation::primary_graph::application_contribution::producer::registry::tests::declared;
use worth_query_execution::equality_type_id::EqualityTypeId as TypeId;
use std::collections::BTreeMap;

pub struct FirstBinding;
pub struct SecondBinding;

#[test]
fn binding_visits_follow_declared_producer_order_and_deduplicate_by_equality() {
    let a = TypeId::of::<FirstBinding>();
    let b = TypeId::of::<SecondBinding>();
    assert_ne!(a, b);
    for (first, second) in [(a, b), (b, a)] {
        let mut alpha = declared(first);
        alpha.identity = "alpha-producer".into();
        alpha.operation_binding_type = first;
        let mut middle = alpha.clone();
        middle.identity = "middle-producer".into();
        let mut zulu = declared(second);
        zulu.identity = "zulu-producer".into();
        zulu.operation_binding_type = second;
        let declarations = BTreeMap::from([
            (zulu.identity.as_str(), &zulu),
            (middle.identity.as_str(), &middle),
            (alpha.identity.as_str(), &alpha),
        ]);
        let families = declared_output_family_bindings(declarations.values().copied());
        assert_eq!(
            families["family"]
                .iter()
                .map(|(binding, _)| *binding)
                .collect::<Vec<_>>(),
            [first, second],
            "alpha precedes middle and zulu; middle repeats alpha's binding"
        );
    }
}
