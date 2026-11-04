//! The fixture names are declared independently of Rust build identity.
use worth_query_execution::court_doors_for_test::domain_computation::primary_graph::output_lineage::invalidation::mark_state::FactPosting;
use worth_query_execution::court_doors_for_test::equality_type_id::EqualityTypeId as TypeId;
use std::collections::BTreeMap;

pub struct A;
pub struct B;

pub fn binding_declarations() -> [[(TypeId, &'static str); 2]; 2] {
    // Two independently declared fixture schemas put the same Rust markers in
    // opposite named roles; both must follow the declared names.
    [
        [
            (TypeId::of_for_test::<A>(), "alpha-binding"),
            (TypeId::of_for_test::<B>(), "zulu-binding"),
        ],
        [
            (TypeId::of_for_test::<B>(), "alpha-binding"),
            (TypeId::of_for_test::<A>(), "zulu-binding"),
        ],
    ]
}

#[test]
fn postings_and_prior_output_visits_follow_declared_binding_order() {
    for bindings in binding_declarations() {
        assert_ne!(bindings[0].0, bindings[1].0);
        assert!(bindings[0].1 < bindings[1].1);
        let (mut lineage, seed) = worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::output_lineage::registry_fixture::recorded_settlement();
        (*lineage.by_source_mut_for_test()).clear();
        let mut postings = im::OrdSet::new();
        for (binding, name) in bindings.into_iter().rev() {
            let mut source = seed.source_for_test().clone();
            (*source.output_binding_mut_for_test()) = lineage.fixture_binding(binding, name);
            (*lineage.by_source_mut_for_test()).insert(source.clone(), BTreeMap::new());
            postings.insert(FactPosting::new_for_test(worth_query_execution::court_doors_for_test::domain_computation::primary_graph::output_lineage::settlement_identity::RecordedSettlementIdentity::retain_for_test(
                    &source,
                    worth_query_execution::court_doors_for_test::domain_computation::primary_graph::output_lineage::ProductCoordinate {
                        occurrence: seed.address_for_test().0,
                        generation: 1,
                    },
                    0,
                ), 0));
        }
        let expected = [bindings[0].1, bindings[1].1];
        let posting_names: Vec<_> = postings
            .iter()
            .map(|p| (*(*p.settlement_for_test()).source_for_test().output_binding_for_test()).as_str_for_test())
            .collect();
        assert_eq!(posting_names, expected);
        // checkpoint_prior_outputs walks this exact owner index's keys and charges
        // the selected source's partition navigation before the next source visit.
        let visits: Vec<_> = (*lineage
            .by_source_for_test())
            .keys()
            .map(|source| (*source.output_binding_for_test()).as_str_for_test())
            .collect();
        assert_eq!(visits, expected);
    }
}
