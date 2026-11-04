//! Producer declarations order visits; equality alone deduplicates bindings.
use worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_contribution::producer::registry::output_family_inventory::declared_output_family_bindings;
use crate::domain_computation::primary_graph::application_contribution::producer::registry::tests::declared;
use worth_query_execution::court_doors_for_test::equality_type_id::EqualityTypeId as TypeId;
use std::collections::BTreeMap;

pub struct FirstBinding;
pub struct SecondBinding;

#[test]
fn binding_visits_follow_declared_producer_order_and_deduplicate_by_equality() {
    let a = TypeId::of_for_test::<FirstBinding>();
    let b = TypeId::of_for_test::<SecondBinding>();
    assert_ne!(a, b);
    for (first, second) in [(a, b), (b, a)] {
        let mut alpha = declared(first);
        (*alpha.identity_mut_for_test()) = "alpha-producer".into();
        (*alpha.operation_binding_type_mut_for_test()) = first;
        let mut middle = alpha.clone();
        (*middle.identity_mut_for_test()) = "middle-producer".into();
        let mut zulu = declared(second);
        (*zulu.identity_mut_for_test()) = "zulu-producer".into();
        (*zulu.operation_binding_type_mut_for_test()) = second;
        let declarations = BTreeMap::from([
            ((*zulu.identity_for_test()).as_str(), &zulu),
            ((*middle.identity_for_test()).as_str(), &middle),
            ((*alpha.identity_for_test()).as_str(), &alpha),
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
