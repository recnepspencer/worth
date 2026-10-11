//! The fixture names are declared independently of Rust build identity.
use crate::domain_computation::primary_graph::output_lineage::invalidation::mark_state::FactPosting;
use std::{any::TypeId, collections::BTreeMap};

struct A;
struct B;

fn reverse_bindings() -> [(TypeId, &'static str); 2] {
    // Two independently declared fixture schemas put the same Rust markers in
    // opposite named roles. Exactly one reverses their build-specific order.
    let schemas = [
        [
            (TypeId::of::<A>(), "alpha-binding"),
            (TypeId::of::<B>(), "zulu-binding"),
        ],
        [
            (TypeId::of::<B>(), "alpha-binding"),
            (TypeId::of::<A>(), "zulu-binding"),
        ],
    ];
    schemas
        .into_iter()
        .find(|bindings| bindings[0].0 > bindings[1].0)
        .unwrap()
}

#[test]
fn postings_and_prior_output_visits_follow_declared_binding_order() {
    let bindings = reverse_bindings();
    assert!(bindings[0].0 > bindings[1].0);
    assert!(bindings[0].1 < bindings[1].1);
    let (mut lineage, seed) = super::super::registry_fixture::recorded_settlement();
    lineage.by_source.clear();
    let mut postings = im::OrdSet::new();
    for (binding, name) in bindings.into_iter().rev() {
        let mut source = seed.source().clone();
        source.output_binding = lineage.fixture_binding(binding, name);
        lineage.by_source.insert(source.clone(), BTreeMap::new());
        postings.insert(FactPosting {
            settlement: super::super::RecordedSettlementIdentity::retain(
                &source,
                super::super::ProductCoordinate {
                    occurrence: seed.address().0,
                    generation: 1,
                },
                0,
            ),
            ordinal: 0,
        });
    }
    let expected = [bindings[0].1, bindings[1].1];
    let posting_names: Vec<_> = postings
        .iter()
        .map(|p| p.settlement.source().output_binding.as_str())
        .collect();
    assert_eq!(posting_names, expected);
    // checkpoint_prior_outputs walks this exact owner index's keys and charges
    // the selected source's partition navigation before the next source visit.
    let visits: Vec<_> = lineage
        .by_source
        .keys()
        .map(|source| source.output_binding.as_str())
        .collect();
    assert_eq!(visits, expected);
}
